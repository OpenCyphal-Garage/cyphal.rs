use {
    UavcanIndexable,
};

pub struct Serializer<T: UavcanIndexable> {
    structure: T,
    current_field_index: usize,
    current_type_index: usize,
    current_bit_index: usize,
}

impl<T: UavcanIndexable> Serializer<T> {
    pub fn from_structure(structure: T) -> Self {
        Self{
            structure: structure,
            current_field_index: 0,
            current_type_index: 0,
            current_bit_index: 0,
        }
    }

    /// serialize(&self, buffer: &mut [u]) -> usize
    ///
    /// serialize into buffer untill one of two occurs
    /// 1. buffer is full
    /// 2. structure is exahusted (all data have been serialized)
    /// The return value is number of bytes serialized
    /// (rounded up to closest whole byte)
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        unimplemented!()
    }

}
