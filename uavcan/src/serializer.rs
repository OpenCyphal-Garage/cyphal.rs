use lib::core::ops::Range;

use types::*;

use {
    UavcanStruct,
    UavcanPrimitiveType,
    UavcanField,
    DynamicArray,
    DynamicArrayLength,
};

use crc;

use bit_field::{
    BitField,
};

#[derive(Debug, PartialEq)]
pub enum SerializationResult {
    BufferFull(usize),
    Finished(usize),
}

#[derive(Debug, PartialEq)]
struct SerializationBuffer<'a> {
    data: &'a mut [u8],
    bit_index: usize,
}

pub trait Serialize {
    fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn bits_remaining(&self, start_bit: usize) -> usize;
}

impl Serialize for DynamicArrayLength {
    fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult {
        let mut bits_serialized: usize = 0;
        
        let mut byte_start = buffer.bit_index / 8;
        let odd_bits_start = buffer.bit_index % 8;
        
        // first get rid of the odd bits
        if odd_bits_start != 0 && 8-odd_bits_start <= self.bit_length - start_bit {
            buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.current_length.get_bits((start_bit as u8)..(start_bit+8-odd_bits_start) as u8) as u8);
            bits_serialized += 8-odd_bits_start;
            buffer.bit_index += 8-odd_bits_start;
            byte_start += 1;
        } else if odd_bits_start != 0 && 8-odd_bits_start > self.bit_length - start_bit {
            buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.current_length.get_bits((start_bit as u8)..(start_bit + (self.bit_length - start_bit) ) as u8) as u8);
            bits_serialized += self.bit_length - start_bit;
            buffer.bit_index += self.bit_length - start_bit;
            return SerializationResult::Finished(bits_serialized);
        }
        
        for i in byte_start..buffer.data.len() {
            let serialization_index = bits_serialized + start_bit;
            let remaining_bits = self.bit_length - serialization_index;

            if remaining_bits == 0 {
                return SerializationResult::Finished(bits_serialized);
            } else if remaining_bits <= 8 {
                buffer.data[i] = self.current_length.get_bits((serialization_index as u8)..(serialization_index+remaining_bits) as u8) as u8;
                buffer.bit_index += remaining_bits;
                bits_serialized += remaining_bits;
                return SerializationResult::Finished(bits_serialized);
            } else {
                buffer.data[i] = self.current_length.get_bits((serialization_index as u8)..(serialization_index+8) as u8) as u8;
                buffer.bit_index += 8;
                bits_serialized += 8;
            }
        }
        
        
        SerializationResult::BufferFull(bits_serialized)

    }

    fn bits_remaining(&self, start_bit: usize) -> usize {
        assert!(start_bit < self.bit_length);
        self.bit_length - start_bit
    }           


}

macro_rules! impl_serialize_for_primitive_type {
    ($type:ident) => {
        impl Serialize for $type {
            fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult {
                let mut bits_serialized: usize = 0;
                
                let mut byte_start = buffer.bit_index / 8;
                let odd_bits_start = buffer.bit_index % 8;
                
                // first get rid of the odd bits
                if odd_bits_start != 0 && 8-odd_bits_start <= $type::bit_length() - start_bit {
                    buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.get_bits(start_bit..(start_bit+8-odd_bits_start)) as u8);
                    bits_serialized += 8-odd_bits_start;
                    buffer.bit_index += 8-odd_bits_start;
                    byte_start += 1;
                } else if odd_bits_start != 0 && 8-odd_bits_start > $type::bit_length() - start_bit {
                    buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.get_bits(start_bit..(start_bit + ($type::bit_length() - start_bit) )) as u8);
                    bits_serialized += $type::bit_length() - start_bit;
                    buffer.bit_index += $type::bit_length() - start_bit;
                    return SerializationResult::Finished(bits_serialized);
                }
                
                for i in byte_start..buffer.data.len() {
                    let serialization_index = bits_serialized + start_bit;
                    let remaining_bits = $type::bit_length() - serialization_index;

                    if remaining_bits == 0 {
                        return SerializationResult::Finished(bits_serialized);
                    } else if remaining_bits <= 8 {
                        buffer.data[i] = self.get_bits(serialization_index..serialization_index+remaining_bits) as u8;
                        buffer.bit_index += remaining_bits;
                        bits_serialized += remaining_bits;
                        return SerializationResult::Finished(bits_serialized);
                    } else {
                        buffer.data[i] = self.get_bits(serialization_index..(serialization_index+8)) as u8;
                        buffer.bit_index += 8;
                        bits_serialized += 8;
                    }
                }
                
                
                SerializationResult::BufferFull(bits_serialized)

            }

            fn bits_remaining(&self, start_bit: usize) -> usize {
                assert!(start_bit < Self::bit_length());
                Self::bit_length() - start_bit
            }           

        }
    };
}

macro_rules! impl_serialize_for_dynamic_array {
    ($type:ident) => {
        impl<T: UavcanPrimitiveType + Serialize> Serialize for $type<T> {
            fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult {
                let mut bits_serialized: usize = 0;
                
                let mut start_element = start_bit / Self::element_bit_length();
                let start_element_bit = start_bit % Self::element_bit_length();

                // first get rid of the odd bits
                if start_element_bit != 0 {
                    match self[start_element].serialize(start_element_bit, buffer) {
                        SerializationResult::Finished(bits) => bits_serialized += bits,
                        SerializationResult::BufferFull(bits) => return SerializationResult::BufferFull(bits_serialized + bits),
                    }
                    start_element += 1;
                }

                for i in start_element..self.length() {
                    match self[i].serialize(0, buffer) {
                        SerializationResult::Finished(bits) => bits_serialized += bits,
                        SerializationResult::BufferFull(bits) => return SerializationResult::BufferFull(bits_serialized + bits),                        
                    }
                }

                SerializationResult::Finished(bits_serialized)
            }
            
            fn bits_remaining(&self, start_bit: usize) -> usize {
                assert!(start_bit < T::bit_length() * self.length());
                T::bit_length() * self.length() - start_bit
            }           
        }
        
    };
}


impl_serialize_for_primitive_type!(Uint2);
impl_serialize_for_primitive_type!(Uint3);
impl_serialize_for_primitive_type!(Uint4);
impl_serialize_for_primitive_type!(Uint5);

impl_serialize_for_primitive_type!(Uint7);
impl_serialize_for_primitive_type!(Uint8);

impl_serialize_for_primitive_type!(Uint16);

impl_serialize_for_primitive_type!(Uint32);

impl_serialize_for_primitive_type!(Float16);


impl_serialize_for_dynamic_array!(DynamicArray3);
impl_serialize_for_dynamic_array!(DynamicArray4);
impl_serialize_for_dynamic_array!(DynamicArray5);
impl_serialize_for_dynamic_array!(DynamicArray6);
impl_serialize_for_dynamic_array!(DynamicArray7);
impl_serialize_for_dynamic_array!(DynamicArray8);
impl_serialize_for_dynamic_array!(DynamicArray9);
impl_serialize_for_dynamic_array!(DynamicArray10);
impl_serialize_for_dynamic_array!(DynamicArray11);
impl_serialize_for_dynamic_array!(DynamicArray12);
impl_serialize_for_dynamic_array!(DynamicArray13);
impl_serialize_for_dynamic_array!(DynamicArray14);
impl_serialize_for_dynamic_array!(DynamicArray15);
impl_serialize_for_dynamic_array!(DynamicArray16);

impl_serialize_for_dynamic_array!(DynamicArray31);
impl_serialize_for_dynamic_array!(DynamicArray32);

impl_serialize_for_dynamic_array!(DynamicArray90);

    


pub struct Serializer<T: UavcanStruct> {
    structure: T,
    field_index: usize,
    bit_index: usize,
}


impl<T: UavcanStruct> Serializer<T> {
    pub fn from_structure(structure: T) -> Self {
        Self{
            structure: structure,
            field_index: 0,
            bit_index: 0,
        }
    }

    
    /// serialize(&self, buffer: &mut [u]) -> usize
    ///
    /// serialize into buffer untill one of two occurs
    /// 1. buffer is full
    /// 2. structure is exahusted (all data have been serialized)
    /// When the serialization is finished the return value will 
    /// contain the number of bits that was serialized
    pub fn serialize(&mut self, buffer: &mut [u8]) -> SerializationResult {
        let mut serialization_buffer = SerializationBuffer{data: buffer, bit_index: 0};

        loop {
            match self.structure.flattened_field(self.field_index) {
                UavcanField::PrimitiveType(primitive_type) => {
                    match primitive_type.serialize(self.bit_index, &mut serialization_buffer) {
                        SerializationResult::Finished(_bits) => {
                            self.field_index += 1;
                            self.bit_index = 0;
                        },
                        SerializationResult::BufferFull(bits) => {
                            self.bit_index += bits;
                            return SerializationResult::BufferFull(serialization_buffer.bit_index);
                        },
                    }
                },
                UavcanField::DynamicArray(array) => {
                    match array.serialize(self.bit_index, &mut serialization_buffer) {
                        SerializationResult::Finished(_bits) => {
                            self.field_index += 1;
                            self.bit_index = 0;
                        },
                        SerializationResult::BufferFull(bits) => {
                            self.bit_index += bits;
                            return SerializationResult::BufferFull(serialization_buffer.bit_index);
                        },
                    }
                },
                UavcanField::UavcanStruct(_x) => unreachable!(),
            }

            if self.field_index == self.structure.flattened_fields_len() {
                return SerializationResult::Finished(serialization_buffer.bit_index);
            }
        }
                
        
    }


    pub fn bits_remaining(&self) -> usize {
        let mut bits_counted = 0;
        
        let mut field_index = self.field_index;
        let mut bit_index = self.bit_index;

        loop {
            if field_index == self.structure.flattened_fields_len() {
                return bits_counted;
            }
            
            bits_counted += match self.structure.flattened_field(field_index) {
                UavcanField::PrimitiveType(primitive_type) => {
                    primitive_type.bits_remaining(bit_index)
                },
                UavcanField::DynamicArray(array) => {
                    array.bits_remaining(bit_index)
                },
                UavcanField::UavcanStruct(_struct) => {
                    unreachable!()
                },
            };

            bit_index = 0;
            field_index += 1;

        }
    }

    pub fn crc(&mut self, data_type_signature: u64) -> u16 {
        let mut crc = 0xffff;

        let field_index = self.field_index;
        let bit_index = self.bit_index;
        
        for i in 0..4 {
            crc = crc::add_byte(crc, &(data_type_signature.get_bits(8*i..8*(i+1)) as u8));;
        }

        loop {
            let mut buffer = [0u8; 8];
            if let SerializationResult::Finished(_bits) = self.serialize(&mut buffer) {
                crc = crc::add(crc, &buffer);
                self.field_index = field_index;
                self.bit_index = bit_index;
                return crc;
            } else {
                crc = crc::add(crc, &buffer);
            }
            
        }
        
    }
        



}       
    


#[cfg(test)]
mod tests {

    use {
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
        UavcanPrimitiveType,
    };

    use serializer::*;
    
    use types::*;

    #[test]
    fn uavcan_serialize_primitive_types() {
        let uint2: Uint2 = 1.into();
        let uint8: Uint8 = 128.into();
        let uint16: Uint16 = 257.into();

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        assert_eq!(uint2.serialize(0, &mut buffer), SerializationResult::Finished(2));
        assert_eq!(buffer.data, [1, 0, 0, 0]);

        buffer.bit_index = 0;
        assert_eq!(uint8.serialize(0, &mut buffer), SerializationResult::Finished(8));
        assert_eq!(buffer.data, [128, 0, 0, 0]);
            
        buffer.bit_index = 0;
        assert_eq!(uint16.serialize(0, &mut buffer), SerializationResult::Finished(16));
        assert_eq!(buffer.data, [1, 1, 0, 0]);
            
        uint2.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 0]);
            
        uint8.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 2]);
            

    }

    #[test]
    fn uavcan_serialize_dynamic_array() {
        let a1: DynamicArray4<Uint2> = DynamicArray4::with_data(&[1.into(), 0.into(), 1.into(), 0.into()]);
        let a2: DynamicArray6<Uint2> = DynamicArray6::with_data(&[1.into(), 0.into(), 1.into(), 0.into(), 1.into(), 0.into()]);
        let a3: DynamicArray4<Uint7> = DynamicArray4::with_data(&[1.into(), 2.into(), 4.into(), 8.into()]);

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        assert_eq!(a1.serialize(0, &mut buffer), SerializationResult::Finished(8));
        assert_eq!(buffer.data, [0b00010001, 0, 0, 0]);

        buffer.bit_index = 0;
        a2.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [0b00010001, 0b00000001, 0, 0]);
            
        buffer.bit_index = 0;
        a3.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 1]);            

    }

    #[test]
    fn uavcan_serialize_dynamic_array_without_length() {
        let a: DynamicArray6<Uint7> = DynamicArray6::with_data(&[1.into(), 4.into(), 16.into(), 64.into()]);

        let mut data = [0u8; 1];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        a.serialize(3, &mut buffer);
        assert_eq!(buffer.data, [1]);
        
        buffer.bit_index = 0;
        a.serialize(11, &mut buffer);
        assert_eq!(buffer.data, [2]);

        buffer.bit_index = 0;
        a.serialize(19, &mut buffer);
        assert_eq!(buffer.data, [4]);

        buffer.bit_index = 0;
        a.serialize(27, &mut buffer);
        assert_eq!(buffer.data, [8]);

    }

    #[test]
    fn uavcan_serialize_test_byte_aligned() {

        #[derive(UavcanStruct)]
        struct Message {
            v1: Uint8,
            v2: Uint32,
            v3: Uint16,
            v4: Uint8,
        }


        let message = Message{
            v1: 17.into(),
            v2: 19.into(),
            v3: 21.into(),
            v4: 23.into(),
        };

        let mut serializer: Serializer<Message> = Serializer::from_structure(message);
        let mut array: [u8; 8] = [0; 8];

        serializer.serialize(&mut array);

        assert_eq!(array, [17, 19, 0, 0, 0, 21, 0, 23]);
        
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

        let message = NodeStatus{
            uptime_sec: 1.into(),
            health: 2.into(),
            mode: 3.into(),
            sub_mode: 4.into(),
            vendor_specific_status_code: 5.into(),
        };

        let mut serializer: Serializer<NodeStatus> = Serializer::from_structure(message);
        let mut array: [u8; 7] = [0; 7];

        serializer.serialize(&mut array);

        assert_eq!(array, [1, 0, 0, 0, 0b10001110, 5, 0]);      

        
    }

    
    #[test]
    fn remaining_bits() {

        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        let message = NodeStatus{
            uptime_sec: 1.into(),
            health: 2.into(),
            mode: 3.into(),
            sub_mode: 4.into(),
            vendor_specific_status_code: 5.into(),
        };

        let mut serializer: Serializer<NodeStatus> = Serializer::from_structure(message);

        assert_eq!(serializer.bits_remaining(), 56);

        let mut array: [u8; 7] = [0; 7];
        serializer.serialize(&mut array);

        assert_eq!(serializer.bits_remaining(), 0);
    }
   
}

