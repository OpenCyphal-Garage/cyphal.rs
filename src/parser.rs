use bit_field::{
    BitField,
    BitArray,
};

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
    buffer: ParserQueue,
}

struct ParserQueue {
    buffer: [u8; 15],
    buffer_end_bit: usize,
}

impl ParserQueue {
    fn new() -> Self { ParserQueue{buffer: [0;15], buffer_end_bit: 0} }
        
    fn bit_length(&self) -> usize { self.buffer_end_bit }
    
    fn pop_bits(&mut self, bit_length: usize) -> u64 {
        assert!(bit_length <= 64);
        assert!(bit_length <= self.buffer_end_bit);
        
        let mut bits = 0u64;
        let mut current_bit: usize = 0;
        while current_bit < bit_length {
            if current_bit + 8 < bit_length {
                bits.set_bits(current_bit as u8..current_bit as u8 + 8, self.buffer.get_bits(current_bit..current_bit+8) as u64);
                current_bit = current_bit + 8;
            } else {
                bits.set_bits(current_bit as u8..bit_length as u8, self.buffer.get_bits(current_bit..bit_length) as u64);
                current_bit = bit_length;
            }
        }

        current_bit = 0;
        while current_bit < self.buffer_end_bit-bit_length {
            if current_bit + 8 < self.buffer_end_bit-bit_length {
                let bitmap = self.buffer.get_bits(current_bit+bit_length..current_bit+bit_length+8);
                self.buffer.set_bits(current_bit..current_bit+8, bitmap);
                current_bit = current_bit + 8;
            } else {
                let bitmap = self.buffer.get_bits(current_bit+bit_length..self.buffer_end_bit);
                self.buffer.set_bits(current_bit..self.buffer_end_bit-bit_length, bitmap);
                current_bit = self.buffer_end_bit-bit_length;
            }
        }
        
        self.buffer_end_bit -= bit_length;
        return bits;
    }
    
    fn push(&mut self, tail: &[u8]) {
        for byte in tail {
            self.buffer.set_bits(self.buffer_end_bit..self.buffer_end_bit+8, byte.clone());
            self.buffer_end_bit += 8;
        }
    }
    
}


impl<T: UavcanIndexable + Default> Parser<T> {
    pub fn new() -> Parser<T> {
        Parser{structure: T::default(), current_field_index: 0, current_type_index: 0, buffer: ParserQueue::new()}
    }

    pub fn parse(mut self, input: &[u8]) -> Result<Parser<T>, ParseError> {
                
        for chunk in input.chunks(8) {
            self.buffer.push(chunk);

            loop {
                
                if self.current_field_index < self.structure.number_of_primitive_fields() {
                    if self.current_type_index < self.structure.primitive_field(self.current_field_index).get_size() {
                        
                        let field_length = self.structure.primitive_field(self.current_field_index).primitive_type(self.current_type_index).bit_length();
                        if field_length <= self.buffer.bit_length() {
                            self.structure.primitive_field_as_mut(self.current_field_index).primitive_type_as_mut(self.current_type_index).set_bits(0..field_length, self.buffer.pop_bits(field_length));
                            self.current_type_index += 1;
                        } else {
                            break;
                        }
                    } else {
                        self.current_type_index = 0;
                        self.current_field_index += 1;
                    }
                } else {
                    if self.buffer.bit_length() >= 8 {
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
    };

    use parser::{
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

