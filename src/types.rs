use core::mem::transmute;
use bit_field::BitField;
use bit_field::BitArray;
use core::ops::Range;

use {
    UavcanIndexable,
    UavcanPrimitiveField,
    UavcanPrimitiveType,
    DynamicArray,
};


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct f16 {
    bitfield: u16,
}

impl f16 {
    fn from_bitmap(bm: u16) -> f16 {
        f16{bitfield: bm}
    }
}



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

macro_rules! dynamic_array_def {
    ($i:ident, $n:expr) => {
        pub struct $i<T: UavcanPrimitiveType> {
            current_size: usize,
            data: [T; $n],
        }
        
        impl<T: UavcanPrimitiveType + Copy> DynamicArray for $i<T> {
            type Field = T;
            
            fn max_size() -> usize {$n}
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
            fn set_length(&mut self, length: usize) {self.current_size = length;}
            fn data(&self) -> &[T] {&self.data[0..self.current_size]}
            fn data_as_mut(&mut self) -> &mut [T] {&mut self.data[0..self.current_size]}
        }
        
        impl<T: UavcanPrimitiveType> UavcanPrimitiveField for $i<T> {
            fn is_constant_size(&self) -> bool {false}
            fn length(&self) -> usize {self.data.len()}
            fn set_length(&mut self, length: usize) {self.current_size = length;}
            fn primitive_type(&self, index: usize) -> &UavcanPrimitiveType {&self.data[index]}
            fn primitive_type_as_mut(&mut self, index: usize) -> &mut UavcanPrimitiveType {&mut self.data[index]}
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



impl<T: UavcanPrimitiveField> UavcanIndexable for T {
    fn number_of_primitive_fields(&self) -> usize{
        self.length()
    }
    fn primitive_field_as_mut(&mut self, field_number: usize) -> &mut UavcanPrimitiveField{
        assert!(field_number == 0);
        self
    }
    fn primitive_field(&self, field_number: usize) -> &UavcanPrimitiveField{
        assert!(field_number == 0);
        self
    }
}

macro_rules! impl_primitive_field {
    ($i:ident) => {
        impl UavcanPrimitiveField for $i{
            fn is_constant_size(&self) -> bool{
                true
            }
            fn length(&self) -> usize{
                1
            }
            fn set_length(&mut self, _length: usize) {
                panic!("Can't set size for a constant sized type (UavcanPrimitiveType)");
            }
            fn primitive_type_as_mut(&mut self, index: usize) -> &mut UavcanPrimitiveType {
                assert!(index == 0);
                self
            }
            fn primitive_type(&self, index: usize) -> &UavcanPrimitiveType {
                assert!(index == 0);
                self
            }
        }
    };
}




impl BitArray<u64> for Uint2 {
    #[inline] fn bit_length(&self) -> usize { 2 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl BitArray<u64> for Uint3 {
    #[inline] fn bit_length(&self) -> usize { 3 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl BitArray<u64> for Uint4 {
    #[inline] fn bit_length(&self) -> usize { 4 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl BitArray<u64> for Uint5 {
    #[inline] fn bit_length(&self) -> usize { 5 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl BitArray<u64> for Uint8 {
    #[inline] fn bit_length(&self) -> usize { 8 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u8); }
}

impl BitArray<u64> for Uint16 {
    #[inline] fn bit_length(&self) -> usize { 16 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u16); }
}
    
impl BitArray<u64> for Uint32 {
    #[inline] fn bit_length(&self) -> usize { 32 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as u32); }
}

impl BitArray<u64> for Float16 {
    #[inline] fn bit_length(&self) -> usize { 16 }
    #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.bitfield.get_bit(bit as u8) }
    #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.bitfield.get_bits(range.start as u8..range.end as u8) as u64}
    #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.bitfield.set_bit(bit as u8, value); }
    #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.bitfield.set_bits((range.start as u8..range.end as u8), value as u16); }
}


impl UavcanPrimitiveType for Uint2 {}
impl UavcanPrimitiveType for Uint3 {}
impl UavcanPrimitiveType for Uint4 {}
impl UavcanPrimitiveType for Uint5 {}
impl UavcanPrimitiveType for Uint8 {}
impl UavcanPrimitiveType for Uint16 {}
impl UavcanPrimitiveType for Uint32 {}
impl UavcanPrimitiveType for Float16 {}

impl_primitive_field!(Uint2);
impl_primitive_field!(Uint3);
impl_primitive_field!(Uint4);
impl_primitive_field!(Uint5);
impl_primitive_field!(Uint8);
impl_primitive_field!(Uint16);
impl_primitive_field!(Uint32);

dynamic_array_def!(DynamicArray2, 2);
dynamic_array_def!(DynamicArray3, 3);
dynamic_array_def!(DynamicArray4, 4);
dynamic_array_def!(DynamicArray5, 5);
dynamic_array_def!(DynamicArray6, 6);
dynamic_array_def!(DynamicArray7, 7);
dynamic_array_def!(DynamicArray8, 8);
dynamic_array_def!(DynamicArray9, 9);
dynamic_array_def!(DynamicArray10, 10);
dynamic_array_def!(DynamicArray11, 11);
dynamic_array_def!(DynamicArray12, 12);
dynamic_array_def!(DynamicArray13, 13);
dynamic_array_def!(DynamicArray14, 14);
dynamic_array_def!(DynamicArray15, 15);
dynamic_array_def!(DynamicArray16, 16);
dynamic_array_def!(DynamicArray31, 31);
dynamic_array_def!(DynamicArray32, 32);
dynamic_array_def!(DynamicArray90, 90);
