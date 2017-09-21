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

#[derive(Debug, PartialEq)]
pub enum DeserializationResult {
    Finished,
    BufferInsufficient,
}



pub struct DeserializationBuffer<'a> {
    buffer: &'a mut [u8],
    buffer_end_bit: usize,
}

impl<'a> DeserializationBuffer<'a> {
    pub fn with_buffer(buffer: &'a mut [u8]) -> Self {
        let buffer_len = buffer.len();
        DeserializationBuffer{buffer: buffer, buffer_end_bit: buffer_len*8}
    }
        
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
}

impl<T: UavcanStruct> Deserializer<T> {
    pub fn new() -> Deserializer<T> {
        let structure: T;
        unsafe {
            structure = mem::zeroed();
        };            
        Deserializer{structure: structure, field_index: 0, bit_index: 0}
    }

    pub fn deserialize(&mut self, input: &mut [u8]) -> DeserializationResult {
        let mut buffer = DeserializationBuffer::with_buffer(input);
        self.structure.deserialize(&mut self.field_index, &mut self.bit_index, &mut buffer)
    }

    pub fn into_structure(mut self) -> Result<T, ()> {
        let number_of_fields = self.structure.flattened_fields_len();

        let finished = if number_of_fields == self.field_index {
            true
        } else if number_of_fields - 1 == self.field_index && self.structure.tail_array_optimizable(){
            true
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

        deserializer.deserialize(&mut [17, 19, 0, 0, 0, 21, 0, 23]);

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

        deserializer.deserialize(&mut [1, 0, 0, 0, 0b10001110, 5, 0]);

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

        deserializer.deserialize(&mut [0u8.set_bits(5..8, 4).get_bits(0..8), b't', b'e', b's', b't', b'l', b'o', b'l']);
        
        let parsed_message = deserializer.into_structure().unwrap();

        assert_eq!(parsed_message.text1.length().current_length, 4);
        assert_eq!(parsed_message.text1, DynamicArray7::with_str("test"));
        assert_eq!(parsed_message.text2.length().current_length, 3);
        assert_eq!(parsed_message.text2, DynamicArray8::with_str("lol"));
    }
}
