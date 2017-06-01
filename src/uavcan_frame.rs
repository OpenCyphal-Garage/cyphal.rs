use core::iter::Iterator;
use core::convert::{From, Into};

use can_frame::{CanFrame,
                CanID,
                ToCanID,
};

use crc;


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
    



#[cfg(test)]
mod tests {
    use uavcan_frame::*;
    use can_frame::*;
    use core::fmt::*;

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
    
}

