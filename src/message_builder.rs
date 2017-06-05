use bit::BitIndex;

use {
    UavcanTransmitable,
    TransportFrame,
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
}


pub struct MessageBuilder<T: UavcanTransmitable> {
    parser: Parser<T>,
    started: bool,
    crc: u16,
    crc_calculated: u16,
    toggle: bool,
    transfer_id: u8,    
}

impl<T:UavcanTransmitable + Default> MessageBuilder<T> {
    fn new() -> Self {
        MessageBuilder{
            parser: Parser::new(),
            started: false,
            crc: 0x00,
            crc_calculated: 0xffff,
            toggle: false,
            transfer_id: 0x00,
        }
    }
    
    fn add_frame<F: TransportFrame>(mut self, frame: F) -> Result<Self, BuilderError> {
        if !self.started {
            if !frame.is_start_frame() {
                return Err(BuilderError::FirstFrameNotStartFrame);
            }
            if frame.get_tail_byte().toggle {
                return Err(BuilderError::ToggleError);
            }
            self.toggle = false;
            self.crc.set_bit_range(0..8, frame.get_data()[0] as u16)
                .set_bit_range(8..16, frame.get_data()[1] as u16); 
            self.transfer_id = frame.get_tail_byte().transfer_id;
            self.started = true;
        }

        let payload = if frame.is_start_frame() && !frame.is_end_frame() {
            &frame.get_data()[2..frame.get_data().len()-1]
        } else {
            &frame.get_data()[0..frame.get_data().len()-1]
        };

        self.parser = match self.parser.parse(payload) {
            Ok(x) => x,
            Err(ParseError::StructureExhausted) => return Err(BuilderError::FormatError),
        };
            

        return Ok(self);
    }
            
                
}


#[cfg(test)]
mod tests {

    use{
        UavcanIndexable,
        UavcanPrimitiveField,
        UavcanPrimitiveType,
    };

    use types::{
        Uint2,
        Uint3,
        Uint4,
        Uint5,
        Uint16,
        Uint32,
    };
    
    use can_frame::{
        CanFrame,
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

    }

}

