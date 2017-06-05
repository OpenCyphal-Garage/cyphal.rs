use core::iter::Iterator;
use core::convert::{From, Into};

use bit::BitIndex;

use types::{
    UavcanPrimitiveType,
    UavcanPrimitiveField,
    UavcanIndexable,
    Bool,
    IntX,
    UintX,
    Float16
};


/// The TransportFrame is uavcan cores main interface to the outside world
///
/// This will in >99% of situations be a CAN2.0B frame
/// But in theory both CAN-FD and other protocols which gives
/// similar guarantees as CAN can also be used
pub trait TransportFrame {
    fn get_tail_byte(&self) -> TailByte {
        TailByte::from(*self.get_data().last().unwrap())
    }
    fn is_start_frame(&self) -> bool {
        self.get_tail_byte().start_of_transfer
    }
    fn is_end_frame(&self) -> bool {
        self.get_tail_byte().end_of_transfer
    }
    fn is_single_frame(&self) -> bool {
        self.is_end_frame() && self.is_start_frame()
    }

    /// with_data(id: u32, data: &[u]) -> TransportFrame creates a TransportFrame
    /// with an 28 bits ID and data between 0 and the return value ofget_max_data_length()
    fn with_data(id: u32,  data: &[u8]) -> Self;
    fn get_max_data_length() -> usize;
    fn get_data(&self) -> &[u8];
    fn get_id(&self) -> u32;
}

pub struct TailByte {
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

pub trait TransportFrameHeader {
    fn to_id(&self) -> u32;
    fn set_priority(&mut self, priority: u8);
    fn get_priority(&self) -> u8;
}

pub trait UavcanTransmitable : UavcanIndexable {
    fn get_header(&self) -> &TransportFrameHeader;
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
}

struct ServiceFrameHeader {
    priority: u8,
    type_id: u8,
    request_not_response: bool,
    destination_node: u8,
    source_node: u8,
}

impl TransportFrameHeader for MessageFrameHeader {
    fn to_id(&self) -> u32 {
        return ((self.priority as u32) << 24)&(0x1f000000) | ((self.type_id as u32) << 8)&(0x00ffff00) | (0u32 << 7) | ((self.source_node as u32))&(0x0000007f);
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bit_range(0..5, priority);
    }
    fn get_priority(&self) -> u8 {
        self.priority.bit_range(0..5)
    }    
}

impl TransportFrameHeader for AnonymousFrameHeader {
    fn to_id(&self) -> u32 {
        return ((self.priority as u32) << 24)&(0x1f000000) | ((self.discriminator as u32) << 10)&(0x00fffc00) | ((self.type_id as u32) << 10)&(0x00000300) | (0u32 << 7);
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bit_range(0..5, priority);
    }
    fn get_priority(&self) -> u8{
        self.priority.bit_range(0..5)
    }
}

impl TransportFrameHeader for ServiceFrameHeader {
    fn to_id(&self) -> u32 {
        return ((self.priority as u32) << 24)&(0x1f000000) | ((self.type_id as u32) << 16)&(0x00ff0000) | ((self.request_not_response as u32) << 15) | ((self.destination_node as u32) << 8)&(0x00007f00) | (1u32 << 7) | ((self.source_node as u32) << 0)&(0x0000007f);
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bit_range(0..5, priority);
    }
    fn get_priority(&self) -> u8{
        self.priority.bit_range(0..5)
    }
}


struct UavcanFrame<H: TransportFrameHeader, B: UavcanIndexable> {
    header: H,
    body: B,
}

impl<H:TransportFrameHeader, B: UavcanIndexable> UavcanIndexable for UavcanFrame<H, B> {
    fn number_of_primitive_fields(&self) -> usize {
        self.body.number_of_primitive_fields()
    }
    fn primitive_field_as_mut(&mut self, field_number: usize) -> Option<&mut UavcanPrimitiveField>{
        self.body.primitive_field_as_mut(field_number)
    }       
    fn primitive_field(&self, field_number: usize) -> Option<&UavcanPrimitiveField>{
        self.body.primitive_field(field_number)
    }
}

impl<H: TransportFrameHeader, B: UavcanIndexable> UavcanTransmitable for UavcanFrame<H, B> {
    fn get_header(&self) -> &TransportFrameHeader {
        &self.header
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
    structure: T,
    current_field_index: usize,
    current_type_index: usize,
    buffer_end_bit: usize,
    buffer: [u8; 15],
}

impl<T: UavcanIndexable> Parser<T> {
    pub fn from_structure(structure: T) -> Parser<T> {
        Parser{structure: structure, current_field_index: 0, current_type_index: 0, buffer: [0; 15], buffer_end_bit: 0}
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
                let bitmask = self.buffer[offset_byte+i].bit_range(offset_bit..8);
                self.buffer[i]
                    .set_bit_range(0..8-offset_bit, bitmask);
            } else {
                let lsb = self.buffer[offset_byte+i].bit_range(offset_bit..8);
                let msb = self.buffer[offset_byte+i+1].bit_range(0..offset_bit);
                
                self.buffer[i]
                    .set_bit_range(0..8-offset_bit, lsb)
                    .set_bit_range(8-offset_bit..8, msb);
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

                if self.structure.primitive_field(self.current_field_index).is_some() {
                    if self.structure.primitive_field(self.current_field_index).unwrap().primitive_type(self.current_type_index).is_some() {
                        
                        let field_length = self.structure.primitive_field(self.current_field_index).unwrap().primitive_type(self.current_type_index).unwrap().bitlength();
                        if field_length <= self.buffer_end_bit {
                            self.structure.primitive_field_as_mut(self.current_field_index).unwrap().primitive_type_as_mut(self.current_type_index).unwrap().set_from_bytes(&self.buffer[0..( (field_length+7)/8 )]);
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
                    if self.buffer_end_bit >= 8 {
                        return Err(ParseError::StructureExhausted);
                    } else {
                        return Ok(self);
                    }
                }

            }

        }
        return Ok(self);
    }

    pub fn to_structure(self) -> T {
        self.structure
    }
}




#[cfg(test)]
mod tests {
    use uavcan_frame::*;
    use core::fmt::*;
    use crc;

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
        
        let mut parser = Parser::from_structure(message);

        parser = parser.parse(&[17, 19, 0, 0, 0, 21, 0, 23]).unwrap();

        let parsed_message = parser.to_structure();

        
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
        
        let mut parser = Parser::from_structure(node_status_message);

        parser = parser.parse(&[1, 0, 0, 0, 0b10001110, 5, 0]).unwrap();

        let parsed_message = parser.to_structure();
        

        assert_eq!(parsed_message.uptime_sec, UintX::new(32, 1));
        assert_eq!(parsed_message.health, UintX::new(2, 2));
        assert_eq!(parsed_message.mode, UintX::new(3, 3));
        assert_eq!(parsed_message.sub_mode, UintX::new(3, 4));
        assert_eq!(parsed_message.vendor_specific_status_code, UintX::new(16, 5));
        
    }


    
    
}

