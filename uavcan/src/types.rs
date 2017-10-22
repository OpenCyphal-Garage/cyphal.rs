pub use ux::*;
pub use half::f16;

use lib::core::mem;

use lib;
use lib::core::fmt;
use lib::core::cmp;
use lib::core::ops::{
    Index,
    IndexMut,
};

use {
    DynamicArray,
    DynamicArrayLength,
};

use serializer::{
    SerializationResult,
    SerializationBuffer,
};

use deserializer::{
    DeserializationResult,
    DeserializationBuffer,
};


pub trait PrimitiveType {
    const BIT_LENGTH: usize;
    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;
}


macro_rules! dynamic_array_def {
    ($i:ident, $n:expr, $log_bits:expr) => {

        pub struct $i<T: PrimitiveType> {
            current_size: usize,
            data: [T; $n],
        }
        
        impl<T: PrimitiveType + Copy> $i<T> {
            pub fn with_data(data: &[T]) -> Self {
                let mut data_t = [data[0]; $n];
                for i in 0..data.len() {
                    data_t[i] = data[i];
                }
                Self{
                    current_size: data.len(),
                    data: data_t,
                }
            }
            
            pub fn iter(&self) -> lib::core::slice::Iter<T> {
                self.data[0..self.current_size].iter()
            }
        }
        
        impl $i<u8>{
            pub fn with_str(string: &str) -> Self {
                let mut data: [u8; $n] = [0.into(); $n];
                for (i, element) in data.iter_mut().enumerate().take(string.len()) {
                    *element = string.as_bytes()[i].into();
                }
                Self{
                    current_size: string.len(),
                    data: data,
                }
            }
        }

        impl<T: PrimitiveType> Index<usize> for $i<T> {
            type Output = T;
            
            fn index(&self, index: usize) -> &T {
                &self.data[0..self.current_size][index]
            }
        }
        
        impl< T: PrimitiveType> IndexMut<usize> for $i<T> {
            fn index_mut(&mut self, index: usize) -> &mut T {
                &mut self.data[0..self.current_size][index]
            }
        }


            
        impl<T: PrimitiveType> DynamicArray for $i<T> {
            fn length_bit_length() -> usize {$log_bits}
            
            fn length(&self) -> DynamicArrayLength {DynamicArrayLength{bit_length: $log_bits, current_length: self.current_size}}
            fn set_length(&mut self, length: usize) {self.current_size = length;}

            fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {
                // serialize length
                if *bit < self.length().bit_length {
                    match self.length().serialize(bit, buffer) {
                        SerializationResult::Finished => {
                        },
                        SerializationResult::BufferFull => {
                            return SerializationResult::BufferFull;
                        },
                    }
                }
                
                let mut start_element = (*bit - Self::length_bit_length()) / T::BIT_LENGTH;
                let start_element_bit = (*bit - Self::length_bit_length()) % T::BIT_LENGTH;

                // first get rid of the odd bits
                if start_element_bit != 0 {
                    let mut bits_serialized = start_element_bit;
                    match self[start_element].serialize(&mut bits_serialized, buffer) {
                        SerializationResult::Finished => {
                            *bit += bits_serialized;
                        },
                        SerializationResult::BufferFull => {
                            *bit += bits_serialized;
                            return SerializationResult::BufferFull;
                        },
                    }
                    start_element += 1;
                }

                for i in start_element..self.length().current_length {
                    let mut bits_serialized = 0;
                    match self[i].serialize(&mut bits_serialized, buffer) {
                        SerializationResult::Finished => {
                            *bit += bits_serialized;
                        },
                        SerializationResult::BufferFull => {
                            *bit += bits_serialized;
                            return SerializationResult::BufferFull;
                        },
                    }
                }

                SerializationResult::Finished
            }
            
            fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                
                // deserialize length
                if *bit < self.length().bit_length {
                    let mut length = self.length();
                    match length.deserialize(bit, buffer) {
                        DeserializationResult::Finished => {
                            self.set_length(length.current_length); // ugly hack, fix when dispatching is mostly done static
                        }, 
                        DeserializationResult::BufferInsufficient => {
                            return DeserializationResult::BufferInsufficient
                        },
                    }
                }
                
                let mut start_element = (*bit - Self::length_bit_length()) / T::BIT_LENGTH;
                let start_element_bit = (*bit - Self::length_bit_length()) % T::BIT_LENGTH;
                
                // first get rid of the odd bits
                if start_element_bit != 0 {
                    let mut bits_deserialized = start_element_bit;
                    match self[start_element].deserialize(&mut bits_deserialized, buffer) {
                        DeserializationResult::Finished => {
                            *bit += bits_deserialized;
                        },
                        DeserializationResult::BufferInsufficient => {
                            *bit += bits_deserialized;
                            return DeserializationResult::BufferInsufficient;
                        },
                    }
                    start_element += 1;
                }
                
                for i in start_element..self.length().current_length {
                    let mut bits_deserialized = start_element_bit;
                    match self[i].deserialize(&mut bits_deserialized, buffer) {
                        DeserializationResult::Finished => {
                            *bit += bits_deserialized;
                        },
                        DeserializationResult::BufferInsufficient => {
                            *bit += bits_deserialized;
                            return DeserializationResult::BufferInsufficient;
                        },
                    }
                }

                DeserializationResult::Finished
            }
            
            
        }
        
        // This is needed since it can't be derived for arrays larger than 32 yet
        impl<T: PrimitiveType + cmp::PartialEq> cmp::PartialEq for $i<T> {
            fn eq(&self, other: &Self) -> bool {
                if self.current_size != other.current_size {
                    return false;
                }

                for i in 0..self.current_size {
                    if self.data[i] != other.data[i] {
                        return false;
                    }
                }

                true
            }
        }
            
        // This is needed since it can't be derived for arrays larger than 32 yet
        impl<T: PrimitiveType + fmt::Debug> fmt::Debug for $i<T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "$i<T> {{ data: [")?;
                for i in 0..self.current_size {
                    write!(f, "{:?}, ", self.data[i])?;
                }
                write!(f, "]}}")
            }
        }
        
    };
}

macro_rules! impl_serialize_for_primitive_type {
    ($underlying_type:ty) => {
        fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {            
            let type_bits_remaining = Self::BIT_LENGTH - *bit;
            let buffer_bits_remaining = buffer.bits_remaining();

            if type_bits_remaining == 0 {
                SerializationResult::Finished
            } else if buffer_bits_remaining == 0 {
                SerializationResult::BufferFull
            } else if buffer_bits_remaining >= type_bits_remaining {
                buffer.push_bits(type_bits_remaining, (u64::from(*self) >> *bit));
                *bit = Self::BIT_LENGTH;
                SerializationResult::Finished
            } else {
                buffer.push_bits(buffer_bits_remaining, (u64::from(*self) >> *bit));
                *bit += buffer_bits_remaining;
                SerializationResult::BufferFull
            }
        }

        fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
            let buffer_len = buffer.bit_length();
            if buffer_len == 0 && *bit == Self::BIT_LENGTH {
                DeserializationResult::Finished
            } else if buffer_len == 0 && *bit != Self::BIT_LENGTH {
                DeserializationResult::BufferInsufficient
            } else if buffer_len < Self::BIT_LENGTH - *bit {
                *self |= unsafe{mem::transmute::<$underlying_type, Self>((buffer.pop_bits(buffer_len) << *bit) as $underlying_type)};  //change into something more sensible
                *bit += buffer_len;
                DeserializationResult::BufferInsufficient
            } else {
                *self |= unsafe{mem::transmute::<$underlying_type, Self>((buffer.pop_bits(Self::BIT_LENGTH-*bit) << *bit) as $underlying_type)}; //change into something more sensible
                *bit += Self::BIT_LENGTH;
                DeserializationResult::Finished
            }
            
        }


    };
}


impl PrimitiveType for u2 {
    const BIT_LENGTH: usize = 2;
    impl_serialize_for_primitive_type!(u8);
}

impl PrimitiveType for u3 {
    const BIT_LENGTH: usize = 3;
    impl_serialize_for_primitive_type!(u8);
}

impl PrimitiveType for u4 {
    const BIT_LENGTH: usize = 4;
    impl_serialize_for_primitive_type!(u8);
}

impl PrimitiveType for u5 {
    const BIT_LENGTH: usize = 5;
    impl_serialize_for_primitive_type!(u8);
}

impl PrimitiveType for u7 {
    const BIT_LENGTH: usize = 7;
    impl_serialize_for_primitive_type!(u8);
}

impl PrimitiveType for u8 {
    const BIT_LENGTH: usize =  8;
    impl_serialize_for_primitive_type!(u8);
}

impl PrimitiveType for u16 {
    const BIT_LENGTH: usize = 16;
    impl_serialize_for_primitive_type!(u16);
}
    
impl PrimitiveType for u32 {
    const BIT_LENGTH: usize = 32;
    impl_serialize_for_primitive_type!(u32);
}

dynamic_array_def!(DynamicArray3, 3, 2);
dynamic_array_def!(DynamicArray4, 4, 3);
dynamic_array_def!(DynamicArray5, 5, 3);
dynamic_array_def!(DynamicArray6, 6, 3);
dynamic_array_def!(DynamicArray7, 7, 3);
dynamic_array_def!(DynamicArray8, 8, 4);
dynamic_array_def!(DynamicArray9, 9, 4);
dynamic_array_def!(DynamicArray10, 10, 4);
dynamic_array_def!(DynamicArray11, 11, 4);
dynamic_array_def!(DynamicArray12, 12, 4);
dynamic_array_def!(DynamicArray13, 13, 4);
dynamic_array_def!(DynamicArray14, 14, 4);
dynamic_array_def!(DynamicArray15, 15, 4);
dynamic_array_def!(DynamicArray16, 16, 5);
dynamic_array_def!(DynamicArray31, 31, 5);
dynamic_array_def!(DynamicArray32, 32, 6);
dynamic_array_def!(DynamicArray90, 90, 7);
