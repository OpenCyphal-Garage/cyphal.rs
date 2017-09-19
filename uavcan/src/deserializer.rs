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
    Finished(usize),
    BufferInsufficient(usize),
}

pub trait Deserialize {
    fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;
}

impl Deserialize for DynamicArrayLength {
    fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
        if buffer.bit_length() + start_bit < self.bit_length {
            DeserializationResult::BufferInsufficient(0)
        } else {
            self.current_length.set_bits(start_bit as u8..self.bit_length as u8, buffer.pop_bits(self.bit_length-start_bit) as usize);
            DeserializationResult::Finished(self.bit_length)
        }
    }
}

macro_rules! impl_deserialize_for_primitive_type {
    ($type:ident) => {
        impl Deserialize for $type {
            fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                if buffer.bit_length() + start_bit < Self::bit_length() {
                    DeserializationResult::BufferInsufficient(0)
                } else {
                    self.set_bits(start_bit..Self::bit_length(), buffer.pop_bits(Self::bit_length()-start_bit));
                    DeserializationResult::Finished(Self::bit_length())
                }
            }
        }
        
        
    };
}

impl_deserialize_for_primitive_type!(Uint2);
impl_deserialize_for_primitive_type!(Uint3);
impl_deserialize_for_primitive_type!(Uint4);
impl_deserialize_for_primitive_type!(Uint5);
    
impl_deserialize_for_primitive_type!(Uint7);
impl_deserialize_for_primitive_type!(Uint8);

impl_deserialize_for_primitive_type!(Uint16);

impl_deserialize_for_primitive_type!(Uint32);

impl_deserialize_for_primitive_type!(Float16);


macro_rules! impl_deserialize_for_dynamic_array {
    ($type:ident) => {
        impl<T: UavcanPrimitiveType + Deserialize> Deserialize for $type<T> {
            fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                let mut bits_deserialized: usize = 0;
                
                // deserialize length
                if start_bit < self.length().bit_length {
                    let mut length = self.length();
                    match length.deserialize(start_bit, buffer) {
                        DeserializationResult::Finished(bits) => {self.set_length(length.current_length); bits_deserialized += bits}, // ugly hack, fix when dispatching is mostly done static
                        DeserializationResult::BufferInsufficient(bits) => return DeserializationResult::BufferInsufficient(bits_deserialized + bits),
                    }
                }
                
                let mut start_element = (start_bit + bits_deserialized - Self::length_bit_length()) / Self::element_bit_length();
                let start_element_bit = (start_bit + bits_deserialized - Self::length_bit_length()) % Self::element_bit_length();

                // first get rid of the odd bits
                if start_element_bit != 0 {
                    match self[start_element].deserialize(start_element_bit, buffer) {
                        DeserializationResult::Finished(bits) => bits_deserialized += bits,
                        DeserializationResult::BufferInsufficient(bits) => return DeserializationResult::BufferInsufficient(bits_deserialized + bits),
                    }
                    start_element += 1;
                }

                for i in start_element..self.length().current_length {
                    match self[i].deserialize(0, buffer) {
                        DeserializationResult::Finished(bits) => bits_deserialized += bits,
                        DeserializationResult::BufferInsufficient(bits) => return DeserializationResult::BufferInsufficient(bits_deserialized + bits),                        
                    }
                }

                DeserializationResult::Finished(bits_deserialized)
            }
            
        
        }
        
    };
}

impl_deserialize_for_dynamic_array!(DynamicArray3);
impl_deserialize_for_dynamic_array!(DynamicArray4);
impl_deserialize_for_dynamic_array!(DynamicArray5);
impl_deserialize_for_dynamic_array!(DynamicArray6);
impl_deserialize_for_dynamic_array!(DynamicArray7);
impl_deserialize_for_dynamic_array!(DynamicArray8);
impl_deserialize_for_dynamic_array!(DynamicArray9);
impl_deserialize_for_dynamic_array!(DynamicArray10);
impl_deserialize_for_dynamic_array!(DynamicArray11);
impl_deserialize_for_dynamic_array!(DynamicArray12);
impl_deserialize_for_dynamic_array!(DynamicArray13);
impl_deserialize_for_dynamic_array!(DynamicArray14);
impl_deserialize_for_dynamic_array!(DynamicArray15);
impl_deserialize_for_dynamic_array!(DynamicArray16);

impl_deserialize_for_dynamic_array!(DynamicArray31);
impl_deserialize_for_dynamic_array!(DynamicArray32);

impl_deserialize_for_dynamic_array!(DynamicArray90);




pub struct DeserializationBuffer {
    buffer: [u8; 15],
    buffer_end_bit: usize,
}

impl DeserializationBuffer {
    fn new() -> Self { DeserializationBuffer{buffer: [0;15], buffer_end_bit: 0} }
        
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
        let mut bits_deserialized: usize = 0;
        let flattened_fields = self.structure.flattened_fields_len();
        
        for chunk in input.chunks(8) {
            self.buffer.push(chunk);

            loop {
                match self.structure.flattened_field_as_mut(self.field_index) {
                    MutUavcanField::PrimitiveType(primitive_type) => {
                        match primitive_type.deserialize(self.bit_index, &mut self.buffer) {
                            DeserializationResult::Finished(bits) => {
                                bits_deserialized += bits;
                                self.field_index += 1;
                                self.bit_index = 0;
                            },
                            DeserializationResult::BufferInsufficient(bits) => {
                                bits_deserialized += bits;
                                self.bit_index += bits;
                                break;
                            },
                        }
                    },
                    MutUavcanField::DynamicArray(array) => {
                        let array_optimization = self.field_index == flattened_fields-1 && array.tail_optimizable();
                        let bit_index = if array_optimization {
                            if self.bit_index == 0 {
                                array.set_length(1)
                            } else {
                                let current_length = array.length().current_length;
                                array.set_length(current_length+1);
                            }
                            self.bit_index + array.length().bit_length
                        } else {
                            self.bit_index
                        };
                        match array.deserialize(bit_index, &mut self.buffer) {
                            DeserializationResult::Finished(bits) => {
                                if array_optimization {
                                    bits_deserialized += bits;
                                    self.bit_index += bits;
                                } else {
                                    bits_deserialized += bits;
                                    self.field_index += 1;
                                    self.bit_index = 0;
                                }
                            },
                            DeserializationResult::BufferInsufficient(bits) => {
                                bits_deserialized += bits;
                                self.bit_index += bits;
                                break;
                            },
                        }
                    },
                    MutUavcanField::UavcanStruct(_x) => unreachable!(),
                }
                
                if self.field_index == self.structure.flattened_fields_len() {
                    return DeserializationResult::Finished(bits_deserialized);
                }
                
            }
            
        }

        DeserializationResult::BufferInsufficient(bits_deserialized)
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

    use bit_field::BitField;
    
    use {
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
        DynamicArray
    };

    use deserializer::{
        Deserializer,
    };
    
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
