use bit_field::BitField;

use transfer::{
    TransferFrame,
    TransferID,
};

use {
    Frame,
    Header,
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

pub struct FrameAssembler<F: Frame> {
    deserializer: Deserializer<F::Body>,
    started: bool,
    id: u32,
    crc: u16,
    crc_calculated: u16,
    toggle: bool,
    transfer_id: TransferID,    
}

impl<F: Frame> FrameAssembler<F> {
    pub fn new() -> Self {
        Self{
            deserializer: Deserializer::new(),
            started: false,
            id: 0x00,
            crc: 0x00,
            crc_calculated: 0xffff,
            toggle: false,
            transfer_id: 0x00.into(),
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

    pub fn build(self) -> Result<F, BuildError> {
        let header = if let Ok(id) = F::Header::from_id(self.id) {
            id
        } else {
            return Err(BuildError::IdError)
        };

        let body = if let Ok(body) = self.deserializer.into_structure() {
            body
        } else {
            return Err(BuildError::NotFinishedParsing)
        };

        Ok(F::from_parts(header, body))
    }
                
}


#[cfg(test)]
mod tests {

    use bit_field::BitField;

    use uavcan;
    
    use tests::{
        CanFrame,
        CanID,
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
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        message_frame_header!(NodeStatusHeader, 341);

        uavcan_frame!(NodeStatusMessage, NodeStatusHeader, NodeStatus, 0);
            
        
        let can_frame = CanFrame{id: CanID(NodeStatusHeader::new(0, 32).id()), dlc: 8, data: [1, 0, 0, 0, 0b10011100, 5, 0, TailByte::new(true, true, false, TransferID::from(0)).into()]};
        
        let mut message_builder = FrameAssembler::new();
        message_builder.add_transfer_frame(can_frame).unwrap();
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
        let mut message_builder = FrameAssembler::new();
        
        message_builder.add_transfer_frame(CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 8,
            data: [crc.get_bits(0..8) as u8, crc.get_bits(8..16) as u8, 0u8.set_bits(5..8, 0).set_bits(0..5, 11).get_bits(0..8), b't', b'e', b's', b't', TailByte::new(true, false, false, TransferID::from(0)).into()],
        }).unwrap();
        
        message_builder.add_transfer_frame(CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 8,
            data: [b' ', b's', b'o', b'u', b'r', b'c', b'e', TailByte::new(false, false, false, TransferID::from(0)).into()],
        }).unwrap();
        
        message_builder.add_transfer_frame(CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 8,
            data: [b't', b'e', b's', b't', b' ', b't', b'e', TailByte::new(false, false, false, TransferID::from(0)).into()],
        }).unwrap();
        
        message_builder.add_transfer_frame(CanFrame{
            id: CanID(LogMessageHeader::new(0, 32).id()),
            dlc: 3,
            data: [b'x', b't', TailByte::new(false, true, true, TransferID::from(0)).into(), 0, 0, 0, 0, 0],
        }).unwrap();

        assert_eq!(uavcan_frame.body.source.length().current_length, 11);
        assert_eq!(uavcan_frame, message_builder.build().unwrap());
        
    }
   


}

