use bit_field::BitField;
use bit_field::BitArray;
use lib::core::ops::Range;

use {
    UavcanIndexable,
    UavcanField,
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
    ($i:ident, $size:ident, $n:expr, $log_bits:expr) => {
        struct $size {
            value: usize,
        }

        impl BitArray<u64> for $size {
            #[inline] fn bit_length(&self) -> usize { $log_bits }
            #[inline] fn get_bit(&self, bit: usize) -> bool { self.value.get_bit(bit as u8) }
            #[inline] fn get_bits(&self, range: Range<usize>) -> u64 { self.value.get_bits(range.start as u8..range.end as u8) as u64}
            #[inline] fn set_bit(&mut self, bit: usize, value: bool) { self.value.set_bit(bit as u8, value); }
            #[inline] fn set_bits(&mut self, range: Range<usize>, value: u64) { self.value.set_bits((range.start as u8..range.end as u8), value as usize); }
        }
        
        pub struct $i<T: UavcanPrimitiveType> {
            current_size: $size,
            data: [T; $n],
        }
        
        impl $i<Uint8>{
            pub fn with_str(string: &str) -> Self {
                let mut data = [0.into(); $n];
                for i in 0..string.len() {
                    data[i] = string.as_bytes()[i].into();
                }
                Self{
                    current_size: $size{value: string.len()},
                    data: data,
                }
            }
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
                    current_size: $size{value: data.len()},
                    data: data_t,
                }
            }
            fn set_length(&mut self, length: usize) {self.current_size.value = length;}
            fn data(&self) -> &[T] {&self.data[0..self.current_size.value]}
            fn data_as_mut(&mut self) -> &mut [T] {&mut self.data[0..self.current_size.value]}
        }
        
        impl<T: UavcanPrimitiveType> UavcanField for $i<T> {
            fn constant_sized(&self) -> bool {false}
            fn length(&self) -> usize {self.current_size.value+1}
            fn bit_array(&self, index: usize) -> &BitArray<u64> {
                if index == 0 { &self.current_size }
                else { &self.data[index-1] }
            }
            fn bit_array_as_mut(&mut self, index: usize) -> &mut BitArray<u64> {
                if index == 0 { &mut self.current_size }
                else { &mut self.data[index-1] }
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



impl<T: UavcanField> UavcanIndexable for T {
    fn number_of_primitive_fields(&self) -> usize{
        1
    }
    fn field_as_mut(&mut self, field_number: usize) -> &mut UavcanField{
        assert!(field_number == 0);
        self
    }
    fn field(&self, field_number: usize) -> &UavcanField{
        assert!(field_number == 0);
        self
    }
}

macro_rules! impl_primitive_field {
    ($i:ident) => {
        impl UavcanField for $i{
            fn constant_sized(&self) -> bool{
                true
            }
            fn length(&self) -> usize{
                1
            }
            fn bit_array_as_mut(&mut self, index: usize) -> &mut BitArray<u64> {
                assert!(index == 0);
                self
            }
            fn bit_array(&self, index: usize) -> &BitArray<u64> {
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

dynamic_array_def!(DynamicArray3, DynamicArray3Size, 3, 2);
dynamic_array_def!(DynamicArray4, DynamicArray4Size, 4, 3);
dynamic_array_def!(DynamicArray5, DynamicArray5Size, 5, 3);
dynamic_array_def!(DynamicArray6, DynamicArray6Size, 6, 3);
dynamic_array_def!(DynamicArray7, DynamicArray7Size, 7, 3);
dynamic_array_def!(DynamicArray8, DynamicArray8Size, 8, 4);
dynamic_array_def!(DynamicArray9, DynamicArray9Size, 9, 4);
dynamic_array_def!(DynamicArray10, DynamicArray10Size, 10, 4);
dynamic_array_def!(DynamicArray11, DynamicArray11Size, 11, 4);
dynamic_array_def!(DynamicArray12, DynamicArray12Size, 12, 4);
dynamic_array_def!(DynamicArray13, DynamicArray13Size, 13, 4);
dynamic_array_def!(DynamicArray14, DynamicArray14Size, 14, 4);
dynamic_array_def!(DynamicArray15, DynamicArray15Size, 15, 4);
dynamic_array_def!(DynamicArray16, DynamicArray16Size, 16, 5);
dynamic_array_def!(DynamicArray31, DynamicArray31Size, 31, 5);
dynamic_array_def!(DynamicArray32, DynamicArray32Size, 32, 6);
dynamic_array_def!(DynamicArray90, DynamicArray90Size, 90, 7);
