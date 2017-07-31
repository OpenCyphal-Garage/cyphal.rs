use {
    TailByte,
    TransportFrame,
    UavcanFrame,
    UavcanHeader,
    UavcanIndexable,
};

use serializer::{
    SerializationResult,
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
    
    pub fn next_transport_frame<T: TransportFrame>(&mut self) -> Option<T> {
        let remaining_bits = self.serializer.remaining_bits();
        let max_data_length = T::max_data_length();
        let max_payload_length = max_data_length - 1;
        let mut transport_frame = T::with_length(self.id, max_data_length);

        
        let first_of_multi_frame = !self.started && (remaining_bits > max_payload_length*8);

        if remaining_bits == 0 {
            return None;
        } else if first_of_multi_frame {
            // TODO: calc crc
            self.serializer.serialize(&mut transport_frame.data_as_mut()[2..max_data_length-1]);
            transport_frame.data_as_mut()[max_data_length-1] = TailByte{start_of_transfer: !self.started, end_of_transfer: false, toggle: self.toggle, transfer_id: self.transfer_id}.into();
        } else {
            if let SerializationResult::Finished(i) = self.serializer.serialize(&mut transport_frame.data_as_mut()[0..max_data_length-1]){
                let frame_length = (i+7)/8 + 1;
                transport_frame.set_data_length(frame_length);
                transport_frame.data_as_mut()[frame_length-1] = TailByte{start_of_transfer: !self.started, end_of_transfer: true, toggle: self.toggle, transfer_id: self.transfer_id}.into();
            }
        }

        self.started = true;
        self.toggle = !self.toggle;
        
        return Some(transport_frame);
    }
}


