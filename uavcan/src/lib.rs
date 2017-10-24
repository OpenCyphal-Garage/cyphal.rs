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
#![cfg_attr(feature="clippy", deny(manual_memcpy))]
#![cfg_attr(feature="clippy", deny(unreadable_literal))]
#![cfg_attr(feature="clippy", deny(if_same_then_else))]
#![cfg_attr(feature="clippy", deny(needless_bool))]
#![cfg_attr(feature="clippy", deny(assign_op_pattern))]
#![cfg_attr(feature="clippy", deny(needless_return))]
#![cfg_attr(feature="clippy", deny(doc_markdown))]

#[allow(unused_imports)]
#[macro_use]
extern crate uavcan_derive;

extern crate bit_field;
extern crate embedded_types;
extern crate ux;
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
pub mod types;
mod crc;
mod deserializer;
mod frame_assembler;
mod serializer;
mod frame_disassembler;
mod node;

use bit_field::BitField;

use transfer::TransferFrameID;


pub use node::NodeConfig;
pub use node::NodeID;
pub use node::Node;
pub use node::SimpleNode;


pub use serializer::{
    SerializationResult,
    SerializationBuffer,        
};

pub use deserializer::{
    DeserializationResult,
    DeserializationBuffer,
};

pub trait Struct: Sized {
    const TAIL_ARRAY_OPTIMIZABLE: bool;
    const FLATTENED_FIELDS_NUMBER: usize;

    const DSDL_SIGNATURE: u64;
    const DATA_TYPE_SIGNATURE: u64;

    fn bit_length(&self) -> usize;
    fn serialize(&self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn deserialize(&mut self, flattened_field: &mut usize, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;
}

pub trait Message: Struct {
    const TYPE_ID: u16;

    fn id(priority: u8, source_node: NodeID) -> TransferFrameID {
        let mut id = 0;
        id.set_bits(0..7, u32::from(source_node));
        id.set_bit(7, false);
        id.set_bits(8..24, u32::from(Self::TYPE_ID));
        id.set_bits(24..29, u32::from(priority));

        TransferFrameID::new(id)
    }

    fn id_anonymous(priority: u8, discriminator: u16) -> TransferFrameID {
        let mut id = 0;
        id.set_bits(0..7, 0);
        id.set_bit(7, false);
        id.set_bits(8..10, u32::from(Self::TYPE_ID));
        id.set_bits(10..24, u32::from(discriminator));
        id.set_bits(24..29, u32::from(priority));
        TransferFrameID::new(id)
    }
}

pub trait Request: Struct {
    type RESPONSE: Response;
    const TYPE_ID: u8;

    fn id(priority: u8, source_node: NodeID, destination_node: NodeID) -> TransferFrameID {
        let mut id = 0;
        id.set_bits(0..7, u32::from(source_node));
        id.set_bit(7, false);
        id.set_bits(8..15, u32::from(destination_node));
        id.set_bit(15, true);
        id.set_bits(16..24, u32::from(Self::TYPE_ID));
        id.set_bits(24..29, u32::from(priority));
        TransferFrameID::new(id)
    }
}

pub trait Response: Struct {
    type REQUEST: Request;
    const TYPE_ID: u8;

    fn id(priority: u8, source_node: NodeID, destination_node: NodeID) -> TransferFrameID {
        let mut id = 0;
        id.set_bits(0..7, u32::from(source_node));
        id.set_bit(7, false);
        id.set_bits(8..15, u32::from(destination_node));
        id.set_bit(15, true);
        id.set_bits(16..24, u32::from(Self::TYPE_ID));
        id.set_bits(24..29, u32::from(priority));
        TransferFrameID::new(id)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Frame<T: Struct> {
    id: TransferFrameID,
    body: T,
}

impl<T: Struct> Frame<T> {

    
    pub fn from_message(message: T, priority: u8, source_node: NodeID) -> Self where T: Message {
        Frame::from_parts(
            T::id(priority, source_node),
            message,
        )
    }

    pub fn from_anonymous_message(message: T, priority: u8, discriminator: u16) -> Self where T: Message {
        Frame::from_parts(
            T::id_anonymous(priority, discriminator),
            message,
        )
    }

    pub fn from_request(request: T, priority: u8, source_node: NodeID, destination_node: NodeID) -> Self where T: Request{
        Frame::from_parts(
            T::id(priority, source_node, destination_node),
            request,
        )
    }

    pub fn from_response(response: T, priority: u8, source_node: NodeID, destination_node: NodeID) -> Self where T: Response {
        Frame::from_parts(
            T::id(priority, source_node, destination_node),
            response,
        )
    }

    
    fn from_parts(id: TransferFrameID, body: T) -> Self {
        Frame{id: id, body: body}
    }
    
    fn into_parts(self) -> (TransferFrameID, T) {
        (self.id, self.body)
    }
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
        const MAX_DATA_LENGTH: usize = 8;
        
        fn new(id: TransferFrameID) -> CanFrame {
            CanFrame{id: id, dlc: 0, data: [0; 8]}
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
