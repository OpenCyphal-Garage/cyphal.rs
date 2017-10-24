use bit_field::BitField;

use transfer::{
    TransferFrame,
    TransferFrameID,
    TransferID,
};

use {
    Frame,
    Struct,
};

use deserializer::{
    Deserializer,
};

#[derive(Debug)]
pub enum AssemblerResult {
    Ok,
    Finished,
}

#[derive(Debug)]
pub enum AssemblerError {
    FirstFrameNotStartFrame,
    BlockAddedAfterEndFrame,
    ToggleError,
}

#[derive(Debug)]
pub enum BuildError {
    CRCError,
    IdError,
    NotFinishedParsing,
}

pub(crate) struct FrameAssembler<S: Struct> {
    deserializer: Deserializer<S>,
    started: bool,
    id: TransferFrameID,
    crc: u16,
    crc_calculated: u16,
    toggle: bool,
    transfer_id: TransferID,    
}

impl<S: Struct> FrameAssembler<S> {
    pub fn new() -> Self {
        Self{
            deserializer: Deserializer::new(),
            started: false,
            id: TransferFrameID::new(0x00),
            crc: 0x00,
            crc_calculated: 0xffff,
            toggle: false,
            transfer_id: TransferID::new(0x00),
        }
    }
    
    pub fn add_transfer_frame<T: TransferFrame>(&mut self, mut frame: T) -> Result<AssemblerResult, AssemblerError> {
        let end_frame = frame.is_end_frame();
        
        if !self.started {
            if !frame.is_start_frame() {
                return Err(AssemblerError::FirstFrameNotStartFrame);
            }
            if frame.tail_byte().toggle() {
                return Err(AssemblerError::ToggleError);
            }
            self.toggle = false;
            self.crc.set_bits(0..8, frame.data()[0] as u16)
                .set_bits(8..16, frame.data()[1] as u16); 
            self.transfer_id = frame.tail_byte().transfer_id();
            self.id = frame.id();
            self.started = true;
        }

        let data_len = frame.data().len();
        let payload = if frame.is_start_frame() && !frame.is_end_frame() {
            &mut frame.data_as_mut()[2..data_len-1]
        } else {
            &mut frame.data_as_mut()[0..data_len-1]
        };

        self.deserializer.deserialize(payload);            

        if end_frame {
            Ok(AssemblerResult::Finished)
        } else {
            Ok(AssemblerResult::Ok)
        }
    }

    pub fn build(self) -> Result<Frame<S>, BuildError> {
        
        let body = if let Ok(body) = self.deserializer.into_structure() {
            body
        } else {
            return Err(BuildError::NotFinishedParsing)
        };

        Ok(Frame::from_parts(self.id, body))
    }
                
}


#[cfg(test)]
mod tests {

    use bit_field::BitField;

    use uavcan;
    
    use tests::{
        CanFrame,
    };
    
    use *;    
    use types::*;

    use transfer::{
        TransferID,
        TailByte,
    };

    use frame_assembler::*;
    
    #[test]
    fn parse_from_can_frames_simple() {

        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: u32,
            health: u2,
            mode: u3,
            sub_mode: u3,
            vendor_specific_status_code: u16,
        }

        impl Message for NodeStatus {
            const TYPE_ID: u16 = 341;
        }
        
        let can_frame = CanFrame{id: NodeStatus::id(0, NodeID::new(32)), dlc: 8, data: [1, 0, 0, 0, 0b10011100, 5, 0, TailByte::new(true, true, false, TransferID::new(0)).into()]};
        
        let mut message_builder = FrameAssembler::new();
        message_builder.add_transfer_frame(can_frame).unwrap();
        let parsed_message: Frame<NodeStatus> = message_builder.build().unwrap();
        
        assert_eq!(parsed_message.body.uptime_sec, 1);
        assert_eq!(parsed_message.body.health, u2::new(2));
        assert_eq!(parsed_message.body.mode, u3::new(3));
        assert_eq!(parsed_message.body.sub_mode, u3::new(4));
        assert_eq!(parsed_message.body.vendor_specific_status_code, 5);
        assert_eq!(parsed_message.id, NodeStatus::id(0, NodeID::new(32)));
                                              
    }
    
    #[test]
    fn deserialize_multi_frame() {
        
        #[derive(Debug, PartialEq, UavcanStruct)]
        struct LogLevel {
            value: u3,
        }
        
        #[derive(Debug, PartialEq, UavcanStruct)]
        struct LogMessage {
            level: LogLevel,
            source: DynamicArray31<u8>,
            text: DynamicArray90<u8>,
        }

        impl Message for LogMessage {
            const TYPE_ID: u16 = 16383;
        }
         
        let uavcan_frame = Frame::from_message(LogMessage{
            level: LogLevel{value: u3::new(0)},
            source: DynamicArray31::with_str("test source"),
            text: DynamicArray90::with_str("test text"),
        }, 0, NodeID::new(32));

        let crc = 0;
        let mut message_builder = FrameAssembler::new();
        
        message_builder.add_transfer_frame(CanFrame{
            id: LogMessage::id(0, NodeID::new(32)),
            dlc: 8,
            data: [crc.get_bits(0..8) as u8, crc.get_bits(8..16) as u8, 0u8.set_bits(5..8, 0).set_bits(0..5, 11).get_bits(0..8), b't', b'e', b's', b't', TailByte::new(true, false, false, TransferID::new(0)).into()],
        }).unwrap();
        
        message_builder.add_transfer_frame(CanFrame{
            id: LogMessage::id(0, NodeID::new(32)),
            dlc: 8,
            data: [b' ', b's', b'o', b'u', b'r', b'c', b'e', TailByte::new(false, false, false, TransferID::new(0)).into()],
        }).unwrap();
        
        message_builder.add_transfer_frame(CanFrame{
            id: LogMessage::id(0, NodeID::new(32)),
            dlc: 8,
            data: [b't', b'e', b's', b't', b' ', b't', b'e', TailByte::new(false, false, false, TransferID::new(0)).into()],
        }).unwrap();
        
        message_builder.add_transfer_frame(CanFrame{
            id: LogMessage::id(0, NodeID::new(32)),
            dlc: 3,
            data: [b'x', b't', TailByte::new(false, true, true, TransferID::new(0)).into(), 0, 0, 0, 0, 0],
        }).unwrap();

        assert_eq!(uavcan_frame.body.source.length().current_length, 11);
        assert_eq!(uavcan_frame, message_builder.build().unwrap());
        
    }
   


}

