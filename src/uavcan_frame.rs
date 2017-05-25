use core::iter::Iterator;

use can_frame::{CanFrame,
                CanID,
                ToCanID,
};


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

struct UavcanFrame {
    pub header: UavcanHeader,
    pub data: [u8],
}

struct CanFrameIterator<'a> {
    data_pos: usize,
    uavcan_frame: &'a UavcanFrame,    
}

impl UavcanFrame {
    fn into_can_frame_iter<'a>(&'a self) -> CanFrameIterator<'a> {
        return CanFrameIterator{ data_pos:0, uavcan_frame: self, };
    }
        
}

impl<'a> Iterator for CanFrameIterator<'a>{
    type Item = CanFrame;
    fn next(&mut self) -> Option<CanFrame>{
        let single_frame_transfer = self.uavcan_frame.data.len() < 8;
        let first_frame = self.data_pos == 0;
        let last_frame = self.uavcan_frame.data.len() - self.data_pos < 8;

        let can_id = self.uavcan_frame.header.to_can_id();
        let dlc =
            if last_frame { self.uavcan_frame.data.len() - self.data_pos + 1
            } else { 8 };

        let mut can_data: [u8; 8] = [0; 8];
        
        for i in 0..dlc {
            can_data[i] = self.uavcan_frame.data[self.data_pos + i];
        }
        
        self.data_pos = self.data_pos + dlc;
                        
        return Some(CanFrame{id: can_id, dlc: dlc, data: can_data});
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
}

