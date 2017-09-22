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

#[macro_use]
mod header_macros;
pub mod types;
mod crc;
mod deserializer;
pub mod frame_assembler;
mod serializer;
pub mod frame_disassembler;

use bit_field::BitField;

use lib::core::convert::{From};
use lib::core::ops::Range;

pub use serializer::{
    SerializationResult,
    SerializationBuffer,        
};

use deserializer::{
    DeserializationResult,
    DeserializationBuffer,
};


/// The TransportFrame is uavcan cores main interface to the outside world
///
/// This will in >99% of situations be a CAN2.0B frame
/// But in theory both CAN-FD and other protocols which gives
/// similar guarantees as CAN can also be used
pub trait TransferFrame {
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






pub trait Header {
    fn from_id(u32) -> Result<Self, ()> where Self: Sized;
    
    fn id(&self) -> u32;
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
    current_length: usize,
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

    use {
        TransferFrame,
        Struct,
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

    impl TransferFrame for CanFrame {
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
