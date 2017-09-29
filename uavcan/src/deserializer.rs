use lib::core::mem;

pub use serializer::SerializationBuffer as DeserializationBuffer;

use {
    Struct,
};

#[derive(Debug, PartialEq)]
pub enum DeserializationResult {
    Finished,
    BufferInsufficient,
}



pub struct Deserializer<T: Struct> {
    structure: T,
    field_index: usize,
    bit_index: usize,
}

impl<T: Struct> Deserializer<T> {
    pub fn new() -> Deserializer<T> {
        let structure: T;
        unsafe {
            structure = mem::zeroed();
        };            
        Deserializer{structure: structure, field_index: 0, bit_index: 0}
    }

    pub fn deserialize(&mut self, input: &mut [u8]) -> DeserializationResult {
        let mut buffer = DeserializationBuffer::with_full_buffer(input);
        self.structure.deserialize(&mut self.field_index, &mut self.bit_index, &mut buffer)
    }

    pub fn into_structure(self) -> Result<T, ()> {
        let number_of_fields = T::FLATTENED_FIELDS_NUMBER;

        let finished = if number_of_fields == self.field_index {
            true
        } else if number_of_fields - 1 == self.field_index && T::TAIL_ARRAY_OPTIMIZABLE{
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
    
    use *;
    use deserializer::*;
    use types::*;
    
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

        deserializer.deserialize(&mut [1, 0, 0, 0, 0b10011100, 5, 0]);

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

        deserializer.deserialize(&mut [0u8.set_bits(0..3, 4).get_bits(0..8), b't', b'e', b's', b't', b'l', b'o', b'l']);
        
        let parsed_message = deserializer.into_structure().unwrap();

        assert_eq!(parsed_message.text1.length().current_length, 4);
        assert_eq!(parsed_message.text1, DynamicArray7::with_str("test"));
        assert_eq!(parsed_message.text2.length().current_length, 3);
        assert_eq!(parsed_message.text2, DynamicArray8::with_str("lol"));
    }
}
