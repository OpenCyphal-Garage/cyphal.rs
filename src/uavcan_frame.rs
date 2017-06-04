use core::iter::Iterator;
use core::convert::{From, Into};

use bit::BitIndex;

use can_frame::{CanFrame,
                CanID,
                ToCanID,
};

use types::{
    UavcanPrimitiveType,
    UavcanPrimitiveField,
    UavcanIndexable,
    Bool,
    IntX,
    UintX,
    Float16
};


impl CanFrame {
    fn tail_byte(&self) -> TailByte {
        TailByte::from(self.data[self.dlc-1])
    }
    fn is_start_frame(&self) -> bool {
        self.tail_byte().start_of_transfer
    }
    fn is_end_frame(&self) -> bool {
        self.tail_byte().end_of_transfer
    }
    fn is_single_frame(&self) -> bool {
        self.is_end_frame() && self.is_start_frame()
    }
}


struct MessageFrameHeader {
    priority: u8,
    type_id: u16,
    source_node: u8,
}

struct AnonymousFrameHeader {
    priority: u8,
    discriminator: u16,
    type_id: u8,
    source_node: u8,
}

struct ServiceFrameHeader {
    priority: u8,
    type_id: u8,
    request_not_response: bool,
    destination_node: u8,
    source_node: u8,
}

impl ToCanID for MessageFrameHeader {
    fn to_can_id(&self) -> CanID {
        return CanID::Extended( ((self.priority as u32) << 24)&(0x1f000000) | ((self.type_id as u32) << 8)&(0x00ffff00) | (0u32 << 7) | ((self.source_node as u32))&(0x0000007f) );
    }
}

impl ToCanID for AnonymousFrameHeader {
    fn to_can_id(&self) -> CanID {
        return CanID::Extended( ((self.priority as u32) << 24)&(0x1f000000) | ((self.discriminator as u32) << 10)&(0x00fffc00) | ((self.type_id as u32) << 10)&(0x00000300) | (0u32 << 7) | ((self.source_node as u32))&(0x0000007f) );
    }
}

impl ToCanID for ServiceFrameHeader {
    fn to_can_id(&self) -> CanID {
        return CanID::Extended( ((self.priority as u32) << 24)&(0x1f000000) | ((self.type_id as u32) << 16)&(0x00ff0000) | ((self.request_not_response as u32) << 15) | ((self.destination_node as u32) << 8)&(0x00007f00) | (1u32 << 7) | ((self.source_node as u32) << 0)&(0x0000007f) );
    }
}

enum UavcanHeader {
    MessageFrameHeader(MessageFrameHeader),
    AnonymousFrameHeader(AnonymousFrameHeader),
    ServiceFrameHeader(ServiceFrameHeader),
}

impl ToCanID for UavcanHeader {
    fn to_can_id(&self) -> CanID {
        let can_id = match self {
            &UavcanHeader::MessageFrameHeader(ref x) => x.to_can_id(),
            &UavcanHeader::AnonymousFrameHeader(ref x) => x.to_can_id(),
            &UavcanHeader::ServiceFrameHeader(ref x) => x.to_can_id(),
        };
        return can_id;
    }
}

struct UavcanFrame<'a> {
    pub header: UavcanHeader,
    pub data: &'a [u8],
    pub transfer_id: u8,
}

struct CanFrameIterator<'a> {
    toggle: bool,
    data_pos: usize,
    uavcan_frame: &'a UavcanFrame<'a>,    
}

impl<'a> UavcanFrame<'a> {
    fn into_can_frame_iter(&'a self) -> CanFrameIterator<'a> {
        return CanFrameIterator{ data_pos:0, toggle: false,  uavcan_frame: self, };
    }
        
}

impl<'a> Iterator for CanFrameIterator<'a>{
    type Item = CanFrame;
    fn next(&mut self) -> Option<CanFrame>{
        if self.data_pos >= self.uavcan_frame.data.len() {
            return None;
        }
        
        let single_frame_transfer = self.uavcan_frame.data.len() < 8;
        let start_of_transfer = self.data_pos == 0;
        let end_of_transfer = self.uavcan_frame.data.len() - self.data_pos < 8;

        let can_id = self.uavcan_frame.header.to_can_id();

        let data_length =
            if end_of_transfer { self.uavcan_frame.data.len() - self.data_pos
            } else { 7 };
    
        let tail_byte = TailByte{start_of_transfer: start_of_transfer, end_of_transfer: end_of_transfer, toggle: self.toggle, transfer_id: self.uavcan_frame.transfer_id}; 
        
        let mut can_data: [u8; 8] = [0; 8];
        
        for i in 0..data_length {
            can_data[i] = self.uavcan_frame.data[self.data_pos + i];
        }
        can_data[data_length] = tail_byte.into();

        self.data_pos = self.data_pos + data_length;
        self.toggle = !self.toggle;

        return Some(CanFrame{id: can_id, dlc: data_length+1, data: can_data});
    }

}

struct TailByte {
    start_of_transfer: bool,
    end_of_transfer: bool,
    toggle: bool,
    transfer_id: u8,
}

impl From<TailByte> for u8 {
    fn from(tb: TailByte) -> u8 {
        ((tb.start_of_transfer as u8) << 7) | ((tb.end_of_transfer as u8) << 6) | ((tb.toggle as u8) << 5) | (tb.transfer_id&0x1f)
    }
}

impl From<u8> for TailByte {
    fn from(u: u8) -> TailByte {
        TailByte{start_of_transfer: (u&(1<<7)) != 0, end_of_transfer: (u&(1<<6)) != 0, toggle: (u&(1<<6)) != 0, transfer_id: u&0x1f}
    }
}

#[derive(Debug)]
pub enum BuilderError {
    FirstFrameNotStartFrame,
    BlockAddedAfterEndFrame,
    CRCError,
}

#[derive(Debug)]
pub enum ParseError {
    StructureExhausted,
}

struct Parser<T: UavcanIndexable> {
    message: T,
    current_field_index: usize,
    current_type_index: usize,
    buffer_end_bit: usize,
    buffer: [u8; 15],
}

impl<T: UavcanIndexable> Parser<T> {
    pub fn from_message(message: T) -> Parser<T> {
        Parser{message: message, current_field_index: 0, current_type_index: 0, buffer: [0; 15], buffer_end_bit: 0}
    }

    fn buffer_consume_bits(&mut self, number_of_bits: usize) {
        if number_of_bits > self.buffer_end_bit { panic!("Offset can't be larger than buffer_end_bit");}
        let new_buffer_len = self.buffer_end_bit - number_of_bits;
        let offset_byte = number_of_bits/8;
        let offset_bit = number_of_bits%8;
        for i in 0..((new_buffer_len+7)/8) {
            let bits_remaining = new_buffer_len - i*8;
            if offset_bit == 0 {
                self.buffer[i] = self.buffer[offset_byte+i];
            } else if bits_remaining + offset_bit < 8 {
                self.buffer[i] = self.buffer[offset_byte+i].bit_range(offset_bit..8) >> offset_bit;
            } else {
                self.buffer[i] = self.buffer[offset_byte+i].bit_range(offset_bit..8) >> offset_bit | self.buffer[offset_byte+1+i].bit_range(0..offset_bit) << 8-offset_bit;
            }
        }
        self.buffer_end_bit -= number_of_bits;
    }

    fn buffer_append(&mut self, tail: &[u8]) {
        let joint_byte = self.buffer_end_bit/8;
        let joint_bit = self.buffer_end_bit%8;

        for i in 0..tail.len() {
            if joint_bit == 0 {
                self.buffer[joint_byte + i] = tail[i];
            } else {
                self.buffer[joint_byte+i] = self.buffer[joint_byte + i].bit_range(0..joint_bit) | (tail[i].bit_range(0..8-joint_bit) << joint_bit);
                self.buffer[joint_byte+i+1] = tail[i].bit_range(8-joint_bit..8) >> 8-joint_bit;
            }
        }

        self.buffer_end_bit += tail.len()*8;
    }
    
    pub fn parse(mut self, input: &[u8]) -> Result<Parser<T>, ParseError> {
                
        for chunk in input.chunks(8) {
            self.buffer_append(chunk);

            loop {

                if self.message.primitive_field(self.current_field_index).is_some() {
                    if self.message.primitive_field(self.current_field_index).unwrap().primitive_type(self.current_type_index).is_some() {
                        
                        let field_length = self.message.primitive_field(self.current_field_index).unwrap().primitive_type(self.current_type_index).unwrap().bitlength();
                        if field_length <= self.buffer_end_bit {
                            self.message.primitive_field_as_mut(self.current_field_index).unwrap().primitive_type_as_mut(self.current_type_index).unwrap().set_from_bytes(&self.buffer[0..( (field_length+7)/8 )]);
                            self.buffer_consume_bits(field_length);
                            self.current_type_index += 1;
                        } else {
                            break;
                        }
                    } else {
                        self.current_type_index = 0;
                        self.current_field_index += 1;
                    }
                } else {
                    return Ok(self);
                }

            }

        }
        return Ok(self);
    }

    
}
                                                    
 


#[cfg(test)]
mod tests {
    use uavcan_frame::*;
    use can_frame::*;
    use core::fmt::*;
    use crc;

    impl PartialEq for CanID {
        fn eq(&self, other: &CanID) -> bool {
            return match (self, other) {
                (&CanID::Extended(ref x), &CanID::Extended(ref y)) => x == y,
                (&CanID::Normal(ref x), &CanID::Normal(ref y)) => x == y,
                _ => false,
            }
        }    
    }

    impl Debug for CanID {
        fn fmt(&self, f: &mut Formatter) -> Result {
            match self {
                &CanID::Extended(ref id) => write!(f, "Extended ID 0x{:x}", id),
                &CanID::Normal(ref id) => write!(f, "Normal ID 0x{:x}", id),
            }
        }
    }

    impl PartialEq for CanFrame {
        fn eq(&self, other: &CanFrame) -> bool {
            (self.id == other.id) && (self.dlc == other.dlc) && (self.data == other.data)
        }
    }
                
    
    impl Debug for CanFrame {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "CanFrame( id: {:?}, dlc: {}, and data: {:?})", self.id, self.dlc, self.data)
        }
    }
                
    
    #[test]
    fn message_frame_id() {
        assert_eq!(MessageFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72}.to_can_id(), CanID::Extended(0x1000aa72));
    }
    
    #[test]
    fn service_frame_id() {
        assert_eq!(ServiceFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72, destination_node: 0x11, request_not_response: true}.to_can_id(), CanID::Extended(0x10aa91f2));
    }
    
    #[test]
    fn uavcan_header_id() {
        assert_eq!(UavcanHeader::MessageFrameHeader(MessageFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72}).to_can_id(), CanID::Extended(0x1000aa72));
        assert_eq!(UavcanHeader::ServiceFrameHeader(ServiceFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72, destination_node: 0x11, request_not_response: true}).to_can_id(), CanID::Extended(0x10aa91f2));
    }

    #[test]
    fn can_frame_iterator_single1() {
        let uavcan_frame = UavcanFrame{header: UavcanHeader::MessageFrameHeader(MessageFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72}), data: &[1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8], transfer_id: 0x00};
        let can_frame = CanFrame{id: CanID::Extended(0x1000aa72), dlc: 8, data: [1, 2, 3, 4, 5, 6, 7, TailByte{start_of_transfer: true, end_of_transfer: true, toggle: false, transfer_id: 0x00}.into()]};

        assert_eq!(uavcan_frame.into_can_frame_iter().next().unwrap(), can_frame); // fix tail byte
    }

    #[test]
    fn can_frame_iterator_multi1() {
        let mut data = [0u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];
        
        let crc = crc::calc(&data[2..], 0x00);
        let crc0 = crc as u8;
        let crc1 = (crc >> 8) as u8;

        data[0] = crc0;
        data[1] = crc1;
        
        let uavcan_frame = UavcanFrame{header: UavcanHeader::MessageFrameHeader(MessageFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72}), data: &data, transfer_id: 0x00};
        
        let can_frame0 = CanFrame{id: CanID::Extended(0x1000aa72), dlc: 8, data: [crc0, crc1, 1, 2, 3, 4, 5, TailByte{start_of_transfer: true, end_of_transfer: false, toggle: false, transfer_id: 0x00}.into()]};
        let can_frame1 = CanFrame{id: CanID::Extended(0x1000aa72), dlc: 4, data: [6, 7, 8, TailByte{start_of_transfer: false, end_of_transfer: true, toggle: true, transfer_id: 0x00}.into(), 0, 0, 0, 0]};

        let mut can_frame_iter = uavcan_frame.into_can_frame_iter();
        
        assert_eq!(can_frame_iter.next().unwrap(), can_frame0);
        assert_eq!(can_frame_iter.next().unwrap(), can_frame1);

    }

    #[test]
    fn can_frame_iterator_multi2() {
        let mut data = [0u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];
        
        let crc = crc::calc(&data[2..], 0x00);
        let crc0 = crc as u8;
        let crc1 = (crc >> 8) as u8;

        data[0] = crc0;
        data[1] = crc1;
        
        let uavcan_frame = UavcanFrame{header: UavcanHeader::MessageFrameHeader(MessageFrameHeader{priority: 0x10, type_id: 0xaa, source_node: 0x72}), data: &data, transfer_id: 0x00};
        
        let can_frame0 = CanFrame{id: CanID::Extended(0x1000aa72), dlc: 8, data: [crc0, crc1, 1, 2, 3, 4, 5, TailByte{start_of_transfer: true, end_of_transfer: false, toggle: false, transfer_id: 0x00}.into()]};
        let can_frame1 = CanFrame{id: CanID::Extended(0x1000aa72), dlc: 4, data: [6, 7, 8, TailByte{start_of_transfer: false, end_of_transfer: true, toggle: true, transfer_id: 0x00}.into(), 0, 0, 0, 0]};

        let mut can_frame_iter = uavcan_frame.into_can_frame_iter();
        
        assert_eq!(can_frame_iter.next().unwrap(), can_frame0);
        assert_eq!(can_frame_iter.next().unwrap(), can_frame1);
        assert_eq!(can_frame_iter.next(), None);
        
    }

    #[test]
    fn uavcan_sized_length_derivation() {
        
        #[derive(UavcanIndexable)]
        struct NodeStatus {
            uptime_sec: UintX,
            health: UintX,
            mode: UintX,
            sub_mode: UintX,
            vendor_specific_status_code: UintX,
        }

        impl NodeStatus {
            fn new() -> NodeStatus{
                NodeStatus {
                    uptime_sec: UintX::new(32, 0),
                    health: UintX::new(2, 0),
                    mode: UintX::new(3, 0),
                    sub_mode: UintX::new(3, 0),
                    vendor_specific_status_code: UintX::new(16, 0),
                }
            }
        }

        
        #[derive(UavcanIndexable)]
        struct TestComposite {
            ns1: NodeStatus,
            ns2: NodeStatus,
        }

        impl TestComposite {
            fn new() -> TestComposite {
                TestComposite{
                    ns1: NodeStatus::new(),
                    ns2: NodeStatus::new(),
                }
            }
        }

        #[derive(UavcanIndexable)]
        struct TestComposite2 {
            ns1: NodeStatus,
            tc: TestComposite,
            ns2: NodeStatus,
        }

        impl TestComposite2 {
            fn new() -> TestComposite2 {
                TestComposite2{
                    ns1: NodeStatus::new(),
                    tc: TestComposite::new(),
                    ns2: NodeStatus::new(),
                }
            }
        }

        
        assert_eq!(NodeStatus::new().number_of_primitive_fields(), 5);
        assert_eq!(TestComposite::new().number_of_primitive_fields(), 10);
        assert_eq!(TestComposite2::new().number_of_primitive_fields(), 20);
        
        
    }

    #[test]
    fn uavcan_index_primitive_field() {

        #[derive(UavcanIndexable)]
        struct NodeStatus {
            uptime_sec: UintX,
            health: UintX,
            mode: UintX,
            sub_mode: UintX,
            vendor_specific_status_code: UintX,
        }

        impl NodeStatus {
            fn new() -> NodeStatus{
                NodeStatus {
                    uptime_sec: UintX::new(32, 0),
                    health: UintX::new(2, 0),
                    mode: UintX::new(3, 0),
                    sub_mode: UintX::new(3, 0),
                    vendor_specific_status_code: UintX::new(16, 0),
                }
            }
        }

        let mut node_status = NodeStatus::new();

        node_status.primitive_field_as_mut(0).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[1, 0, 0, 0]);
        node_status.primitive_field_as_mut(1).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[2]);
        node_status.primitive_field_as_mut(2).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[3]);
        node_status.primitive_field_as_mut(3).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[4]);
        node_status.primitive_field_as_mut(4).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[5, 0]);

        node_status.health.primitive_field_as_mut(0).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[2, 0, 0, 0]);
        
        assert_eq!(node_status.uptime_sec, UintX::new(32, 1));
        assert_eq!(node_status.health, UintX::new(2, 2));
        assert_eq!(node_status.mode, UintX::new(3, 3));
        assert_eq!(node_status.sub_mode, UintX::new(3, 4));
        assert_eq!(node_status.vendor_specific_status_code, UintX::new(16, 5));
        
    }

    #[test]
    fn uavcan_parse_test_byte_aligned() {

        #[derive(UavcanIndexable)]
        struct Message {
            v1: UintX,
            v2: UintX,
            v3: UintX,
            v4: UintX,
        }

        impl Message {
            fn new() -> Message{
                Message {
                    v1: UintX::new(8, 0),
                    v2: UintX::new(32, 0),
                    v3: UintX::new(16, 0),
                    v4: UintX::new(8, 0),
                }
            }
        }

        let mut message = Message::new();
        
        let mut parser = Parser::from_message(message);

        parser = parser.parse(&[17, 19, 0, 0, 0, 21, 0, 23]).unwrap();

        let parsed_message = parser.message;

        
        assert_eq!(parsed_message.v1, UintX::new(8,17));
        assert_eq!(parsed_message.v2, UintX::new(32,19));
        assert_eq!(parsed_message.v3, UintX::new(16,21));
        assert_eq!(parsed_message.v4, UintX::new(8,23));
    }




    #[test]
    fn uavcan_parse_test_misaligned() {

        #[derive(UavcanIndexable)]
        struct NodeStatus {
            uptime_sec: UintX,
            health: UintX,
            mode: UintX,
            sub_mode: UintX,
            vendor_specific_status_code: UintX,
        }

        impl NodeStatus {
            fn new() -> NodeStatus{
                NodeStatus {
                    uptime_sec: UintX::new(32, 0),
                    health: UintX::new(2, 0),
                    mode: UintX::new(3, 0),
                    sub_mode: UintX::new(3, 0),
                    vendor_specific_status_code: UintX::new(16, 0),
                }
            }
        }

        let mut node_status_message = NodeStatus::new();
        
        let mut parser = Parser::from_message(node_status_message);

        parser = parser.parse(&[1, 0, 0, 0, 0b10001110, 5, 0]).unwrap();

        let parsed_message = parser.message;
        

        assert_eq!(parsed_message.uptime_sec, UintX::new(32, 1));
        assert_eq!(parsed_message.health, UintX::new(2, 2));
        assert_eq!(parsed_message.mode, UintX::new(3, 3));
        assert_eq!(parsed_message.sub_mode, UintX::new(3, 4));
        assert_eq!(parsed_message.vendor_specific_status_code, UintX::new(16, 5));
        
    }


    
    
}

