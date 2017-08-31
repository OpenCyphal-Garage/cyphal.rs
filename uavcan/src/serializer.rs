use lib::core::ops::Range;

use types::*;

use {
    UavcanStruct,
    UavcanPrimitiveType,
    UavcanField,
    DynamicArray,
};

use crc;

use bit_field::{
    BitField,
};

pub enum SerializationResult {
    BufferFull(usize),
    Finished(usize),
}

impl SerializationResult {
    pub fn bits_serialized(&self) -> usize {
        match self {
            &SerializationResult::BufferFull(bits) => bits,
            &SerializationResult::Finished(bits) => bits,
        }
    }
}

#[derive(Debug, PartialEq)]
struct SerializationBuffer<'a> {
    data: &'a mut [u8],
    bit_index: usize,
}
 
trait Serialize {
    fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult;
}

macro_rules! impl_serialize_for_primitive_type {
    ($type:ident) => {
        impl Serialize for $type {
            fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult {
                let mut bits_serialized: usize = 0;
                
                let mut byte_start = buffer.bit_index / 8;
                let odd_bits_start = buffer.bit_index % 8;
                
                // first get rid of the odd bits
                if odd_bits_start != 0 && 8-odd_bits_start <= $type::bit_length() - start_bit {
                    buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.get_bits(start_bit..(start_bit+8-odd_bits_start)) as u8);
                    bits_serialized += 8-odd_bits_start;
                    buffer.bit_index += 8-odd_bits_start;
                    byte_start += 1;
                } else if odd_bits_start != 0 && 8-odd_bits_start > $type::bit_length() - start_bit {
                    buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.get_bits(start_bit..(start_bit + ($type::bit_length() - start_bit) )) as u8);
                    bits_serialized += $type::bit_length() - start_bit;
                    buffer.bit_index += $type::bit_length() - start_bit;
                    return SerializationResult::Finished(bits_serialized);
                }
                
                for i in byte_start..buffer.data.len() {
                    let serialization_index = bits_serialized + start_bit;
                    let remaining_bits = $type::bit_length() - serialization_index;

                    if remaining_bits == 0 {
                        return SerializationResult::Finished(bits_serialized);
                    } else if remaining_bits <= 8 {
                        buffer.data[i] = self.get_bits(serialization_index..serialization_index+remaining_bits) as u8;
                        buffer.bit_index += remaining_bits;
                        bits_serialized += remaining_bits;
                        return SerializationResult::Finished(bits_serialized);
                    } else {
                        buffer.data[i] = self.get_bits(serialization_index..(serialization_index+8)) as u8;
                        buffer.bit_index += 8;
                        bits_serialized += 8;
                    }
                }
                
                
                SerializationResult::BufferFull(bits_serialized)

            }
        }
    };
}

impl_serialize_for_primitive_type!(Uint2);
impl_serialize_for_primitive_type!(Uint3);
impl_serialize_for_primitive_type!(Uint4);
impl_serialize_for_primitive_type!(Uint5);

impl_serialize_for_primitive_type!(Uint8);

impl_serialize_for_primitive_type!(Uint16);

impl_serialize_for_primitive_type!(Uint32);

pub struct Serializer<T: UavcanStruct> {
    structure: T,
    field_index: usize,
    type_index: usize,
    bit_index: usize,
}


impl<T: UavcanStruct> Serializer<T> {
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
            let primitive_field = self.structure.field(self.field_index);
            let primitive_type = primitive_field.bit_array(self.type_index);
            let buffer_bits_remaining = buffer_bit_length - buffer_next_bit;
            let type_bits_remaining = primitive_type.bit_length() - self.bit_index;
            
            if type_bits_remaining == 0 {
                if self.type_index < primitive_field.length()-1 {
                    self.type_index += 1;
                    self.bit_index = 0;
                } else if self.field_index < self.structure.flattened_fields_len() - 1 {
                    self.bit_index = 0;
                    self.type_index = 0;
                    self.field_index += 1;
                    
                    // Dynamic length array tail optimization
                    if (self.field_index == self.structure.flattened_fields_len() - 1) && !self.structure.field(self.field_index).constant_sized() {
                        self.type_index = 1;
                    }
                } else {
                    return SerializationResult::Finished(buffer_next_bit);
                }
            } else if buffer_bits_remaining >= 8 && type_bits_remaining >= 8 {
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
        if self.any_remaining_bits() {
            return SerializationResult::BufferFull;
        } else {
            return SerializationResult::Finished(buffer_next_bit);
        }
    }

    fn any_remaining_bits(&self) -> bool {
        let mut bits_counted = 0;
        
        let mut field_index = self.field_index;
        let mut type_index = self.type_index;
        let mut bit_index = self.bit_index;

        loop {
            let primitive_type = self.structure.field(field_index).bit_array(type_index);
            bits_counted += primitive_type.bit_length() - bit_index;
            
            if bits_counted > 0 { return true;}
            
            bit_index = 0;
            type_index += 1;

            if type_index >= self.structure.field(field_index).length() {
                type_index = 0;
                field_index += 1;
            }
            if field_index >= self.structure.flattened_fields_len() {
                return bits_counted > 0;
            }
            
        }
    }

    pub fn remaining_bits(&self) -> usize {
        let mut bits_counted = 0;
        
        let mut field_index = self.field_index;
        let mut type_index = self.type_index;
        let mut bit_index = self.bit_index;

        loop {
            let primitive_type = self.structure.field(field_index).bit_array(type_index);
            bits_counted += primitive_type.bit_length() - bit_index;
            
            bit_index = 0;
            type_index += 1;

            if type_index >= self.structure.field(field_index).length() {
                type_index = 0;
                field_index += 1;
            }
            if field_index >= self.structure.flattened_fields_len() {
                return bits_counted
            }
        }
    }

    pub fn crc(&self, data_type_signature: u64) -> u16 {
        let mut field_index = 0;
        let mut type_index = 0;

        let mut crc = 0xffff;
        let mut remaining_data: u8 = 0;
        let mut remaining_bits = 0;

        for i in 0..4 {
            crc = crc::add_byte(crc, &(data_type_signature.get_bits(8*i..8*(i+1)) as u8));;
        }
        
        loop {
            let primitive_type = self.structure.field(field_index).bit_array(type_index);
            let bit_length = primitive_type.bit_length();
            let mut bit_index = 0;
            let data = primitive_type.get_bits(0..bit_length);

            if remaining_bits + bit_length < 8 {
                remaining_data.set_bits(remaining_bits as u8..(remaining_bits+bit_length) as u8, data.get_bits(0..bit_length as u8) as u8);
                bit_index = bit_length;
                remaining_bits += bit_length;
            } else {
                if remaining_bits != 0 {
                    crc::add_byte(crc, &0u8
                                  .set_bits(0..remaining_bits as u8, remaining_data.get_bits(0..remaining_bits as u8) as u8)
                                  .set_bits(remaining_bits as u8..8, data.get_bits(0..8-remaining_bits as u8) as u8)
                                  .get_bits(0..8)
                    );
                    bit_index += 8-remaining_bits;
                    remaining_bits = 0;
                }
                
                while bit_length - bit_index >= 8 {
                    crc::add_byte(crc, &(data.get_bits(bit_index as u8..bit_index as u8+8) as u8));
                    bit_index += 8;
                }
                
                remaining_bits = bit_length-bit_index;
                if bit_length-bit_index != 0 {
                    remaining_data = data.get_bits(bit_index as u8..bit_length as u8) as u8;
                }
            }
                    
            type_index += 1;
            
            if type_index >= self.structure.field(field_index).length() {
                type_index = 0;
                field_index += 1;
            }
            if field_index >= self.structure.flattened_fields_len() {
                return crc;
            }
        }
        
    }
}       
    


#[cfg(test)]
mod tests {

    use {
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
        UavcanPrimitiveType,
    };

    use serializer::{
        Serializer,
        Serialize,
        SerializationBuffer,
    };
    
    use types::{
        Uint2,
        Uint3,
        Uint8,
        Uint16,
        Uint32,
    };


    #[test]
    fn uavcan_serialize_primitive_types() {
        let uint2: Uint2 = 1.into();
        let uint8: Uint8 = 128.into();
        let uint16: Uint16 = 257.into();

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        uint2.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 0, 0, 0]);

        buffer.bit_index = 0;
        uint8.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [128, 0, 0, 0]);
            
        buffer.bit_index = 0;
        uint16.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 0, 0]);
            
        uint2.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 0]);
            
        uint8.serialize(0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 2]);
            

    }

    #[test]
    fn uavcan_serialize_test_byte_aligned() {

        #[derive(UavcanStruct)]
        struct Message {
            v1: Uint8,
            v2: Uint32,
            v3: Uint16,
            v4: Uint8,
        }


        let message = Message{
            v1: 17.into(),
            v2: 19.into(),
            v3: 21.into(),
            v4: 23.into(),
        };

        let mut serializer: Serializer<Message> = Serializer::from_structure(message);
        let mut array: [u8; 8] = [0; 8];

        serializer.serialize(&mut array);

        assert_eq!(array, [17, 19, 0, 0, 0, 21, 0, 23]);
        
    }


    #[test]
    fn uavcan_parse_test_misaligned() {
        
        
        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        let message = NodeStatus{
            uptime_sec: 1.into(),
            health: 2.into(),
            mode: 3.into(),
            sub_mode: 4.into(),
            vendor_specific_status_code: 5.into(),
        };

        let mut serializer: Serializer<NodeStatus> = Serializer::from_structure(message);
        let mut array: [u8; 7] = [0; 7];

        serializer.serialize(&mut array);

        assert_eq!(array, [1, 0, 0, 0, 0b10001110, 5, 0]);      

        
    }

    
    #[test]
    fn remaining_bits() {

        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        let message = NodeStatus{
            uptime_sec: 1.into(),
            health: 2.into(),
            mode: 3.into(),
            sub_mode: 4.into(),
            vendor_specific_status_code: 5.into(),
        };

        let mut serializer: Serializer<NodeStatus> = Serializer::from_structure(message);

        assert_eq!(serializer.remaining_bits(), 56);

        let mut array: [u8; 7] = [0; 7];
        serializer.serialize(&mut array);

        assert_eq!(serializer.remaining_bits(), 0);
    }
}

