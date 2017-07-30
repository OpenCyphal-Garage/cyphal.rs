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
    


#[cfg(test)]
mod tests {

    use {
        UavcanIndexable,
        UavcanPrimitiveField,
        UavcanPrimitiveType,
    };

    use serializer::{
        Serializer,
    };
    
    use types::{
        Uint2,
        Uint3,
        Uint8,
        Uint16,
        Uint32,
    };
    
    #[test]
    fn uavcan_serialize_test_byte_aligned() {

        #[derive(UavcanIndexable)]
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
        
        
        #[derive(UavcanIndexable)]
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
}

