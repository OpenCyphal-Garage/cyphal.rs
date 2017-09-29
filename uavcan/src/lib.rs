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

mod uavcan {
    #[allow(unused_imports)]
    pub use *;
}

pub use uavcan_derive::*;

pub mod transfer;
#[macro_use]
mod header_macros;
pub mod types;
mod crc;
mod deserializer;
pub mod frame_assembler;
mod serializer;
pub mod frame_disassembler;

use bit_field::BitField;

use lib::core::ops::Range;

use transfer::TransferFrameID;

pub use serializer::{
    SerializationResult,
    SerializationBuffer,        
};

pub use deserializer::{
    DeserializationResult,
    DeserializationBuffer,
};


pub trait Header {
    fn from_id(TransferFrameID) -> Result<Self, ()> where Self: Sized;
    
    fn id(&self) -> TransferFrameID;
    fn set_priority(&mut self, priority: u8);
    fn get_priority(&self) -> u8;
}

pub trait MessageFrameHeader : Header {
    const TYPE_ID: u16;
    
    fn new(priority: u8, source_node: u8) -> Self;
}

pub trait AnonymousFrameHeader : Header {
    const TYPE_ID: u8;
    
    fn new(priority: u8, discriminator: u16) -> Self;
}

pub trait ServiceFrameHeader : Header {
    const TYPE_ID: u8;
    
    fn new(priority: u8, request: bool, source_node: u8, destination_node: u8) -> Self;
}





pub trait Struct {
    const TAIL_ARRAY_OPTIMIZABLE: bool;
    const FLATTENED_FIELDS_NUMBER: usize;

    fn bit_length(&self) -> usize;
    fn serialize(&self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn deserialize(&mut self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;
}





pub struct DynamicArrayLength {
    bit_length: usize,
    pub current_length: usize,
}

pub trait DynamicArray {
    fn length_bit_length() -> usize where Self: Sized;
    
    fn length(&self) -> DynamicArrayLength;
    fn set_length(&mut self, length: usize);

    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;

}


impl DynamicArrayLength {
    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {
        let type_bits_remaining = self.bit_length - *bit;
        let buffer_bits_remaining = buffer.bits_remaining();

        if buffer_bits_remaining >= type_bits_remaining {
            buffer.push_bits(type_bits_remaining, self.current_length.get_bits((*bit as u8)..(self.bit_length as u8)) as u64);
            *bit = self.bit_length;
            SerializationResult::Finished
        } else {
            buffer.push_bits(buffer_bits_remaining, self.current_length.get_bits((*bit as u8)..(*bit + buffer_bits_remaining) as u8) as u64);
            *bit += buffer_bits_remaining;
            SerializationResult::BufferFull
        }
    }

    fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
        let buffer_len = buffer.bit_length();
        if buffer_len + *bit < self.bit_length {
            self.current_length.set_bits(*bit as u8..(*bit+buffer_len) as u8, buffer.pop_bits(buffer_len) as usize);
            *bit += buffer_len;
            DeserializationResult::BufferInsufficient
        } else {
            self.current_length.set_bits(*bit as u8..self.bit_length as u8, buffer.pop_bits(self.bit_length-*bit) as usize);
            *bit += self.bit_length;
            DeserializationResult::Finished
        }
    }
        
}







pub trait PrimitiveType {
    fn bit_length() -> usize where Self: Sized;
    fn get_bits(&self, range: Range<usize>) -> u64;
    fn set_bits(&mut self, range: Range<usize>, value: u64);
    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;

}






pub trait Frame {
    type Header : Header;
    type Body : Struct;

    const DATA_TYPE_SIGNATURE: u64;
    
    fn from_parts(header: Self::Header, body: Self::Body) -> Self;
    fn to_parts(self) -> (Self::Header, Self::Body);
    fn header(&self) -> &Self::Header;
    fn body(&self) -> &Self::Body;
}








#[cfg(test)]
mod tests {

    use *;

    // Implementing some types common for several tests
    
    #[derive(Debug, PartialEq)]
    pub struct CanFrame {
        pub id: TransferFrameID,
        pub dlc: usize,
        pub data: [u8; 8],
    }

    impl transfer::TransferFrame for CanFrame {
        fn with_data(id: TransferFrameID, data: &[u8]) -> CanFrame {
            let mut can_data = [0; 8];
            can_data[0..data.len()].clone_from_slice(data);
            CanFrame{id: id, dlc: data.len(), data: can_data}
        }

        fn with_length(id: TransferFrameID, length: usize) -> CanFrame {
            CanFrame{id: id, dlc: length, data: [0; 8]}
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
        
        fn id(&self) -> TransferFrameID {
            self.id 
        }
    }

    
    
}
