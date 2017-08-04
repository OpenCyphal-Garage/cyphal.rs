use bit_field::BitField;

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
            let crc = self.serializer.crc(0xd654a48e0c049d75);
            transport_frame.data_as_mut()[0] = crc.get_bits(0..8) as u8;
            transport_frame.data_as_mut()[1] = crc.get_bits(8..16) as u8;
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




#[cfg(test)]
mod tests {

    use{
        UavcanIndexable,
        UavcanField,
        UavcanHeader,
        MessageFrameHeader,
        UavcanFrame,
        TailByte,
        DynamicArray,
    };

    use bit_field::BitField;
    
    use types::{
        Uint2,
        Uint3,
        Uint8,
        Uint16,
        Uint32,
        DynamicArray31,
        DynamicArray90,
    };

    use tests::{
        CanFrame,
        CanID,
    };

    
    use frame_generator::{
        FrameGenerator,
    };
    
    #[test]
    fn serialize_node_status_frame() {

        #[derive(UavcanIndexable, Default)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        message_frame_header!(NodeStatusHeader, 341);
        
        #[derive(UavcanFrame, Default)]
        struct NodeStatusMessage {
            header: NodeStatusHeader,
            body: NodeStatus,
        }
            
        let can_frame = CanFrame{id: CanID::Extended(NodeStatusHeader::new(0, 32).to_id()), dlc: 8, data: [1, 0, 0, 0, 0b10001110, 5, 0, TailByte{start_of_transfer: true, end_of_transfer: true, toggle: false, transfer_id: 0}.into()]};

        let uavcan_frame = NodeStatusMessage{
            header: NodeStatusHeader::new(0, 32),
            body: NodeStatus{
                uptime_sec: 1.into(),
                health: 2.into(),
                mode: 3.into(),
                sub_mode: 4.into(),
                vendor_specific_status_code: 5.into(),
            },
        };

        let mut frame_generator = FrameGenerator::from_uavcan_frame(uavcan_frame, 0);

        assert_eq!(frame_generator.next_transport_frame(), Some(can_frame));
        assert_eq!(frame_generator.next_transport_frame::<CanFrame>(), None);
        
    }
    
    #[test]
    fn serialize_multi_frame() {

        #[derive(UavcanIndexable)]
        struct LogLevel {
            value: Uint3,
        }
        
        #[derive(UavcanIndexable)]
        struct LogMessage {
            level: LogLevel,
            source: DynamicArray31<Uint8>,
            text: DynamicArray90<Uint8>,
        }

        message_frame_header!(LogMessageHeader, 16383);

        #[derive(UavcanFrame)]
        struct LogMessageMessage {
            header: LogMessageHeader,
            body: LogMessage,
        }
        
        let uavcan_frame = LogMessageMessage{
            header: LogMessageHeader::new(0, 32),
            body: LogMessage{
                level: LogLevel{value: 0.into()},
                source: DynamicArray31::with_str("test source"),
                text: DynamicArray90::with_str("test text"),
            },
        };

        assert_eq!(uavcan_frame.body.number_of_primitive_fields(), 21);
        
        let mut frame_generator = FrameGenerator::from_uavcan_frame(uavcan_frame, 0);

        let crc = frame_generator.serializer.crc(0xd654a48e0c049d75);

        
        assert_eq!(
            frame_generator.next_transport_frame(),
            Some(CanFrame{
                id: CanID::Extended(LogMessageHeader::new(0, 32).to_id()),
                dlc: 8,
                data: [crc.get_bits(0..8) as u8, crc.get_bits(8..16) as u8, 0u8.set_bits(3..8, 11).get_bits(0..8), 't' as u8, 'e' as u8, 's' as u8, 't' as u8, TailByte{start_of_transfer: true, end_of_transfer: false, toggle: false, transfer_id: 0}.into()],
            })
        );
        


        
    }

}
