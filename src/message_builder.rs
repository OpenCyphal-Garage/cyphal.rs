use bit_field::BitField;

use {
    UavcanFrame,
    UavcanIndexable,
    TransportFrame,
    UavcanHeader,
};

use parser::{
    ParseError,
    Parser,
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
}


pub struct MessageBuilder<B: UavcanIndexable> {
    parser: Parser<B>,
    started: bool,
    id: u32,
    crc: u16,
    crc_calculated: u16,
    toggle: bool,
    transfer_id: u8,    
}

impl<B: UavcanIndexable> MessageBuilder<B> {
    pub fn new() -> Self {
        MessageBuilder{
            parser: Parser::new(),
            started: false,
            id: 0x00,
            crc: 0x00,
            crc_calculated: 0xffff,
            toggle: false,
            transfer_id: 0x00,
        }
    }
    
    pub fn add_frame<F: TransportFrame>(mut self, frame: F) -> Result<Self, BuilderError> {
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

        self.parser = match self.parser.parse(payload) {
            Ok(x) => x,
            Err(ParseError::StructureExhausted) => return Err(BuilderError::FormatError),
        };
            

        return Ok(self);
    }

    pub fn build<H: UavcanHeader, F: UavcanFrame<H, B>>(self) -> Result<F, BuilderError> {
        if let Ok(id) = H::from_id(self.id) {
            Ok(F::from_parts(id, self.parser.to_structure()))
        } else {
            Err(BuilderError::IdError)
        }
    }
                
}


#[cfg(test)]
mod tests {

    use bit_field::BitField;
    
    use{
        UavcanIndexable,
        UavcanPrimitiveField,
        UavcanHeader,
        MessageFrameHeader,
        UavcanFrame,
        TailByte,
    };
    
    use types::{
        Uint2,
        Uint3,
        Uint16,
        Uint32,
    };
    
    use tests::{
        CanFrame,
        CanID,
    };

    use message_builder::{
        MessageBuilder,
    };
    
    #[test]
    fn parse_from_can_frames_simple() {

        #[derive(UavcanIndexable)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        message_frame_header!(NodeStatusHeader, 341);

        #[derive(UavcanFrame)]
        struct NodeStatusMessage {
            header: NodeStatusHeader,
            body: NodeStatus,
        }
            
        
        let can_frame = CanFrame{id: CanID::Extended(NodeStatusHeader::new(0, 32).to_id()), dlc: 8, data: [1, 0, 0, 0, 0b10001110, 5, 0, TailByte{start_of_transfer: true, end_of_transfer: true, toggle: false, transfer_id: 0}.into()]};
        
        let mut message_builder = MessageBuilder::new();
        message_builder = message_builder.add_frame(can_frame).unwrap();
        let parsed_message: NodeStatusMessage = message_builder.build().unwrap();
        
        assert_eq!(parsed_message.body.uptime_sec, 1.into());
        assert_eq!(parsed_message.body.health, 2.into());
        assert_eq!(parsed_message.body.mode, 3.into());
        assert_eq!(parsed_message.body.sub_mode, 4.into());
        assert_eq!(parsed_message.body.vendor_specific_status_code, 5.into());
        assert_eq!(parsed_message.header, NodeStatusHeader::new(0, 32));
                                              
    }

}

