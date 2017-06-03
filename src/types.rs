use core::mem::transmute;

pub trait UavcanIndexable {
    fn uavcan_bit_size(&self) -> usize;
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize>;
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize>;
}

pub struct f16 {
    bitfield: u16,
}


pub struct Bool {
    value: bool,
}

pub struct IntX {
    x: usize,
    value: i64,
}

pub struct UintX {
    x: usize,
    value: u64,
}

pub struct Float16 {
    value: f16,
}

pub struct Float32 {
    value: f32,
}

pub struct Float64 {
    value: f64,
}

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







impl UavcanIndexable for Bool {
    fn uavcan_bit_size(&self) -> usize {
        1
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }
}

impl UavcanIndexable for IntX {
    fn uavcan_bit_size(&self) -> usize {
        self.x
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }

}

impl UavcanIndexable for UintX {
    fn uavcan_bit_size(&self) -> usize {
        self.x
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }   
}

impl UavcanIndexable for Float16 {
    fn uavcan_bit_size(&self) -> usize {
        16
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }
}

impl UavcanIndexable for Float32 {
    fn uavcan_bit_size(&self) -> usize {
        32
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }
}

impl UavcanIndexable for Float64 {
    fn uavcan_bit_size(&self) -> usize {
        64
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }
}

impl UavcanIndexable for VoidX {
    fn uavcan_bit_size(&self) -> usize {
        self.x
    }
    fn field_start_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(0)
        } else {
            None
        }
    }
    fn field_length_from_field_num(&self, field_num: usize) -> Option<usize> {
        if field_num == 0 {
            Some(self.uavcan_bit_size())
        } else {
            None
        }
    }
}



