use crc::TransferCRC;

use header::Header;

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

use framing::{
    Deframer,
    DeframingResult,
    DeframingError,
};

pub(crate) struct Version0Deframer<S: Struct> {
    deserializer: Deserializer<S>,
    started: bool,
    finished: bool,
    id: TransferFrameID,
    crc_received: Option<TransferCRC>,
    crc_calculated: TransferCRC,
    toggle: bool,
    transfer_id: TransferID,    
}

impl<S: Struct> Deframer<S> for Version0Deframer<S> {
    fn new() -> Self {
        Self{
            deserializer: Deserializer::new(true),
            started: false,
            finished: false,
            id: TransferFrameID::new(0x00),
            crc_received: None,
            crc_calculated: TransferCRC::from_signature(S::DATA_TYPE_SIGNATURE),
            toggle: false,
            transfer_id: TransferID::new(0x00),
        }
    }
    
    fn add_frame<T: TransferFrame>(&mut self, mut frame: T) -> Result<DeframingResult<::Frame<S>>, DeframingError> {
        let end_frame = frame.is_end_frame();
        
        if self.finished {
            return Err(DeframingError::FrameAfterEndFrame);
        }
        
        if !self.started {
            if !frame.is_start_frame() {
                return Err(DeframingError::FirstFrameNotStartFrame);
            }
            
            if frame.tail_byte().toggle() {
                return Err(DeframingError::ToggleError);
            }
            
            if !end_frame {
                self.crc_received = Some(TransferCRC::from((frame.data()[0] as u16) | (frame.data()[1] as u16) << 8));
            }
            
            self.toggle = false;
            self.transfer_id = frame.tail_byte().transfer_id();
            self.id = frame.id();
            self.started = true;
        }

        if self.id != frame.id() {
            return Err(DeframingError::IDError);
        }

        let data_len = frame.data().len();
        let payload = if frame.is_start_frame() && !frame.is_end_frame() {
            &mut frame.data_as_mut()[2..data_len-1]
        } else {
            &mut frame.data_as_mut()[0..data_len-1]
        };

        self.crc_calculated.add(payload);
        self.deserializer.deserialize(payload);            

        if end_frame {
            if self.crc_calculated != self.crc_received.unwrap_or(self.crc_calculated) {
                Result::Err(DeframingError::CRCError)
            } else {
                let deserializer = ::lib::core::mem::replace(&mut self.deserializer, Deserializer::new(true));
                self.started = false;
                self.finished = false;
                self.crc_received = None;
                self.crc_calculated = TransferCRC::from_signature(S::DATA_TYPE_SIGNATURE);
                self.toggle = false;
                Ok(DeframingResult::Finished(Frame::from_parts(Header::from(self.id), deserializer.into_structure().unwrap())))
            }
        } else {
            Ok(DeframingResult::Ok)
        }
    }
}


#[cfg(test)]
mod tests {

    use bit_field::BitField;

    use tests::{
        CanFrame,
    };
    
    use *;    
    use types::*;

    use transfer::{
        TransferID,
        TailByte,
    };

    use super::*;
    
    #[test]
    fn parse_from_can_frames_simple() {

        #[derive(Debug, PartialEq, Clone, UavcanStruct, Default)]
        struct NodeStatus {
            uptime_sec: u32,
            health: u2,
            mode: u3,
            sub_mode: u3,
            vendor_specific_status_code: u16,
        }

        impl Message for NodeStatus {
            const TYPE_ID: Option<u16> = Some(341);
        }
        
        let can_frame = CanFrame{id: TransferFrameID::new(0), dlc: 8, data: [1, 0, 0, 0, 0b10011100, 5, 0, TailByte::new(true, true, false, TransferID::new(0)).into()]};
        
        let mut deframer: Version0Deframer<NodeStatus> = Version0Deframer::new();
        match deframer.add_frame(can_frame).unwrap() {
            DeframingResult::Finished(parsed_message) => {
                assert_eq!(parsed_message.body.uptime_sec, 1);
                assert_eq!(parsed_message.body.health, u2::new(2));
                assert_eq!(parsed_message.body.mode, u3::new(3));
                assert_eq!(parsed_message.body.sub_mode, u3::new(4));
                assert_eq!(parsed_message.body.vendor_specific_status_code, 5);
                assert_eq!(TransferFrameID::from(parsed_message.header), TransferFrameID::new(0));
            },
            _ => unreachable!("Node status is a single frame transfer"),
        }
                                              
    }
    
    #[test]
    fn deserialize_multi_frame() {
        
        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        #[DSDLSignature = "0x711bf141af572346"]
        #[DataTypeSignature = "0x711bf141af572346"]
        struct LogLevel {
            value: u3,
        }

        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        #[DataTypeSignature = "0xd654a48e0c049d75"]
        #[DSDLSignature = "0xe9862b78d38762ba"]
        struct LogMessage {
            level: LogLevel,
            source: Dynamic<[u8; 31]>,
            text: Dynamic<[u8; 90]>,
        }

        impl Message for LogMessage {
            const TYPE_ID: Option<u16> = Some(16383);
        }
         
        let uavcan_frame = Frame::from_message(LogMessage{
            level: LogLevel{value: u3::new(0)},
            source: Dynamic::<[u8; 31]>::with_data("test source".as_bytes()),
            text: Dynamic::<[u8; 90]>::with_data("test text".as_bytes()),
        }, 0, ProtocolVersion::Version0, NodeID::new(32));

        let crc = 0x6383;
        let mut deframer = Version0Deframer::new();
        
        assert_eq!(deframer.add_frame(CanFrame {
            id: TransferFrameID::new(4194080),
            dlc: 8,
            data: [crc.get_bits(0..8) as u8, crc.get_bits(8..16) as u8, 0u8.set_bits(5..8, 0).set_bits(0..5, 11).get_bits(0..8), b't', b'e', b's', b't', TailByte::new(true, false, false, TransferID::new(0)).into()],
        }),
        Ok(DeframingResult::Ok)
        );

        assert_eq!(deframer.add_frame(CanFrame {
            id: TransferFrameID::new(4194080),
            dlc: 8,
            data: [b' ', b's', b'o', b'u', b'r', b'c', b'e', TailByte::new(false, false, false, TransferID::new(0)).into()],
        }),
        Ok(DeframingResult::Ok)
        );
        
        assert_eq!(deframer.add_frame(CanFrame {
            id: TransferFrameID::new(4194080),
            dlc: 8,
            data: [b't', b'e', b's', b't', b' ', b't', b'e', TailByte::new(false, false, false, TransferID::new(0)).into()],
        }),
        Ok(DeframingResult::Ok)
        );
        
        assert_eq!(deframer.add_frame(CanFrame {
            id: TransferFrameID::new(4194080),
            dlc: 3,
            data: [b'x', b't', TailByte::new(false, true, true, TransferID::new(0)).into(), 0, 0, 0, 0, 0],
        }),
        Ok(DeframingResult::Finished(uavcan_frame))
        );
    }

}

