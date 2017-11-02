use {
    Struct,
};

use crc::TransferCRC;

use bit_field::{
    BitField,
};

#[derive(Debug, PartialEq)]
pub enum SerializationResult {
    BufferFull,
    Finished,
}

#[derive(Debug, PartialEq)]
pub struct SerializationBuffer<'a> {
    pub data: &'a mut [u8],
    start_bit_index: usize,
    stop_bit_index: usize,
}

impl<'a> SerializationBuffer<'a> {
    pub fn with_empty_buffer(buffer: &'a mut [u8]) -> Self {
        Self{data: buffer, start_bit_index: 0, stop_bit_index: 0}
    }
        
    pub fn with_full_buffer(buffer: &'a mut [u8]) -> Self {
        let data_len = buffer.len()*8;
        Self{data: buffer, start_bit_index: 0, stop_bit_index: data_len}
    }
        
    pub fn bit_length(&self) -> usize { self.stop_bit_index - self.start_bit_index }
    pub fn bits_remaining(&self) -> usize { self.data.len()*8 - self.bit_length() }

    pub fn pop_bits(&mut self, bit_length: usize) -> u64 {
        assert!(bit_length <= 64);
        assert!(bit_length <= self.bit_length());
        
        let mut bits = 0u64;
        let mut bit = 0;
        
        let mut remaining_bits = bit_length - bit;

        let byte_start = self.start_bit_index / 8;
        let bit_start = self.start_bit_index % 8;

        // first get rid of the odd bits
        if bit_start != 0 && remaining_bits >= (8-bit_start) {
            bits.set_bits(0..(8-bit_start as u8), self.data[byte_start].get_bits(0..(8-bit_start as u8)) as u64);
            self.start_bit_index += 8-bit_start;
            bit += 8-bit_start;
        } else if bit_start != 0 && remaining_bits < (8-bit_start) {
            bits.set_bits(0..(remaining_bits as u8), self.data[byte_start].get_bits(((8 - bit_start - remaining_bits) as u8)..(8 - bit_start as u8)) as u64);
            self.start_bit_index += remaining_bits;
            bit += remaining_bits;
        }

        remaining_bits = bit_length - bit;
        
        while remaining_bits != 0 {
            if remaining_bits >= 8 {
                bits.set_bits((bit as u8)..(bit as u8 + 8), self.data[self.start_bit_index/8] as u64);
                bit += 8;
                self.start_bit_index += 8;
            } else {
                bits.set_bits((bit as u8)..(bit_length as u8), self.data[self.start_bit_index/8].get_bits((8 - remaining_bits as u8)..8) as u64);
                bit += remaining_bits;
                self.start_bit_index += remaining_bits;
            }
            remaining_bits = bit_length - bit;
        }
        
        bits
    }

    
    pub fn push_bits(&mut self, bit_length: usize, bits: u64) {
        assert!(bit_length <= 64);
        assert!(self.stop_bit_index + bit_length <= self.data.len()*8);

        let mut bit = 0;
        let mut remaining_bits = bit_length;
        
        let mut byte_start = self.stop_bit_index / 8;
        let odd_bits_start = self.stop_bit_index % 8;
        
        // first get rid of the odd bits
        if odd_bits_start != 0 && 8-odd_bits_start <= remaining_bits {
            self.data[byte_start].set_bits(0..(8-odd_bits_start as u8), bits.get_bits((bit as u8)..(bit+8-odd_bits_start) as u8) as u8);
            self.stop_bit_index += 8-odd_bits_start;
            bit += 8-odd_bits_start;
            byte_start += 1;
        } else if odd_bits_start != 0 && 8-odd_bits_start > remaining_bits {
            self.data[byte_start].set_bits(((8-odd_bits_start-remaining_bits) as u8)..(8-odd_bits_start as u8), bits.get_bits((bit as u8)..(bit + (bit_length - bit) ) as u8) as u8);
            self.stop_bit_index += remaining_bits;
            return;
        }
        
        for i in byte_start..self.data.len() {
            remaining_bits = bit_length - bit;

            if remaining_bits == 0 {
                return;
            } else if remaining_bits <= 8 {
                self.data[i].set_bits((8-remaining_bits as u8)..8 ,bits.get_bits((bit as u8)..(bit+remaining_bits) as u8) as u8);
                self.stop_bit_index += remaining_bits;
                return;
            } else {
                self.data[i] = bits.get_bits((bit as u8)..(bit+8) as u8) as u8;
                self.stop_bit_index += 8;
                bit += 8;
            }
        }
        
    }


    
}



pub struct Serializer<T: Struct> {
    structure: T,
    field_index: usize,
    bit_index: usize,
}


impl<T: Struct> Serializer<T> {
    pub fn from_structure(structure: T) -> Self {
        Self{
            structure: structure,
            field_index: 0,
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
    pub fn serialize(&mut self, buffer: &mut SerializationBuffer) -> SerializationResult {
        let last_field = self.field_index == T::FLATTENED_FIELDS_NUMBER-1;
        self.structure.serialize(&mut self.field_index, &mut self.bit_index, last_field, buffer)
    }

    pub fn single_frame_transfer(&self) -> bool {
        self.structure.bit_length() <= 8*8
    }

    pub fn crc(&mut self, data_type_signature: u64) -> u16 {
        let mut crc = TransferCRC::from_signature(data_type_signature);

        let field_index = self.field_index;
        let bit_index = self.bit_index;

        self.field_index = 0;
        self.bit_index = 0;
        
        loop {
            let mut buffer = [0u8; 8];
            
            let mut serialization_buffer = SerializationBuffer::with_empty_buffer(&mut buffer);
            if let SerializationResult::Finished = self.serialize(&mut serialization_buffer) {
                crc.add(&serialization_buffer.data[0..(serialization_buffer.stop_bit_index+7)/8]);
                self.field_index = field_index;
                self.bit_index = bit_index;
                return crc.into();
            } else {
                crc.add(&serialization_buffer.data);
            }
            
        }
        
    }
        



}       
    


#[cfg(test)]
mod tests {
    
    use uavcan;

    use serializer::*;
    
    use types::*;

    #[test]
    fn buffer_test() {
        let mut data = [0u8; 8];
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut data);

        buffer.push_bits(10, 0x1aa);
        assert_eq!(buffer.start_bit_index, 0);
        assert_eq!(buffer.stop_bit_index, 10);
        assert_eq!(buffer.pop_bits(10), 0x1aa);
        assert_eq!(buffer.start_bit_index, 10);
        assert_eq!(buffer.stop_bit_index, 10);
        
        buffer.push_bits(8, 0xaa);
        assert_eq!(buffer.start_bit_index, 10);
        assert_eq!(buffer.stop_bit_index, 18);
        assert_eq!(buffer.pop_bits(8), 0xaa);
        assert_eq!(buffer.start_bit_index, 18);
        assert_eq!(buffer.stop_bit_index, 18);
        
        buffer.push_bits(2, 0b11);
        assert_eq!(buffer.pop_bits(2), 0b11);
        
      
        buffer.push_bits(8, 0xaa);
        buffer.push_bits(2, 0b11);
        assert_eq!(buffer.pop_bits(8), 0xaa);
        assert_eq!(buffer.pop_bits(2), 0b11);

        
        buffer.push_bits(7, 0);
        buffer.push_bits(3, 0b111);
        assert_eq!(buffer.pop_bits(7), 0);
        assert_eq!(buffer.pop_bits(3), 0b111);
        
        
        buffer.push_bits(5, 0b10101);
        buffer.push_bits(4, 0b1111);
        buffer.push_bits(15, 0b100000000000001);
        assert_eq!(buffer.pop_bits(5), 0b10101);
        assert_eq!(buffer.pop_bits(4), 0b1111);
        assert_eq!(buffer.pop_bits(15), 0b100000000000001);
    }
    
    #[test]
    fn uavcan_serialize_primitive_types() {
        let uint2: u2 = u2::new(1);
        let uint8: u8 = 128;
        let uint16: u16 = 257;
        let int16: i16 = -1;
        let int7: i7 = i7::new(-64);
        let float16: f16 = f16::from_f32(3.141592);
        let float32: f32 = 1.0;
        let float64: f64 = 2.718281828459045235360;

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut data);

        let mut bits_serialized = 0;
        assert_eq!(uint2.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 2);
        assert_eq!(buffer.data, [0b01000000, 0, 0, 0]);

        buffer.stop_bit_index = 0;
        bits_serialized = 0;
        assert_eq!(uint8.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 8);
        assert_eq!(buffer.data, [128, 0, 0, 0]);
            
        buffer.stop_bit_index = 0;
        bits_serialized = 0;
        assert_eq!(uint16.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 16);
        assert_eq!(buffer.data, [1, 1, 0, 0]);
            
        uint2.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 0b01000000, 0]);
            
        uint8.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 0b01000000, 0b10000000]);

        buffer.stop_bit_index = 0;
        int16.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data[0..2], [0xff, 0xff]);

        buffer.stop_bit_index = 0;
        buffer.data[0] = 0;
        int7.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data[0], (-64i8 as u8) << 1);
        
        buffer.stop_bit_index = 0;
        float16.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data[0..2], [f16::from_f32(3.141592).as_bits() as u8, (f16::from_f32(3.141592).as_bits() >> 8) as u8]);
            
        buffer.stop_bit_index = 0;
        float32.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [0x00, 0x00, 0x80, 0x3f]);
            
        buffer.stop_bit_index = 0;
        float64.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [0x69, 0x57, 0x14, 0x8b]);
            

    }

    #[test]
    fn uavcan_serialize_dynamic_array() {
        let a1 = Dynamic::<[u2; 4]>::with_data(&[u2::new(1), u2::new(0), u2::new(1), u2::new(0)]);
        let a2 = Dynamic::<[u2; 6]>::with_data(&[u2::new(1), u2::new(0), u2::new(1), u2::new(0), u2::new(1), u2::new(0)]);
        let a3 = Dynamic::<[u7; 4]>::with_data(&[u7::new(1), u7::new(2), u7::new(4), u7::new(8)]);

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut data);

        let mut bits_serialized = 0;
        assert_eq!(a1.serialize(&mut bits_serialized, false, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 11);
        assert_eq!(buffer.data, [0b10001001, 0, 0, 0]);

        buffer.stop_bit_index = 0;
        a2.serialize(&mut 0, false, &mut buffer);
        assert_eq!(buffer.data, [0b11001001, 0b00001000, 0, 0]);
            
        buffer.stop_bit_index = 0;
        a3.serialize(&mut 0, false, &mut buffer);
        assert_eq!(buffer.data, [0b10000001, 0b00000010, 0b00000100, 0b00010000]);

    }

    #[test]
    fn uavcan_serialize_dynamic_array_without_length() {
        let a = Dynamic::<[u7; 6]>::with_data(&[u7::new(1), u7::new(1), u7::new(1), u7::new(1)]);

        let mut data = [0u8; 1];
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut data);

        a.serialize(&mut 3, false, &mut buffer);
        assert_eq!(buffer.data, [0b00000011]);
        
        buffer.stop_bit_index = 0;
        a.serialize(&mut 11, false, &mut buffer);
        assert_eq!(buffer.data, [0b00000001]);

        buffer.stop_bit_index = 0;
        a.serialize(&mut 19, false, &mut buffer);
        assert_eq!(buffer.data, [0b00000001]);

        buffer.stop_bit_index = 0;
        a.serialize(&mut 27, false, &mut buffer);
        assert_eq!(buffer.data[0].get_bits((8 - buffer.stop_bit_index as u8)..8), 0b00000000);

    }

    #[test]
    fn uavcan_serialize_test_byte_aligned() {

        #[derive(UavcanStruct)]
        struct Message {
            v1: u8,
            v2: u32,
            v3: u16,
            v4: u8,
        }


        let message = Message{
            v1: 17,
            v2: 19,
            v3: 21,
            v4: 23,
        };

        let mut serializer: Serializer<Message> = Serializer::from_structure(message);
        let mut array: [u8; 8] = [0; 8];

        
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut array);
        serializer.serialize(&mut buffer);

        assert_eq!(buffer.data, [17, 19, 0, 0, 0, 21, 0, 23]);
        
    }

    #[test]
    fn uavcan_serialize_static_array() {

        #[derive(UavcanStruct)]
        struct Message {
            a: [u16; 4],
        }


        let message = Message{
            a: [5, 6, 7, 8],
        };

        let mut serializer: Serializer<Message> = Serializer::from_structure(message);
        let mut array: [u8; 8] = [0; 8];

        
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut array);
        serializer.serialize(&mut buffer);

        assert_eq!(buffer.data, [5, 0, 6, 0, 7, 0, 8, 0]);
        
    }


    #[test]
    fn uavcan_parse_test_misaligned() {
        
        
        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: u32,
            health: u2,
            mode: u3,
            sub_mode: u3,
            vendor_specific_status_code: u16,
        }

        let message = NodeStatus{
            uptime_sec: 1,
            health: u2::new(2),
            mode: u3::new(3),
            sub_mode: u3::new(4),
            vendor_specific_status_code: 5,
        };

        let mut serializer: Serializer<NodeStatus> = Serializer::from_structure(message);
        let mut array: [u8; 7] = [0; 7];

        let mut buffer = SerializationBuffer::with_empty_buffer(&mut array);
        serializer.serialize(&mut buffer);

        assert_eq!(buffer.data, [1, 0, 0, 0, 0b10011100, 5, 0]);      

        
    }

    #[test]
    fn uavcan_parse_padded() {

        #[derive(UavcanStruct, Default)]
        struct Message {
            v1: u8,
            _v2: void32,
            v3: u16,
            v4: u8,
        }


        let message = Message{
            v1: 17,
            v3: 21,
            v4: 23,
            .. Default::default()
        };

        let mut serializer: Serializer<Message> = Serializer::from_structure(message);
        let mut array: [u8; 8] = [0; 8];

        
        let mut buffer = SerializationBuffer::with_empty_buffer(&mut array);
        serializer.serialize(&mut buffer);

        assert_eq!(buffer.data, [17, 0, 0, 0, 0, 21, 0, 23]);
        
    }

    
}

