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

enum UavcanHeader {
    MessageFrameHeader(MessageFrameHeader),
    AnonymousFrameHeader(AnonymousFrameHeader),
    ServiceFrameHeader(ServiceFrameHeader),
}

struct UavcanFrame {
    data_pos: usize,
    pub header: UavcanHeader,
    pub data: [u8],
}

impl Iterator for UavcanFrame {
    type Item = CanFrame;
    fn next(&mut self) -> Option<CanFrame>{
        let single_frame_transfer = self.data.len() < 8;
        let first_frame = self.data_pos == 0;
        let last_frame = self.data.len() - self.data_pos < 8;

        let can_id = 0x00; // fix value
        
                
        return Some(CanFrame{id: CanID::Extended(can_id), dlc: 0, data: [0;8]});
    }
}

