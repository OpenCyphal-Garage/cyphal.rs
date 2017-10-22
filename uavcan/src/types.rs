pub use ux::{
    u2, u3, u4, u5, u6, u7, u9, u10, u11, u12, u13, u14, u15, u17, u18, u19, u20, u21, u22, u23, u24, u25, u26, u27, u28, u29, u30, u31,
    u33, u34, u35, u36, u37, u38, u39, u40, u41, u42, u43, u44, u45, u46, u47, u48, u49, u50, u51, u52, u53, u54, u55, u56, u57, u58, u59, u60, u61, u62, u63,
    i2, i3, i4, i5, i6, i7, i9, i10, i11, i12, i13, i14, i15, i17, i18, i19, i20, i21, i22, i23, i24, i25, i26, i27, i28, i29, i30, i31,
    i33, i34, i35, i36, i37, i38, i39, i40, i41, i42, i43, i44, i45, i46, i47, i48, i49, i50, i51, i52, i53, i54, i55, i56, i57, i58, i59, i60, i61, i62, i63,
};
pub use half::f16;

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

/// This trait is only exposed so `Struct` can be derived.
/// It is not intended for use outside the derive macro and
/// must not be considered as a stable part of the API.
#[doc(hidden)]
pub trait PrimitiveType : Sized + Copy{
    const BIT_LENGTH: usize;
    
    /// Mask bits exceeding `BIT_LENGTH`
    fn from_bits(v: u64) -> Self;
    
    /// Zeroes bits exceeding `BIT_LENGTH`
    fn to_bits(self) -> u64;

    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {            
        let type_bits_remaining = Self::BIT_LENGTH - *bit;
        let buffer_bits_remaining = buffer.bits_remaining();
        
        if type_bits_remaining == 0 {
            SerializationResult::Finished
        } else if buffer_bits_remaining == 0 {
            SerializationResult::BufferFull
        } else if buffer_bits_remaining >= type_bits_remaining {
            buffer.push_bits(type_bits_remaining, (self.to_bits() >> *bit));
            *bit = Self::BIT_LENGTH;
            SerializationResult::Finished
        } else {
            buffer.push_bits(buffer_bits_remaining, (self.to_bits() >> *bit));
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
            *self = Self::from_bits(self.to_bits() | (buffer.pop_bits(buffer_len) << *bit));
            *bit += buffer_len;
            DeserializationResult::BufferInsufficient
        } else {
            *self = Self::from_bits(self.to_bits() | (buffer.pop_bits(Self::BIT_LENGTH-*bit) << *bit));
            *bit += Self::BIT_LENGTH;
            DeserializationResult::Finished
        }
        
    }
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


macro_rules! impl_primitive_types_ux{
    {[$(($type:ident, $bits:expr)),*], $underlying_type:ident} => {$(impl_primitive_types_ux!($type, $bits, $underlying_type);)*};
    ($type:ident, $bits:expr, $underlying_type:ident) => {
        impl PrimitiveType for $type {
            const BIT_LENGTH: usize = $bits;
            fn from_bits(v: u64) -> Self {
                $type::new(v as $underlying_type)
            }
            fn to_bits(self) -> u64 {
                u64::from(self)
            }
        }
    };
}

macro_rules! impl_primitive_types_ix{
    {[$(($type:ident, $bits:expr)),*], $underlying_type:ident} => {$(impl_primitive_types_ix!($type, $bits, $underlying_type);)*};
    ($type:ident, $bits:expr, $underlying_type:ident) => {
        impl PrimitiveType for $type {
            const BIT_LENGTH: usize = $bits;
            fn from_bits(v: u64) -> Self {
                $type::new(v as $underlying_type)
            }
            fn to_bits(self) -> u64 {
                i64::from(self) as u64
            }
        }
    };
}

impl_primitive_types_ux!([(u2, 2), (u3, 3), (u4, 4), (u5, 5), (u6, 6), (u7, 7)], u8);

impl_primitive_types_ux!([(u9, 9), (u10, 10), (u11, 11), (u12, 12), (u13, 13), (u14, 14), (u15, 15)], u16);

impl_primitive_types_ux!([(u17, 17), (u18, 18), (u19, 19), (u20, 20), (u21, 21), (u22, 22), (u23, 23), (u24, 24),
                          (u25, 25), (u26, 26), (u27, 27), (u28, 28), (u29, 29), (u30, 30), (u31, 31)], u32);

impl_primitive_types_ux!([(u33, 33), (u34, 34), (u35, 35), (u36, 36), (u37, 37), (u38, 38), (u39, 39), (u40, 40),
                          (u41, 41), (u42, 42), (u43, 43), (u44, 44), (u45, 45), (u46, 46), (u47, 47), (u48, 48),
                          (u49, 49), (u50, 50), (u51, 51), (u52, 52), (u53, 53), (u54, 54), (u55, 55), (u56, 56),
                          (u57, 57), (u58, 58), (u59, 59), (u60, 60), (u61, 61), (u62, 62), (u63, 63)], u64);



impl_primitive_types_ix!([(i2, 2), (i3, 3), (i4, 4), (i5, 5), (i6, 6), (i7, 7)], i8);

impl_primitive_types_ix!([(i9, 9), (i10, 10), (i11, 11), (i12, 12), (i13, 13), (i14, 14), (i15, 15)], i16);

impl_primitive_types_ix!([(i17, 17), (i18, 18), (i19, 19), (i20, 20), (i21, 21), (i22, 22), (i23, 23), (i24, 24),
                          (i25, 25), (i26, 26), (i27, 27), (i28, 28), (i29, 29), (i30, 30), (i31, 31)], i32);

impl_primitive_types_ix!([(i33, 33), (i34, 34), (i35, 35), (i36, 36), (i37, 37), (i38, 38), (i39, 39), (i40, 40),
                          (i41, 41), (i42, 42), (i43, 43), (i44, 44), (i45, 45), (i46, 46), (i47, 47), (i48, 48),
                          (i49, 49), (i50, 50), (i51, 51), (i52, 52), (i53, 53), (i54, 54), (i55, 55), (i56, 56),
                          (i57, 57), (i58, 58), (i59, 59), (i60, 60), (i61, 61), (i62, 62), (i63, 63)], i64);


impl PrimitiveType for u8 {
    const BIT_LENGTH: usize =  8;
    fn from_bits(v: u64) -> Self {
        v as u8
    }
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
}

impl PrimitiveType for u16 {
    const BIT_LENGTH: usize = 16;
    fn from_bits(v: u64) -> Self {
        v as u16
    }
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
}
    
impl PrimitiveType for u32 {
    const BIT_LENGTH: usize = 32;
    fn from_bits(v: u64) -> Self {
        v as u32
    }
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
}

impl PrimitiveType for u64 {
    const BIT_LENGTH: usize = 64;
    fn from_bits(v: u64) -> Self {
        v
    }
    fn to_bits(self) -> u64 {
        self
    }
}

impl PrimitiveType for i8 {
    const BIT_LENGTH: usize = 8;
    fn from_bits(v: u64) -> Self {
        v as i8
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}

impl PrimitiveType for i16 {
    const BIT_LENGTH: usize = 16;
    fn from_bits(v: u64) -> Self {
        v as i16
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}
    
impl PrimitiveType for i32 {
    const BIT_LENGTH: usize = 32;
    fn from_bits(v: u64) -> Self {
        v as i32
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}

impl PrimitiveType for i64 {
    const BIT_LENGTH: usize = 64;
    fn from_bits(v: u64) -> Self {
        v as i64
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}

impl PrimitiveType for f16 {
    const BIT_LENGTH: usize = 16;
    fn from_bits(v: u64) -> Self {
        f16::from_bits(v as u16)
    }
    fn to_bits(self) -> u64 {
        u64::from(f16::as_bits(self))
    }
}

impl PrimitiveType for f32 {
    const BIT_LENGTH: usize = 32;
    fn from_bits(v: u64) -> Self {
        let mut v32 = v as u32;
        const EXP_MASK: u32   = 0x7F800000;
        const FRACT_MASK: u32 = 0x007FFFFF;
        if v32  & EXP_MASK == EXP_MASK && v32 & FRACT_MASK != 0 {
            v32 = unsafe { lib::core::mem::transmute(lib::core::f32::NAN) };
        }
        unsafe { lib::core::mem::transmute::<u32, f32>(v32) }
    }
    fn to_bits(self) -> u64 {
        (unsafe { lib::core::mem::transmute::<f32, u32>(self) }) as u64

            
    }
}

impl PrimitiveType for f64 {
    const BIT_LENGTH: usize = 64;
    fn from_bits(mut v: u64) -> Self {
        const EXP_MASK: u64   = 0x7FF0000000000000;
        const FRACT_MASK: u64 = 0x000FFFFFFFFFFFFF;
        if v & EXP_MASK == EXP_MASK && v & FRACT_MASK != 0 {
            v = unsafe { lib::core::mem::transmute(lib::core::f64::NAN) };
        }
        unsafe { lib::core::mem::transmute(v) }
    }
    fn to_bits(self) -> u64 {
        unsafe { lib::core::mem::transmute(self) }
    }
}

impl PrimitiveType for bool {
    const BIT_LENGTH: usize = 1;
    fn from_bits(v: u64) -> Self {
        v & 1 == 1
    }
    fn to_bits(self) -> u64 {
        if self {
            1
        } else {
            0
        }
    }
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
