use bit_field::BitField;
use bit_field::BitArray;
use half::f16;
use lib;
use lib::core::ops::Range;
use lib::core::fmt;
use lib::core::cmp;
use lib::core::ops::{
    Index,
    IndexMut,
};

use {
    UavcanField,
    AsUavcanField,
    UavcanPrimitiveType,
    DynamicArray,
};



#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Bool {
    value: bool,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint2 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint3 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint4 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint5 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint6 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint7 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint8 {
    value: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint9 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint10 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint11 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint12 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint13 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint14 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint15 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint16 {
    value: u16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint17 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint18 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint19 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint20 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint21 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint22 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint23 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint24 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint25 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint26 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint27 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint28 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint29 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint30 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint31 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Uint32 {
    value: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Float16 {
    value: f16,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Float32 {
    value: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Float64 {
    value: f64,
}

impl<T: UavcanPrimitiveType> AsUavcanField for T {
    fn as_uavcan_field(&self) -> &UavcanField {
        &UavcanField::PrimitiveType(self)
    }
    fn as_mut_uavcan_field(&mut self) -> &mut UavcanField {
        &mut UavcanField::PrimitiveType(self)
    }
}

macro_rules! dynamic_array_def {
    ($i:ident, $n:expr, $log_bits:expr) => {

        pub struct $i<T: UavcanPrimitiveType> {
            current_size: usize,
            data: [T; $n],
        }
        
        impl<T: UavcanPrimitiveType + Copy> $i<T> {
            fn with_data(data: &[T]) -> Self {
                let mut data_t = [data[0]; $n];
                for i in 0..data.len() {
                    data_t[i] = data[i];
                }
                Self{
                    current_size: data.len(),
                    data: data_t,
                }
            }
            
            fn iter(&self) -> lib::core::slice::Iter<T> {
                self.data[0..self.current_size].iter()
            }
        }
        
        impl $i<Uint8>{
            pub fn with_str(string: &str) -> Self {
                let mut data: [Uint8; $n] = [0.into(); $n];
                for (i, element) in data.iter_mut().enumerate().take(string.len()) {
                    *element = string.as_bytes()[i].into();
                }
                Self{
                    current_size: string.len(),
                    data: data,
                }
            }
        }

        impl<T: UavcanPrimitiveType> Index<usize> for $i<T> {
            type Output = T;
            
            fn index(&self, index: usize) -> &T {
                &self.data[0..self.current_size][index]
            }
        }
        
        impl< T: UavcanPrimitiveType> IndexMut<usize> for $i<T> {
            fn index_mut(&mut self, index: usize) -> &mut T {
                &mut self.data[0..self.current_size][index]
            }
        }


            
        impl<T: UavcanPrimitiveType> DynamicArray for $i<T> {
            fn max_size() -> usize {$n}
            
            fn length_bit_length() -> usize {$log_bits}
            fn element_bit_length() -> usize {T::bit_length()}
            
            fn set_length(&mut self, length: usize) {self.current_size = length;}
            fn element(&self, index: usize) -> &UavcanPrimitiveType {& self.data[0..self.current_size][index]}
            fn element_as_mut(&mut self, index: usize) -> &mut UavcanPrimitiveType {&mut self.data[0..self.current_size][index]}
        }
        
        impl<T: UavcanPrimitiveType> AsUavcanField for $i<T> {
            fn as_uavcan_field(&self) -> &UavcanField {
                &UavcanField::DynamicArray(self)
            }
            fn as_mut_uavcan_field(&mut self) -> &mut UavcanField {
                &mut UavcanField::DynamicArray(self)
            }
        }

        // This is needed since it can't be derived for arrays larger than 32 yet
        impl<T: UavcanPrimitiveType + cmp::PartialEq> cmp::PartialEq for $i<T> {
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
        impl<T: UavcanPrimitiveType + fmt::Debug> fmt::Debug for $i<T> {
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



impl From<u8> for Uint2 {
    fn from(t: u8) -> Uint2 {
        Uint2{value: t.get_bits(0..2)}
    }
}

impl From<u8> for Uint3 {
    fn from(t: u8) -> Uint3 {
        Uint3{value: t.get_bits(0..3)}
    }
}

impl From<u8> for Uint8 {
    fn from(t: u8) -> Uint8 {
        Uint8{value: t.get_bits(0..8)}
    }
}

impl From<u16> for Uint16 {
    fn from(t: u16) -> Uint16 {
        Uint16{value: t.get_bits(0..16)}
    }
}

impl From<u32> for Uint32 {
    fn from(t: u32) -> Uint32 {
        Uint32{value: t.get_bits(0..32)}
    }
}

impl From<Bool> for bool {
    fn from(t: Bool) -> bool {
        t.value
    }
}

impl From<Float16> for f16 {
    fn from(t: Float16) -> f16 {
        t.value
    }
}

impl From<Float32> for f32 {
    fn from(t: Float32) -> f32 {
        t.value
    }
}

impl From<Float64> for f64 {
    fn from(t: Float64) -> f64 {
        t.value
    }
}



impl UavcanPrimitiveType for Uint2 {
    #[inline] fn bit_length() -> usize { 2 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl UavcanPrimitiveType for Uint3 {
    #[inline] fn bit_length() -> usize { 3 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl UavcanPrimitiveType for Uint4 {
    #[inline] fn bit_length() -> usize { 4 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl UavcanPrimitiveType for Uint5 {
    #[inline] fn bit_length() -> usize { 5 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl UavcanPrimitiveType for Uint8 {
    #[inline] fn bit_length() -> usize { 8 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl UavcanPrimitiveType for Uint16 {
    #[inline] fn bit_length() -> usize { 16 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u16); }
}
    
impl UavcanPrimitiveType for Uint32 {
    #[inline] fn bit_length() -> usize { 32 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u32); }
}

impl UavcanPrimitiveType for Float16 {
    #[inline] fn bit_length() -> usize { 16 }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.as_bits().get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.as_bits().set_bits((range.start as u8..range.end as u8), value as u16); }
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
