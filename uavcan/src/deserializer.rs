use lib::core::mem;

use bit_field::BitField;

use {
    Struct,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeserializationResult {
    Finished,
    BufferInsufficient,
}

#[derive(Debug, PartialEq)]
pub struct DeserializationBuffer<'a> {
    pub data: &'a mut [u8],
    start_bit_index: usize,
    stop_bit_index: usize,
}

pub struct Deserializer<T: Struct> {
    structure: T,
    field_index: usize,
    bit_index: usize,
    optimize_tail_array: bool,
}


impl<'a> DeserializationBuffer<'a> {
    pub fn with_full_buffer(buffer: &'a mut [u8]) -> Self {
        let data_len = buffer.len() * 8;
        Self { data: buffer, start_bit_index: 0, stop_bit_index: data_len }
    }

    pub fn bit_length(&self) -> usize { self.stop_bit_index - self.start_bit_index }

    pub fn pop_bits(&mut self, bit_length: usize) -> u64 {
        assert!(bit_length <= 64);
        assert!(bit_length <= self.bit_length());

        let mut bits = 0u64;
        let mut bit = 0;

        let mut remaining_bits = bit_length - bit;

        let byte_start = self.start_bit_index / 8;
        let bit_start = self.start_bit_index % 8;

        // first get rid of the odd bits
        if bit_start != 0 && remaining_bits >= (8 - bit_start) {
            bits.set_bits(0..(8 - bit_start as u8), self.data[byte_start].get_bits(0..(8 - bit_start as u8)) as u64);
            self.start_bit_index += 8 - bit_start;
            bit += 8 - bit_start;
        } else if bit_start != 0 && remaining_bits < (8 - bit_start) {
            bits.set_bits(0..(remaining_bits as u8), self.data[byte_start].get_bits(((8 - bit_start - remaining_bits) as u8)..(8 - bit_start as u8)) as u64);
            self.start_bit_index += remaining_bits;
            bit += remaining_bits;
        }

        remaining_bits = bit_length - bit;

        while remaining_bits != 0 {
            if remaining_bits >= 8 {
                bits.set_bits((bit as u8)..(bit as u8 + 8), self.data[self.start_bit_index / 8] as u64);
                bit += 8;
                self.start_bit_index += 8;
            } else {
                bits.set_bits((bit as u8)..(bit_length as u8), self.data[self.start_bit_index / 8].get_bits((8 - remaining_bits as u8)..8) as u64);
                bit += remaining_bits;
                self.start_bit_index += remaining_bits;
            }
            remaining_bits = bit_length - bit;
        }

        bits
    }
}

impl<T: Struct> Deserializer<T> {
    pub fn new(optimize_tail_array: bool) -> Deserializer<T> {
        let structure: T;
        unsafe {
            structure = mem::zeroed();
        };            
        Deserializer{
            structure,
            field_index: 0,
            bit_index: 0,
            optimize_tail_array,
        }
    }

    pub fn deserialize(&mut self, input: &mut [u8]) -> DeserializationResult {
        let mut buffer = DeserializationBuffer::with_full_buffer(input);
        self.structure.deserialize(&mut self.field_index, &mut self.bit_index, self.optimize_tail_array, &mut buffer)
    }

    pub fn into_structure(self) -> Result<T, ()> {
        Ok(self.structure)    
    }
}


#[cfg(test)]
mod tests {

    use bit_field::BitField;
    
    use *;
    use deserializer::*;
    use types::*;
    
    #[test]
    fn uavcan_parse_test_byte_aligned() {

        #[derive(Debug, PartialEq, Clone, UavcanStruct, Default)]
        struct Message {
            v1: u8,
            v2: u32,
            v3: u16,
            v4: u8,
        }

        let mut deserializer: Deserializer<Message> = Deserializer::new(true);

        deserializer.deserialize(&mut [17, 19, 0, 0, 0, 21, 0, 23]);

        let parsed_message = deserializer.into_structure().unwrap();

        
        assert_eq!(parsed_message.v1, 17);
        assert_eq!(parsed_message.v2, 19);
        assert_eq!(parsed_message.v3, 21);
        assert_eq!(parsed_message.v4, 23);
    }




    #[test]
    fn uavcan_parse_test_misaligned() {
        
        
        #[derive(Debug, PartialEq, Clone, UavcanStruct, Default)]
        struct NodeStatus {
            uptime_sec: u32,
            health: u2,
            mode: u3,
            sub_mode: u3,
            vendor_specific_status_code: u16,
        }

        
        let mut deserializer: Deserializer<NodeStatus> = Deserializer::new(true);

        deserializer.deserialize(&mut [1, 0, 0, 0, 0b10011100, 5, 0]);

        let parsed_message = deserializer.into_structure().unwrap();
        

        assert_eq!(parsed_message.uptime_sec, 1);
        assert_eq!(parsed_message.health, u2::new(2));
        assert_eq!(parsed_message.mode, u3::new(3));
        assert_eq!(parsed_message.sub_mode, u3::new(4));
        assert_eq!(parsed_message.vendor_specific_status_code, 5);
        
    }

    #[test]
    fn deserialize_dynamic_array() {

        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        struct TestMessage {
            pad: u5,
            text1: Dynamic<[u8; 7]>,
            text2: Dynamic<[u8; 8]>,
        }
        
        let mut deserializer: Deserializer<TestMessage> = Deserializer::new(true);

        deserializer.deserialize(&mut [0u8.set_bits(0..3, 4).get_bits(0..8), b't', b'e', b's', b't', b'l', b'o', b'l']);
        
        let parsed_message = deserializer.into_structure().unwrap();

        assert_eq!(parsed_message,
                   TestMessage{
                       pad: u5::new(0),
                       text1: Dynamic::<[u8; 7]>::with_data("test".as_bytes()),
                       text2: Dynamic::<[u8; 8]>::with_data("lol".as_bytes()),
                   }
        );
    }

    #[test]
    fn tail_array_optimization_struct() {
        #[derive(Debug, PartialEq, UavcanStruct, Clone)]
        struct DynamicArrayStruct {
            value: Dynamic<[u8; 255]>,
        }
        
        #[derive(Debug, PartialEq, UavcanStruct, Clone)]
        struct TestStruct {
            t1: DynamicArrayStruct, // this array should not be tail array optimized (should encode length)
            t2: DynamicArrayStruct, // this array should be tail array optimized (should not encode length)
        }
        
        assert_eq!(DynamicArrayStruct::FLATTENED_FIELDS_NUMBER, 256);
        assert_eq!(TestStruct::FLATTENED_FIELDS_NUMBER, 512);
        
        let dynamic_array_struct = DynamicArrayStruct{value: Dynamic::<[u8; 255]>::with_data(&[4u8, 5u8, 6u8])};
        
        let test_struct = TestStruct{
            t1: dynamic_array_struct.clone(),
            t2: dynamic_array_struct.clone(),
        };
        
        let mut deserializer: Deserializer<TestStruct> = Deserializer::new(true);
        deserializer.deserialize(&mut [3, 4, 5, 6, 4, 5, 6]);
        let parsed_struct = deserializer.into_structure().unwrap();
        
        assert_eq!(parsed_struct, test_struct);                   

    }

    #[test]
    fn deserialize_static_array() {

        #[derive(Debug, PartialEq, Clone, UavcanStruct, Default)]
        struct Message {
            a: [u16; 4],
        }

        let mut deserializer: Deserializer<Message> = Deserializer::new(true);
        deserializer.deserialize(&mut [5, 0, 6, 0, 7, 0, 8, 0]);
        let parsed = deserializer.into_structure().unwrap();

        assert_eq!(parsed,
                   Message{
                       a: [5, 6, 7, 8],
                   }
        );
        
    }


    #[test]
    fn dynamic_array_of_structs() {
        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        pub struct Command {
            pub actuator_id: u8,
            pub command_type: u8,
            pub command_value: f16,
        }
        
        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        pub struct ArrayCommand {
            pub commands: Dynamic<[Command; 15]>,
        }

        let mut actuator_command = Command {
            actuator_id: 0,
            command_type: 3,
            command_value: f16::from_f32(1.0),
        };
        
        let mut actuator_message = ArrayCommand {
            commands: Dynamic::<[Command; 15]>::new(),
        };

        actuator_message.commands.push(actuator_command.clone());
        
        actuator_command.actuator_id = 1;
        actuator_message.commands.push(actuator_command);
        
        let mut deserializer: Deserializer<ArrayCommand> = Deserializer::new(true);
        deserializer.deserialize(&mut [0, 3, f16::from_f32(1.0).as_bits() as u8, (f16::from_f32(1.0).as_bits() >> 8) as u8, 1, 3, (f16::from_f32(1.0).as_bits() as u8), (f16::from_f32(1.0).as_bits() >> 8) as u8]);
        
        assert_eq!(deserializer.into_structure().unwrap(), actuator_message);                   

    }

    #[test]
    fn static_array_of_structs() {
        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        pub struct Command {
            pub actuator_id: u8,
            pub command_type: u8,
            pub command_value: f16,
        }
        
        #[derive(Debug, PartialEq, Clone, UavcanStruct)]
        pub struct ArrayCommand {
            pub commands: [Command; 2],
        }

        let mut actuator_command1 = Command {
            actuator_id: 0,
            command_type: 3,
            command_value: f16::from_f32(1.0),
        };

        let actuator_command0 = actuator_command1.clone();
        actuator_command1.actuator_id = 1;
        
        let actuator_message = ArrayCommand {
            commands: [actuator_command0, actuator_command1],
        };
        
        let mut deserializer: Deserializer<ArrayCommand> = Deserializer::new(true);
        deserializer.deserialize(&mut [0, 3, f16::from_f32(1.0).as_bits() as u8, (f16::from_f32(1.0).as_bits() >> 8) as u8, 1, 3, (f16::from_f32(1.0).as_bits() as u8), (f16::from_f32(1.0).as_bits() >> 8) as u8]);
        assert_eq!(deserializer.into_structure().unwrap(), actuator_message);                   
        
    }



}
