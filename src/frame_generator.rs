use {
    TransportFrame,
    UavcanFrame,
    UavcanHeader,
    UavcanIndexable,
};

use serializer::{
    Serializer,
};



pub struct FrameGenerator<B: UavcanIndexable> {
    serializer: Serializer<B>,
    started: bool,
    id: u32,
    toggle: bool,
    transfer_id: u8,
}

impl<B: UavcanIndexable> FrameGenerator<B> {
    pub fn from_uavcan_frame<H: UavcanHeader, F: UavcanFrame<H, B>>(frame: F, transfer_id: u8) -> Self {
        let (header, body) = frame.to_parts();
        Self{
            serializer: Serializer::from_structure(body),
            started: false,
            id: header.to_id(),
            toggle: false,
            transfer_id: transfer_id,
        }
    }
    
}
