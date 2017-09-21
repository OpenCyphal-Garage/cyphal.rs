use lib::core::mem;

use bit_field::{
    BitField,
    BitArray,
};

use types::*;

use {
    UavcanStruct,
    UavcanPrimitiveType,
    DynamicArrayLength,
    DynamicArray,
    MutUavcanField,
};

#[derive(Debug)]
pub enum DeserializationResult {
    Finished,
    BufferInsufficient,
}



pub struct DeserializationBuffer {
    buffer: [u8; 15],
    buffer_end_bit: usize,
}

impl DeserializationBuffer {
    pub fn new() -> Self { DeserializationBuffer{buffer: [0;15], buffer_end_bit: 0} }
        
    pub fn bit_length(&self) -> usize { self.buffer_end_bit }
    
    pub fn pop_bits(&mut self, bit_length: usize) -> u64 {
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
    
    pub fn push(&mut self, tail: &[u8]) {
        for byte in tail {
            self.buffer.set_bits(self.buffer_end_bit..self.buffer_end_bit+8, *byte);
            self.buffer_end_bit += 8;
        }
    }
    
}




pub struct Deserializer<T: UavcanStruct> {
    structure: T,
    field_index: usize,
    bit_index: usize,
    buffer: DeserializationBuffer,
}

impl<T: UavcanStruct> Deserializer<T> {
    pub fn new() -> Deserializer<T> {
        let structure: T;
        unsafe {
            structure = mem::zeroed();
        };            
        Deserializer{structure: structure, field_index: 0, bit_index: 0, buffer: DeserializationBuffer::new()}
    }

    pub fn deserialize(&mut self, input: &[u8]) -> DeserializationResult {
        let flattened_fields = self.structure.flattened_fields_len();
        
        for chunk in input.chunks(8) {
            self.buffer.push(chunk);

            loop {
                match self.structure.flattened_field_as_mut(self.field_index) {
                    MutUavcanField::PrimitiveType(primitive_type) => {
                        match primitive_type.deserialize(&mut self.bit_index, &mut self.buffer) {
                            DeserializationResult::Finished => {
                                self.field_index += 1;
                                self.bit_index = 0;
                            },
                            DeserializationResult::BufferInsufficient => {
                                break;
                            },
                        }
                    },
                    MutUavcanField::DynamicArray(array) => {
                        let array_optimization = self.field_index == flattened_fields-1 && array.tail_optimizable();
                        if array_optimization {
                            if self.bit_index == 0 {
                                self.bit_index += array.length().bit_length;                                    
                                array.set_length(1);
                            } else {
                                let current_length = array.length().current_length;
                                array.set_length(current_length+1);
                            }
                        }
                        match array.deserialize(&mut self.bit_index, &mut self.buffer) {
                            DeserializationResult::Finished => {
                                if !array_optimization {
                                    self.field_index += 1;
                                    self.bit_index = 0;
                                }
                            },
                            DeserializationResult::BufferInsufficient => {
                                break;
                            },
                        }
                    },
                    MutUavcanField::UavcanStruct(_x) => unreachable!(),
                }
                
                if self.field_index == self.structure.flattened_fields_len() {
                    return DeserializationResult::Finished;
                }
                
            }
            
        }

        DeserializationResult::BufferInsufficient
    }

    pub fn into_structure(mut self) -> Result<T, ()> {
        let number_of_fields = self.structure.flattened_fields_len();

        let finished = if number_of_fields == self.field_index {
            true
        } else if let MutUavcanField::DynamicArray(array) = self.structure.flattened_field_as_mut(self.field_index) {
            if array.tail_optimizable() {
                let current_length = array.length().current_length;
                array.set_length(current_length-1);
                true
            } else {
                false
            }
        } else {
            false
        };

        if finished {
            Ok(self.structure)
        } else {
            Err(())
        }   
         
    }
}


#[cfg(test)]
mod tests {

    use uavcan;
    
    use bit_field::BitField;
    
    use {
        SerializationBuffer,
        SerializationResult,
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
        DynamicArray,
        UavcanPrimitiveType,
    };

    use deserializer::{
        Deserializer,
    };
    
    use types::*;

    use serializer::*;
    
    #[test]
    fn uavcan_parse_test_byte_aligned() {

        #[derive(UavcanStruct)]
        struct Message {
            v1: Uint8,
            v2: Uint32,
            v3: Uint16,
            v4: Uint8,
        }

        let mut deserializer: Deserializer<Message> = Deserializer::new();

        deserializer.deserialize(&[17, 19, 0, 0, 0, 21, 0, 23]);

        let parsed_message = deserializer.into_structure().unwrap();

        
        assert_eq!(parsed_message.v1, 17.into());
        assert_eq!(parsed_message.v2, 19.into());
        assert_eq!(parsed_message.v3, 21.into());
        assert_eq!(parsed_message.v4, 23.into());
    }




    #[test]
    fn uavcan_parse_test_misaligned() {
        
        
        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        
        let mut deserializer: Deserializer<NodeStatus> = Deserializer::new();

        deserializer.deserialize(&[1, 0, 0, 0, 0b10001110, 5, 0]);

        let parsed_message = deserializer.into_structure().unwrap();
        

        assert_eq!(parsed_message.uptime_sec, 1.into());
        assert_eq!(parsed_message.health, 2.into());
        assert_eq!(parsed_message.mode, 3.into());
        assert_eq!(parsed_message.sub_mode, 4.into());
        assert_eq!(parsed_message.vendor_specific_status_code, 5.into());
        
    }

    #[test]
    fn deserialize_dynamic_array() {

        #[derive(UavcanStruct)]
        struct TestMessage {
            pad: Uint5,
            text1: DynamicArray7<Uint8>,
            text2: DynamicArray8<Uint8>,
        }
        //test_array: DynamicArray8<Uint8> = DynamicArray8::with_str("test");

        let mut deserializer: Deserializer<TestMessage> = Deserializer::new();

        deserializer.deserialize(&[0u8.set_bits(5..8, 4).get_bits(0..8), b't', b'e', b's', b't', b'l', b'o', b'l']);
        
        let parsed_message = deserializer.into_structure().unwrap();

        assert_eq!(parsed_message.text1.length().current_length, 4);
        assert_eq!(parsed_message.text1, DynamicArray7::with_str("test"));
        assert_eq!(parsed_message.text2.length().current_length, 3);
        assert_eq!(parsed_message.text2, DynamicArray8::with_str("lol"));
    }
}
