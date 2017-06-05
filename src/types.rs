use core::mem::transmute;
use bit::BitIndex;

use {
    UavcanIndexable,
    UavcanPrimitiveField,
    UavcanPrimitiveType,
};


#[derive(Debug, PartialEq)]
pub struct f16 {
    bitfield: u16,
}

#[allow(non_camel_case_types)]
impl f16 {
    fn from_bitmap(bm: u16) -> f16 {
        f16{bitfield: bm}
    }
}



#[derive(Debug, PartialEq)]
pub struct Bool {
    value: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Uint2 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint3 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint4 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint5 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint6 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint7 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint8 {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub struct Uint9 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint10 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint11 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint12 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint13 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint14 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint15 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint16 {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct Uint17 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint18 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint19 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint20 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint21 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint22 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint23 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint24 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint25 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint26 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint27 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint28 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint29 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint30 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint31 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Uint32 {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct Float16 {
    value: f16,
}

#[derive(Debug, PartialEq)]
pub struct Float32 {
    value: f32,
}

#[derive(Debug, PartialEq)]
pub struct Float64 {
    value: f64,
}







impl From<u8> for Uint2 {
    fn from(t: u8) -> Uint2 {
        Uint2{value: t.bit_range(0..2)}
    }
}

impl From<u8> for Uint3 {
    fn from(t: u8) -> Uint3 {
        Uint3{value: t.bit_range(0..3)}
    }
}

impl From<u8> for Uint8 {
    fn from(t: u8) -> Uint8 {
        Uint8{value: t.bit_range(0..8)}
    }
}

impl From<u16> for Uint16 {
    fn from(t: u16) -> Uint16 {
        Uint16{value: t.bit_range(0..16)}
    }
}

impl From<u32> for Uint32 {
    fn from(t: u32) -> Uint32 {
        Uint32{value: t.bit_range(0..32)}
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
        self.get_size()
    }
    fn primitive_field_as_mut(&mut self, field_number: usize) -> Option<&mut UavcanPrimitiveField>{
        if field_number == 0 {
            Some(self)
        } else {
            None
        }
    }
    fn primitive_field(&self, field_number: usize) -> Option<&UavcanPrimitiveField>{
        if field_number == 0 {
            Some(self)
        } else {
            None
        }
    }
}


impl<T: UavcanPrimitiveType> UavcanPrimitiveField for T{
    fn is_constant_size(&self) -> bool{
        true
    }
    fn get_size(&self) -> usize{
        1
    }
    fn get_size_mut(&self) -> Option<&mut usize>{
        None
    }
    fn primitive_type_as_mut(&mut self, index: usize) -> Option<&mut UavcanPrimitiveType> {
        if index == 0 {
            Some(self)
        } else {
            None
        }
    }
    fn primitive_type(&self, index: usize) -> Option<&UavcanPrimitiveType> {
        if index == 0 {
            Some(self)
        } else {
            None
        }
    }
}





impl UavcanPrimitiveType for Bool {
    fn bitlength(&self) -> usize {
        1
    }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        if buffer[0] & 1 == 0 {
            self.value = false;
        } else {
            self.value == true;
        }
    }
}

impl UavcanPrimitiveType for Uint2 {
    fn bitlength(&self) -> usize { 2 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..2, buffer[0].bit_range(0..2));
    }
}

impl UavcanPrimitiveType for Uint3 {
    fn bitlength(&self) -> usize { 3 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..3, buffer[0].bit_range(0..3));
    }
}

impl UavcanPrimitiveType for Uint4 {
    fn bitlength(&self) -> usize { 4 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..4, buffer[0].bit_range(0..4));
    }
}

impl UavcanPrimitiveType for Uint5 {
    fn bitlength(&self) -> usize { 5 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..5, buffer[0].bit_range(0..5));
    }
}

impl UavcanPrimitiveType for Uint6 {
    fn bitlength(&self) -> usize { 6 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..6, buffer[0].bit_range(0..6));
    }
}

impl UavcanPrimitiveType for Uint7 {
    fn bitlength(&self) -> usize { 7 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..7, buffer[0].bit_range(0..7));
    }
}

impl UavcanPrimitiveType for Uint8 {
    fn bitlength(&self) -> usize { 8 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value = buffer[0];
    }
}

impl UavcanPrimitiveType for Uint16 {
    fn bitlength(&self) -> usize { 16 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..8, buffer[0].bit_range(0..8) as u16)
            .set_bit_range(8..16, buffer[1].bit_range(0..8) as u16);
    }
}

impl UavcanPrimitiveType for Uint32 {
    fn bitlength(&self) -> usize { 32 }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        self.value.set_bit_range(0..8, buffer[0].bit_range(0..8) as u32)
            .set_bit_range(8..16, buffer[1].bit_range(0..8) as u32)
            .set_bit_range(16..24, buffer[2].bit_range(0..8) as u32)
            .set_bit_range(24..32, buffer[3].bit_range(0..8) as u32);
    }
}








impl UavcanPrimitiveType for Float16 {
    fn bitlength(&self) -> usize {
        16
    }

    fn set_from_bytes(&mut self, buffer: &[u8]) {
        let bm: u16 = (buffer[0] as u16) | ((buffer[1] as u16) << 8);
        self.value = f16::from_bitmap(bm);
    }
}

impl UavcanPrimitiveType for Float32 {
    fn bitlength(&self) -> usize {
        32
    }

    fn set_from_bytes(&mut self, buffer: &[u8]) {
        let bm: u32 = (buffer[0] as u32)
            | ((buffer[0] as u32) << 8)
            | ((buffer[1] as u32) << 16)
            | ((buffer[2] as u32) << 24);
        self.value = unsafe { transmute::<u32, f32>(bm) };
    }
}

impl UavcanPrimitiveType for Float64 {
    fn bitlength(&self) -> usize {
        64
    }

    fn set_from_bytes(&mut self, buffer: &[u8]) {
        let bm: u64 = (buffer[0] as u64)
            | ((buffer[0] as u64) << 8)
            | ((buffer[1] as u64) << 16)
            | ((buffer[2] as u64) << 24)
            | ((buffer[3] as u64) << 32)
            | ((buffer[4] as u64) << 40)
            | ((buffer[5] as u64) << 48)
            | ((buffer[6] as u64) << 56);
        self.value = unsafe { transmute::<u64, f64>(bm) };
    }

}
