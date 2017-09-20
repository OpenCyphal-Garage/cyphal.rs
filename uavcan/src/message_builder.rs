use bit_field::BitField;

use {
    UavcanFrame,
    UavcanStruct,
    TransportFrame,
    UavcanHeader,
};

use deserializer::{
    Deserializer,
};

use crc::calc;

#[derive(Debug)]
pub enum BuilderError {
    FirstFrameNotStartFrame,
    BlockAddedAfterEndFrame,
    ToggleError,
    CRCError,
    FormatError,
    IdError,
    NotFinishedParsing,
}

pub struct MessageBuilder<B: UavcanStruct> {
    deserializer: Deserializer<B>,
    started: bool,
    id: u32,
    crc: u16,
    crc_calculated: u16,
    toggle: bool,
    transfer_id: u8,    
}

impl<B: UavcanStruct> MessageBuilder<B> {
    pub fn new() -> Self {
        MessageBuilder{
            deserializer: Deserializer::new(),
            started: false,
            id: 0x00,
            crc: 0x00,
            crc_calculated: 0xffff,
            toggle: false,
            transfer_id: 0x00,
        }
    }
    
    pub fn add_frame<F: TransportFrame>(&mut self, frame: &F) -> Result<(), BuilderError> {
        if !self.started {
            if !frame.is_start_frame() {
                return Err(BuilderError::FirstFrameNotStartFrame);
            }
            if frame.tail_byte().toggle {
                return Err(BuilderError::ToggleError);
            }
            self.toggle = false;
            self.crc.set_bits(0..8, frame.data()[0] as u16)
                .set_bits(8..16, frame.data()[1] as u16); 
            self.transfer_id = frame.tail_byte().transfer_id;
            self.id = frame.id();
            self.started = true;
        }

        let payload = if frame.is_start_frame() && !frame.is_end_frame() {
            &frame.data()[2..frame.data().len()-1]
        } else {
            &frame.data()[0..frame.data().len()-1]
        };

        self.deserializer.deserialize(payload);            

        return Ok(());
    }

    pub fn build<H: UavcanHeader, F: UavcanFrame<H, B>>(self) -> Result<F, BuilderError> {
        let header = if let Ok(id) = H::from_id(self.id) {
            id
        } else {
            return Err(BuilderError::IdError)
        };

        let body = if let Ok(body) = self.deserializer.into_structure() {
            body
        } else {
            return Err(BuilderError::NotFinishedParsing)
        };

        Ok(F::from_parts(header, body))
    }
                
}


#[cfg(test)]
mod tests {

    use bit_field::BitField;
    
    use{
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
        UavcanHeader,
        MessageFrameHeader,
        UavcanFrame,
        TailByte,
        DynamicArray,
        SerializationBuffer,
        UavcanPrimitiveType,
    };
    
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

    use message_builder::{
        MessageBuilder,
    };

    use frame_generator::{
        FrameGenerator,
    };

    use serializer::*;
    
    #[test]
    fn parse_from_can_frames_simple() {

        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        message_frame_header!(NodeStatusHeader, 341);

        uavcan_frame!(NodeStatusMessage, NodeStatusHeader, NodeStatus, 0);
            
        
        let can_frame = CanFrame{id: CanID(NodeStatusHeader::new(0, 32).id()), dlc: 8, data: [1, 0, 0, 0, 0b10001110, 5, 0, TailByte{start_of_transfer: true, end_of_transfer: true, toggle: false, transfer_id: 0}.into()]};
        
        let mut message_builder = MessageBuilder::new();
        message_builder.add_frame(&can_frame);
        let parsed_message: NodeStatusMessage = message_builder.build().unwrap();
        
        assert_eq!(parsed_message.body.uptime_sec, 1.into());
        assert_eq!(parsed_message.body.health, 2.into());
        assert_eq!(parsed_message.body.mode, 3.into());
        assert_eq!(parsed_message.body.sub_mode, 4.into());
        assert_eq!(parsed_message.body.vendor_specific_status_code, 5.into());
        assert_eq!(parsed_message.header, NodeStatusHeader::new(0, 32));
                                              
    }
    
    #[test]
    fn deserialize_multi_frame() {
        
        #[derive(Debug, PartialEq, UavcanStruct)]
        struct LogLevel {
            value: Uint3,
        }
        
        #[derive(Debug, PartialEq, UavcanStruct)]
        struct LogMessage {
            level: LogLevel,
            source: DynamicArray31<Uint8>,
            text: DynamicArray90<Uint8>,
        }
        
        message_frame_header!(LogMessageHeader, 16383);

        uavcan_frame!(derive(Debug, PartialEq), LogMessageMessage, LogMessageHeader, LogMessage, 0xd654a48e0c049d75);

        let uavcan_frame = LogMessageMessage{
            header: LogMessageHeader::new(0, 32),
            body: LogMessage{
                level: LogLevel{value: 0.into()},
                source: DynamicArray31::with_str("test source"),
                text: DynamicArray90::with_str("test text"),
            },
        };

        let crc = 0;
        let mut message_builder = MessageBuilder::new();
        
        message_builder.add_frame(&CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 8,
            data: [crc.get_bits(0..8) as u8, crc.get_bits(8..16) as u8, 0u8.set_bits(0..3, 0).set_bits(3..8, 11).get_bits(0..8), b't', b'e', b's', b't', TailByte{start_of_transfer: true, end_of_transfer: false, toggle: false, transfer_id: 0}.into()],
        });
        
        message_builder.add_frame(&CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 8,
            data: [b' ', b's', b'o', b'u', b'r', b'c', b'e', TailByte{start_of_transfer: false, end_of_transfer: false, toggle: true, transfer_id: 0}.into()],
        });
        
        message_builder.add_frame(&CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 8,
            data: [b't', b'e', b's', b't', b' ', b't', b'e', TailByte{start_of_transfer: false, end_of_transfer: false, toggle: false, transfer_id: 0}.into()],
        });
        
        message_builder.add_frame(&CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 3,
            data: [b'x', b't', TailByte{start_of_transfer: false, end_of_transfer: true, toggle: true, transfer_id: 0}.into(), 0, 0, 0, 0, 0],
        });

        assert_eq!(uavcan_frame.body.source.length().current_length, 11);
        assert_eq!(uavcan_frame, message_builder.build().unwrap());
        
    }
   


}

