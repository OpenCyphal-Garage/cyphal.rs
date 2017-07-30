use {
    UavcanIndexable,
};

use bit_field::BitArray;

pub struct Serializer<T: UavcanIndexable> {
    structure: T,
    field_index: usize,
    type_index: usize,
    bit_index: usize,
}

pub enum SerializationResult {
    BufferFull,
    Finished(usize),
}

impl<T: UavcanIndexable> Serializer<T> {
    pub fn from_structure(structure: T) -> Self {
        Self{
            structure: structure,
            field_index: 0,
            type_index: 0,
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
        let buffer_bit_length = buffer.len()*8;
        let mut buffer_next_bit = 0;

        while buffer_next_bit < buffer_bit_length {
            let primitive_type = self.structure.primitive_field(self.field_index).primitive_type(self.type_index);
            let buffer_bits_remaining = buffer_bit_length - buffer_next_bit;
            let type_bits_remaining = primitive_type.bit_length() - self.bit_index;

            if type_bits_remaining == 0 {
                self.type_index += 1;
                self.bit_index = 0;
                if self.type_index >= self.structure.primitive_field(self.field_index).get_size() {
                    self.type_index = 0;
                    self.field_index += 1;
                }
                if self.field_index >= self.structure.number_of_primitive_fields() {
                    return SerializationResult::Finished(buffer_next_bit);
                }
            }else if buffer_bits_remaining >= 8 && type_bits_remaining >= 8 {
                buffer.set_bits(buffer_next_bit..buffer_next_bit+8, primitive_type.get_bits(self.bit_index..self.bit_index+8) as u8);
                buffer_next_bit += 8;
                self.bit_index += 8;
            } else if buffer_bits_remaining <= type_bits_remaining {
                buffer.set_bits(buffer_next_bit..buffer_bit_length, primitive_type.get_bits(self.bit_index..self.bit_index+(buffer_bit_length-buffer_next_bit)) as u8);
                self.bit_index += buffer_bit_length - buffer_next_bit;
                buffer_next_bit = buffer_bit_length;
            } else if buffer_bits_remaining > type_bits_remaining {
                buffer.set_bits(buffer_next_bit..buffer_next_bit+type_bits_remaining, primitive_type.get_bits(self.bit_index..self.bit_index+type_bits_remaining) as u8);
                buffer_next_bit += type_bits_remaining;
                self.bit_index += type_bits_remaining;
            }
        }
        return SerializationResult::BufferFull;
    }
}       
    
