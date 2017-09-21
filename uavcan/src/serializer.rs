use types::*;

use {
    UavcanStruct,
    UavcanPrimitiveType,
    UavcanField,
    DynamicArray,
    DynamicArrayLength,
};

use crc;

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
    pub bit_index: usize,
}

impl<'a> SerializationBuffer<'a> {
    pub fn full(&self) -> bool {
        if self.bit_index == self.data.len()*8 {true}
        else {false}
    }
}
    


pub struct Serializer<T: UavcanStruct> {
    structure: T,
    field_index: usize,
    bit_index: usize,
}


impl<T: UavcanStruct> Serializer<T> {
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
        self.structure.serialize(&mut self.field_index, &mut self.bit_index, buffer)
    }

    pub fn single_frame_transfer(&self) -> bool {
        self.structure.bit_length() <= 8*8
    }

    pub fn crc(&mut self, data_type_signature: u64) -> u16 {
        let mut crc = 0xffff;

        let field_index = self.field_index;
        let bit_index = self.bit_index;
        
        for i in 0..4 {
            crc = crc::add_byte(crc, &(data_type_signature.get_bits(8*i..8*(i+1)) as u8));;
        }

        loop {
            let mut buffer = [0u8; 8];
            
            let mut serialization_buffer = SerializationBuffer{data: &mut buffer, bit_index: 0};
            if let SerializationResult::Finished = self.serialize(&mut serialization_buffer) {
                crc = crc::add(crc, &serialization_buffer.data);
                self.field_index = field_index;
                self.bit_index = bit_index;
                return crc;
            } else {
                crc = crc::add(crc, &serialization_buffer.data);
            }
            
        }
        
    }
        



}       
    


#[cfg(test)]
mod tests {
    
    use uavcan;

    use {
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
        UavcanPrimitiveType,
    };

    use serializer::*;
    
    use types::*;

    #[test]
    fn uavcan_serialize_primitive_types() {
        let uint2: Uint2 = 1.into();
        let uint8: Uint8 = 128.into();
        let uint16: Uint16 = 257.into();

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        let mut bits_serialized = 0;
        assert_eq!(uint2.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 2);
        assert_eq!(buffer.data, [1, 0, 0, 0]);

        buffer.bit_index = 0;
        bits_serialized = 0;
        assert_eq!(uint8.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 8);
        assert_eq!(buffer.data, [128, 0, 0, 0]);
            
        buffer.bit_index = 0;
        bits_serialized = 0;
        assert_eq!(uint16.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 16);
        assert_eq!(buffer.data, [1, 1, 0, 0]);
            
        uint2.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 0]);
            
        uint8.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [1, 1, 1, 2]);
            

    }

    #[test]
    fn uavcan_serialize_dynamic_array() {
        let a1: DynamicArray4<Uint2> = DynamicArray4::with_data(&[1.into(), 0.into(), 1.into(), 0.into()]);
        let a2: DynamicArray6<Uint2> = DynamicArray6::with_data(&[1.into(), 0.into(), 1.into(), 0.into(), 1.into(), 0.into()]);
        let a3: DynamicArray4<Uint7> = DynamicArray4::with_data(&[1.into(), 2.into(), 4.into(), 8.into()]);

        let mut data = [0u8; 4];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        let mut bits_serialized = 0;
        assert_eq!(a1.serialize(&mut bits_serialized, &mut buffer), SerializationResult::Finished);
        assert_eq!(bits_serialized, 11);
        assert_eq!(buffer.data, [0b10001100, 0, 0, 0]);

        buffer.bit_index = 0;
        a2.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [0b10001110, 0b0001000, 0, 0]);
            
        buffer.bit_index = 0;
        a3.serialize(&mut 0, &mut buffer);
        assert_eq!(buffer.data, [12, 8, 8, 8]);            

    }

    #[test]
    fn uavcan_serialize_dynamic_array_without_length() {
        let a: DynamicArray6<Uint7> = DynamicArray6::with_data(&[1.into(), 4.into(), 16.into(), 64.into()]);

        let mut data = [0u8; 1];
        let mut buffer = SerializationBuffer{data: &mut data, bit_index: 0};

        a.serialize(&mut 3, &mut buffer);
        assert_eq!(buffer.data, [1]);
        
        buffer.bit_index = 0;
        a.serialize(&mut 11, &mut buffer);
        assert_eq!(buffer.data, [2]);

        buffer.bit_index = 0;
        a.serialize(&mut 19, &mut buffer);
        assert_eq!(buffer.data, [4]);

        buffer.bit_index = 0;
        a.serialize(&mut 27, &mut buffer);
        assert_eq!(buffer.data, [8]);

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

        
        let mut buffer = SerializationBuffer{bit_index: 0, data: &mut array};
        serializer.serialize(&mut buffer);

        assert_eq!(buffer.data, [17, 19, 0, 0, 0, 21, 0, 23]);
        
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

        let mut buffer = SerializationBuffer{bit_index: 0, data: &mut array};
        serializer.serialize(&mut buffer);

        assert_eq!(buffer.data, [1, 0, 0, 0, 0b10001110, 5, 0]);      

        
    }

    
}

