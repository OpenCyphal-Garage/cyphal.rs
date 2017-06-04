use core::mem::transmute;

pub trait UavcanIndexable {
    fn number_of_primitive_fields(&self) -> usize;
    fn primitive_field_as_mut(&mut self, field_number: usize) -> Option<&mut UavcanPrimitiveField>;
}


/// An UavcanPrimitiveField is a field of a flatted out uavcan struct
///
/// It's a superset of Primitive Data Types from the uavcan protocol
/// also containing both constant and variable size arrays.
///
/// All primitive data types have 1 primitive fields,
/// All composite data structures have the same number of primtiive fields
/// as the sum of their members. Except the variable length array.
/// This array has number of primitive fields as their members (elements)+1
pub trait UavcanPrimitiveField{
    fn is_constant_size(&self) -> bool;
    /// get_size(&self) -> usize returns the number of primitive data types in this field
    ///
    /// for primtiive data types (non-array) it will return 1
    fn get_size(&self) -> usize;
    /// get_size_mut(&self) -> Option<&mut usize> returns a mutable reference to the size
    /// if the field is of variable size, or None if the field is constant size 
    fn get_size_mut(&self) -> Option<&mut usize>;
    fn primitive_type_as_mut(&mut self, index: usize) -> Option<&mut UavcanPrimitiveType>;    
}

pub trait UavcanPrimitiveType{
    fn bitlength(&self) -> usize;
    fn set_from_bytes(&mut self, buffer: &[u8]);
}


#[derive(Debug, PartialEq)]
pub struct f16 {
    bitfield: u16,
}

impl f16 {
    fn from_bitmap(bm: u16) -> f16 {
        f16{bitfield: bm}
    }
}



#[derive(Debug, PartialEq)]
pub struct Bool {
    value: bool,
}

#[derive(Debug, PartialEq)]
pub struct IntX {
    x: usize,
    value: i64,
}

#[derive(Debug, PartialEq)]
pub struct UintX {
    x: usize,
    value: u64,
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

#[derive(Debug, PartialEq)]
pub struct VoidX{
    x: usize,
}





impl Bool {
    pub fn new(value: bool) -> Bool {
        Bool{value: value}
    }
}

impl IntX {
    pub fn new(x: usize, value: i64) -> IntX {
        IntX{x: x, value: value}
    }
}

impl UintX {
    pub fn new(x: usize, value: u64) -> UintX {
        UintX{x: x, value: value}
    }
}

impl Float16 {
    pub fn new(value: f16) -> Float16 {
        Float16{value: value}
    }
}

impl Float32 {
    pub fn new(value: f32) -> Float32 {
        Float32{value: value}
    }
}

impl Float64 {
    pub fn new(value: f64) -> Float64 {
        Float64{value: value}
    }
}

impl VoidX {
    pub fn new(x: usize) -> VoidX {
        VoidX{x: x}
    }
}





impl From<Bool> for bool {
    fn from(t: Bool) -> bool {
        t.value
    }
}

impl From<IntX> for i64 {
    fn from(t: IntX) -> i64 {
        t.value
    }
}

impl From<UintX> for u64 {
    fn from(t: UintX) -> u64 {
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

impl UavcanPrimitiveType for IntX {
    fn bitlength(&self) -> usize {
        self.x
    }
    
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        let mut temp_bm: u64 = 0;
        for i in 0..((self.x/8) + 1) {
            temp_bm |= (buffer[i] as u64) << i*8;
        }
        self.value = unsafe { transmute::<u64, i64>(temp_bm) };
    }

}

impl UavcanPrimitiveType for UintX {
    fn bitlength(&self) -> usize {
        self.x
    }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        let mut temp_value: u64 = 0;
        for i in 0..(self.x/8) {
            temp_value |= (buffer[i] as u64) << i*8;
        }
        self.value = temp_value;
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

impl UavcanPrimitiveType for VoidX {
    fn bitlength(&self) -> usize {
        self.x
    }
    fn set_from_bytes(&mut self, buffer: &[u8]) {
        // consider doing a check that only 0 is set?
    }
}

