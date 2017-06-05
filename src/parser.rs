use bit::BitIndex;

use {
    UavcanIndexable,
};

#[derive(Debug)]
pub enum ParseError {
    StructureExhausted,
}

pub struct Parser<T: UavcanIndexable> {
    structure: T,
    current_field_index: usize,
    current_type_index: usize,
    buffer_end_bit: usize,
    buffer: [u8; 15],
}

impl<T: UavcanIndexable + Default> Parser<T> {
    pub fn new() -> Parser<T> {
        Parser{structure: T::default(), current_field_index: 0, current_type_index: 0, buffer: [0; 15], buffer_end_bit: 0}
    }

    fn buffer_consume_bits(&mut self, number_of_bits: usize) {
        if number_of_bits > self.buffer_end_bit { panic!("Offset can't be larger than buffer_end_bit");}
        let new_buffer_len = self.buffer_end_bit - number_of_bits;
        let offset_byte = number_of_bits/8;
        let offset_bit = number_of_bits%8;
        for i in 0..((new_buffer_len+7)/8) {
            let bits_remaining = new_buffer_len - i*8;
            if offset_bit == 0 {
                self.buffer[i] = self.buffer[offset_byte+i];
            } else if bits_remaining + offset_bit < 8 {
                let bitmask = self.buffer[offset_byte+i].bit_range(offset_bit..8);
                self.buffer[i]
                    .set_bit_range(0..8-offset_bit, bitmask);
            } else {
                let lsb = self.buffer[offset_byte+i].bit_range(offset_bit..8);
                let msb = self.buffer[offset_byte+i+1].bit_range(0..offset_bit);
                
                self.buffer[i]
                    .set_bit_range(0..8-offset_bit, lsb)
                    .set_bit_range(8-offset_bit..8, msb);
            }
        }
        self.buffer_end_bit -= number_of_bits;
    }

    fn buffer_append(&mut self, tail: &[u8]) {
        let joint_byte = self.buffer_end_bit/8;
        let joint_bit = self.buffer_end_bit%8;

        for i in 0..tail.len() {
            if joint_bit == 0 {
                self.buffer[joint_byte + i] = tail[i];
            } else {
                self.buffer[joint_byte+i] = self.buffer[joint_byte + i].bit_range(0..joint_bit) | (tail[i].bit_range(0..8-joint_bit) << joint_bit);
                self.buffer[joint_byte+i+1] = tail[i].bit_range(8-joint_bit..8) >> 8-joint_bit;
            }
        }

        self.buffer_end_bit += tail.len()*8;
    }
    
    pub fn parse(mut self, input: &[u8]) -> Result<Parser<T>, ParseError> {
                
        for chunk in input.chunks(8) {
            self.buffer_append(chunk);

            loop {
                
                if self.structure.primitive_field(self.current_field_index).is_some() {
                    if self.structure.primitive_field(self.current_field_index).unwrap().primitive_type(self.current_type_index).is_some() {
                        
                        let field_length = self.structure.primitive_field(self.current_field_index).unwrap().primitive_type(self.current_type_index).unwrap().bitlength();
                        if field_length <= self.buffer_end_bit {
                            self.structure.primitive_field_as_mut(self.current_field_index).unwrap().primitive_type_as_mut(self.current_type_index).unwrap().set_from_bytes(&self.buffer[0..( (field_length+7)/8 )]);
                            self.buffer_consume_bits(field_length);
                            self.current_type_index += 1;
                        } else {
                            break;
                        }
                    } else {
                        self.current_type_index = 0;
                        self.current_field_index += 1;
                    }
                } else {
                    if self.buffer_end_bit >= 8 {
                        return Err(ParseError::StructureExhausted);
                    } else {
                        return Ok(self);
                    }
                }

            }

        }
        return Ok(self);
    }

    pub fn to_structure(self) -> T {
        self.structure
    }
}


#[cfg(test)]
mod tests {

    use {
        UavcanIndexable,
        UavcanPrimitiveField,
        UavcanPrimitiveType,
    };

    use parser::{
        ParseError,
        Parser,
    };
    
    use types::{
        Uint2,
        Uint3,
        Uint8,
        Uint16,
        Uint32,
    };
    
    #[test]
    fn uavcan_parse_test_byte_aligned() {

        #[derive(UavcanIndexable, Default)]
        struct Message {
            v1: Uint8,
            v2: Uint32,
            v3: Uint16,
            v4: Uint8,
        }

        let mut parser: Parser<Message> = Parser::new();

        parser = parser.parse(&[17, 19, 0, 0, 0, 21, 0, 23]).unwrap();

        let parsed_message = parser.to_structure();

        
        assert_eq!(parsed_message.v1, 17.into());
        assert_eq!(parsed_message.v2, 19.into());
        assert_eq!(parsed_message.v3, 21.into());
        assert_eq!(parsed_message.v4, 23.into());
    }




    #[test]
    fn uavcan_parse_test_misaligned() {
        
        
        #[derive(UavcanIndexable, Default)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        
        let mut parser: Parser<NodeStatus> = Parser::new();

        parser = parser.parse(&[1, 0, 0, 0, 0b10001110, 5, 0]).unwrap();

        let parsed_message = parser.to_structure();
        

        assert_eq!(parsed_message.uptime_sec, 1.into());
        assert_eq!(parsed_message.health, 2.into());
        assert_eq!(parsed_message.mode, 3.into());
        assert_eq!(parsed_message.sub_mode, 4.into());
        assert_eq!(parsed_message.vendor_specific_status_code, 5.into());
        
    }
}

