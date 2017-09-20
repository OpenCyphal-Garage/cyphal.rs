#![cfg_attr(not(feature="std"), no_std)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![cfg_attr(feature="clippy", deny(almost_swapped))]
#![cfg_attr(feature="clippy", deny(blacklisted_name))]
#![cfg_attr(feature="clippy", deny(bool_comparison))]
#![cfg_attr(feature="clippy", deny(builtin_type_shadow))]
#![cfg_attr(feature="clippy", deny(clone_on_copy))]
#![cfg_attr(feature="clippy", deny(char_lit_as_u8))]
#![cfg_attr(feature="clippy", deny(should_assert_eq))]

#[allow(unused_imports)]
#[macro_use]
extern crate uavcan_derive;

extern crate bit_field;
extern crate half;

mod lib {
    pub mod core {
        #[cfg(feature="std")]
        pub use std::*;
        #[cfg(not(feature="std"))]
        pub use core::*;
    }
}

pub use uavcan_derive::*;

#[macro_use]
mod header_macros;
pub mod types;
mod crc;
mod deserializer;
pub mod message_builder;
mod serializer;
pub mod frame_generator;

use bit_field::BitField;

use lib::core::convert::{From};
use lib::core::ops::Range;

use serializer::{
    SerializationResult,
    SerializationBuffer,        
};


/// The TransportFrame is uavcan cores main interface to the outside world
///
/// This will in >99% of situations be a CAN2.0B frame
/// But in theory both CAN-FD and other protocols which gives
/// similar guarantees as CAN can also be used
pub trait TransportFrame {
    fn tail_byte(&self) -> TailByte {
        TailByte::from(*self.data().last().unwrap())
    }
    fn is_start_frame(&self) -> bool {
        self.tail_byte().start_of_transfer
    }
    fn is_end_frame(&self) -> bool {
        self.tail_byte().end_of_transfer
    }
    fn is_single_frame(&self) -> bool {
        self.is_end_frame() && self.is_start_frame()
    }

    /// with_data(id: u32, data: &[u]) -> TransportFrame creates a TransportFrame
    /// with an 28 bits ID and data between 0 and the return value ofget_max_data_length()
    fn with_data(id: u32,  data: &[u8]) -> Self;
    fn with_length(id: u32, length: usize) -> Self;
    fn set_data_length(&mut self, length: usize);
    fn max_data_length() -> usize;
    fn data(&self) -> &[u8];
    fn data_as_mut(&mut self) -> &mut[u8];
    fn id(&self) -> u32;
}

pub struct TailByte {
    start_of_transfer: bool,
    end_of_transfer: bool,
    toggle: bool,
    transfer_id: u8,
}

impl From<TailByte> for u8 {
    fn from(tb: TailByte) -> u8 {
        ((tb.start_of_transfer as u8) << 7) | ((tb.end_of_transfer as u8) << 6) | ((tb.toggle as u8) << 5) | (tb.transfer_id&0x1f)
    }
}

impl From<u8> for TailByte {
    fn from(u: u8) -> TailByte {
        TailByte{start_of_transfer: (u&(1<<7)) != 0, end_of_transfer: (u&(1<<6)) != 0, toggle: (u&(1<<5)) != 0, transfer_id: u&0x1f}
    }
}

pub trait UavcanHeader : Sized {
    fn type_id() -> u16;
    fn id(&self) -> u32;
    fn from_id(u32) -> Result<Self, ()>;
    fn set_priority(&mut self, priority: u8);
    fn get_priority(&self) -> u8;
}

pub trait MessageFrameHeader : UavcanHeader {
    fn new(priority: u8, source_node: u8) -> Self;
}

pub trait AnonymousFrameHeader : UavcanHeader {
    fn new(priority: u8, discriminator: u16) -> Self;
}

pub trait ServiceFrameHeader : UavcanHeader {
    fn new(priority: u8, request: bool, source_node: u8, destination_node: u8) -> Self;
}





pub trait UavcanStruct {
    fn fields_len(&self) -> usize;    
    fn field(&self, field_number: usize) -> UavcanField;
    fn field_as_mut(&mut self, field_number: usize) -> MutUavcanField;
    
    fn flattened_fields_len(&self) -> usize;
    
    fn flattened_field(&self, field_number: usize) -> UavcanField {
        assert!(field_number < self.flattened_fields_len());
        
        let mut accumulated_fields_len = 0;
        let mut current_field = 0;
        loop {
            let current_field_len = match self.field(current_field) {
                UavcanField::PrimitiveType(_x) => 1,
                UavcanField::DynamicArray(_x) => 1,
                UavcanField::UavcanStruct(x) => x.flattened_fields_len(),                
            };
            
            if accumulated_fields_len + current_field_len > field_number {
                break;
            } else {
                accumulated_fields_len += current_field_len;
                current_field += 1;
            }
        }

        match self.field(current_field) {
            UavcanField::UavcanStruct(x) => x.flattened_field(field_number - accumulated_fields_len),
            x => x,
        }

    }
    
    
    fn flattened_field_as_mut(&mut self, field_number: usize) -> MutUavcanField {
        assert!(field_number < self.flattened_fields_len());
        
        let mut accumulated_fields_len = 0;
        let mut current_field = 0;
        loop {
            let current_field_len = match self.field(current_field) {
                UavcanField::PrimitiveType(_x) => 1,
                UavcanField::DynamicArray(_x) => 1,
                UavcanField::UavcanStruct(x) => x.flattened_fields_len(),                
            };
            
            if accumulated_fields_len + current_field_len > field_number {
                break;
            } else {
                accumulated_fields_len += current_field_len;
                current_field += 1;
            }
            
        }

        match self.field_as_mut(current_field) {
            MutUavcanField::UavcanStruct(x) => x.flattened_field_as_mut(field_number - accumulated_fields_len),
            x => x,
        }
        
    }

}


pub struct DynamicArrayLength {
    bit_length: usize,
    current_length: usize,
}

pub trait DynamicArray : serializer::Serialize + deserializer::Deserialize {
    fn max_size() -> usize where Self: Sized;
    
    fn length_bit_length() -> usize where Self: Sized;
    fn element_bit_length() -> usize where Self: Sized;
    
    fn length(&self) -> DynamicArrayLength;
    fn set_length(&mut self, length: usize);

    fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult;

}


impl DynamicArrayLength {
    fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult {
        let mut bits_serialized: usize = 0;
        
        let mut byte_start = buffer.bit_index / 8;
        let odd_bits_start = buffer.bit_index % 8;
        
        // first get rid of the odd bits
        if odd_bits_start != 0 && 8-odd_bits_start <= self.bit_length - start_bit {
            buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.current_length.get_bits((start_bit as u8)..(start_bit+8-odd_bits_start) as u8) as u8);
            bits_serialized += 8-odd_bits_start;
            buffer.bit_index += 8-odd_bits_start;
            byte_start += 1;
        } else if odd_bits_start != 0 && 8-odd_bits_start > self.bit_length - start_bit {
            buffer.data[byte_start].set_bits((odd_bits_start as u8)..8, self.current_length.get_bits((start_bit as u8)..(start_bit + (self.bit_length - start_bit) ) as u8) as u8);
            bits_serialized += self.bit_length - start_bit;
            buffer.bit_index += self.bit_length - start_bit;
            return SerializationResult::Finished(bits_serialized);
        }
        
        for i in byte_start..buffer.data.len() {
            let serialization_index = bits_serialized + start_bit;
            let remaining_bits = self.bit_length - serialization_index;

            if remaining_bits == 0 {
                return SerializationResult::Finished(bits_serialized);
            } else if remaining_bits <= 8 {
                buffer.data[i] = self.current_length.get_bits((serialization_index as u8)..(serialization_index+remaining_bits) as u8) as u8;
                buffer.bit_index += remaining_bits;
                bits_serialized += remaining_bits;
                return SerializationResult::Finished(bits_serialized);
            } else {
                buffer.data[i] = self.current_length.get_bits((serialization_index as u8)..(serialization_index+8) as u8) as u8;
                buffer.bit_index += 8;
                bits_serialized += 8;
            }
        }
        
        
        SerializationResult::BufferFull(bits_serialized)

    }

}





/// An UavcanField is a field of a flatted out uavcan struct
///
/// It's a superset of Primitive Data Types from the uavcan protocol
/// also containing both constant and variable size arrays.
///
/// All primitive data types have 1 primitive fields,
/// All composite data structures have the same number of primtiive fields
/// as the sum of their members. Except the variable length array.
/// This array has number of primitive fields as their members (elements)+1
pub enum UavcanField<'a>{
    PrimitiveType(&'a UavcanPrimitiveType),
    DynamicArray(&'a DynamicArray),
    UavcanStruct(&'a UavcanStruct),
}

pub enum MutUavcanField<'a>{
    PrimitiveType(&'a mut UavcanPrimitiveType),
    DynamicArray(&'a mut DynamicArray),
    UavcanStruct(&'a mut UavcanStruct),
}

pub trait AsUavcanField {
    fn as_uavcan_field(&self) -> UavcanField; 
    fn as_mut_uavcan_field(&mut self) -> MutUavcanField; 
}








pub trait UavcanPrimitiveType : serializer::Serialize + deserializer::Deserialize {
    fn bit_length() -> usize where Self: Sized;
    fn get_bits(&self, range: Range<usize>) -> u64;
    fn set_bits(&mut self, range: Range<usize>, value: u64);
    fn serialize(&self, start_bit: usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    
}






pub trait UavcanFrame<H: UavcanHeader, B: UavcanStruct> {
    fn from_parts(header: H, body: B) -> Self;
    fn to_parts(self) -> (H, B);
    fn header(&self) -> &H;
    fn body(&self) -> &B;
    fn data_type_signature(&self) -> u64;
}




#[cfg(test)]
mod tests {

    use {
        TransportFrame,
        UavcanStruct,
        UavcanField,
        MutUavcanField,
        AsUavcanField,
    };
    
    use types::*;

    // Implementing some types common for several tests
    
    #[derive(Debug, PartialEq)]
    pub struct CanID(pub u32);

    #[derive(Debug, PartialEq)]
    pub struct CanFrame {
        pub id: CanID,
        pub dlc: usize,
        pub data: [u8; 8],
    }

    impl TransportFrame for CanFrame {
        fn with_data(id: u32, data: &[u8]) -> CanFrame {
            let mut can_data = [0; 8];
            can_data[0..data.len()].clone_from_slice(data);
            CanFrame{id: CanID(id), dlc: data.len(), data: can_data}
        }

        fn with_length(id: u32, length: usize) -> CanFrame {
            CanFrame{id: CanID(id), dlc: length, data: [0; 8]}
        }
        
        fn max_data_length() -> usize {
            8
        }

        fn set_data_length(&mut self, length: usize) {
            assert!(length <= 8);
            self.dlc = length;
        }

        fn data(&self) -> &[u8] {
            &self.data[0..self.dlc]
        }

        fn data_as_mut(&mut self) -> &mut[u8] {
            &mut self.data[0..self.dlc]
        }
        
        fn id(&self) -> u32 {
            match self.id {
                CanID(x) => x,
            }
        }
    }

    
    
}
