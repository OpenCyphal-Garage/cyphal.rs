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

/// This module is only exposed so `Struct` can be derived.
/// It is not intended for use outside the derive macro and
/// must not be considered as a stable part of the API.
#[doc(hidden)]
pub use uavcan_derive::*;

pub mod transfer;
pub mod types;
pub mod framing;
mod crc;
mod header;
pub mod versioning;
mod deserializer;
mod serializer;
pub mod node;

pub use node::NodeConfig;
pub use node::NodeID;
pub use node::Node;
pub use node::SimpleNode;

use header::{
    Header,
    MessageHeader,
    AnonymousHeader,
    ServiceHeader,
};

use versioning::{
    ProtocolVersion,
};

/// These data type is only exposed so `Struct` can be derived.
/// It is not intended for use outside the derive macro and
/// must not be considered as a stable part of the API.
#[doc(hidden)]
pub use serializer::{
    SerializationResult,
    SerializationBuffer,
};
/// These data type is only exposed so `Struct` can be derived.
/// It is not intended for use outside the derive macro and
/// must not be considered as a stable part of the API.
#[doc(hidden)]
pub use deserializer::{
    DeserializationResult,
    DeserializationBuffer,
};

/// The trait that needs to be implemented for all types that will be sent over Uavcan
///
/// The (de)serialization is based on flattening all structures to primitive fields
/// The serializer will then iterate over all fields and bits
pub trait Serializable {
    
    /// The minimum bit length an uavcan type can have
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate uavcan;
    /// # use uavcan::Struct;
    /// # use uavcan::types::*;
    /// # use uavcan::Serializable;
    ///
    /// # fn main() {
    /// // The primitive types have a fixed amount of bits
    /// assert_eq!(u2::BIT_LENGTH_MIN, 2);
    ///
    /// // The static arrays also have a fixed amount of bits
    /// assert_eq!(<[i62; 4] as Serializable>::BIT_LENGTH_MIN, 62*4);
    /// 
    /// // The dynamic arrays have their length coding included even though they can be optimized in some cases
    /// assert_eq!(Dynamic::<[void11; 3]>::BIT_LENGTH_MIN, 2);
    ///
    /// // Structs have the sum of all fields `MIN_BIT_LENGTH` as their `MIN_BIT_LENGTH`.
    /// #[derive(UavcanStruct)]
    /// struct Foo {
    ///     v1: u2,
    ///     v2: [i62; 4],
    ///     v3: Dynamic<[void11; 3]>,
    /// }
    ///
    /// assert_eq!(Foo::BIT_LENGTH_MIN, 2 + 62*4 + 2);
    ///
    /// // Enums have the minimum of all variants `MIN_BIT_LENGTH` as their `MIN_BIT_LENGTH`.
    /// // (But this is not implemented yet)
    ///
    /// # }
    /// ```
    const BIT_LENGTH_MIN: usize;
    
    /// Number of primitive fields after flattening of data type.
    ///
    /// Flattening of a struct consists of replacing all structs with its fields.
    /// Flattening of an enum consists of putting all fields in order
    ///
    /// # Examples
    /// ## Flattening of struct
    /// ```
    /// # #[macro_use]
    /// # extern crate uavcan;
    /// # use uavcan::Struct;
    /// # use uavcan::Serializable;
    /// #[derive(UavcanStruct)]
    /// struct InnerStruct {
    ///     v1: u8,
    ///     v2: u8,
    /// }
    ///
    /// #[derive(UavcanStruct)]
    /// struct OuterStruct {
    ///     v1: InnerStruct,
    ///     v2: InnerStruct,
    /// }
    ///
    /// # fn main() {
    /// assert_eq!(InnerStruct::FLATTENED_FIELDS_NUMBER, 2);
    /// assert_eq!(OuterStruct::FLATTENED_FIELDS_NUMBER, 4);
    /// # }
    /// ```
    /// ## Flattening of enum
    /// ```
    /// # #[macro_use]
    /// # extern crate uavcan;
    /// # use uavcan::Struct;
    /// # use uavcan::Serializable;
    /// #[derive(UavcanStruct)]
    /// enum InnerEnum {
    ///     V1(u8),
    ///     V2(u8),
    /// }
    ///
    /// #[derive(UavcanStruct)]
    /// enum OuterEnum {
    ///     V1(InnerEnum),
    ///     V2(InnerEnum),
    /// }
    ///
    /// # fn main() {
    /// assert_eq!(InnerEnum::FLATTENED_FIELDS_NUMBER, 2);
    /// assert_eq!(OuterEnum::FLATTENED_FIELDS_NUMBER, 4);
    /// # }
    /// ```
    const FLATTENED_FIELDS_NUMBER: usize;

    fn serialize(
        &self,
        flattened_field: &mut usize,
        bit: &mut usize,
        optimize_tail_array: bool,
        buffer: &mut SerializationBuffer
    ) -> SerializationResult;

    fn deserialize(
        &mut self,
        flattened_field: &mut usize,
        bit: &mut usize,
        optimize_tail_array: bool,
        buffer: &mut DeserializationBuffer
    ) -> DeserializationResult;
}

pub trait Struct: Sized + Serializable {
    const DSDL_SIGNATURE: u64;
    const DATA_TYPE_SIGNATURE: u64;
}

pub trait Message: Struct {
    const TYPE_ID: Option<u16>;
}

pub trait Request: Struct {
    type RESPONSE: Response;
    const TYPE_ID: Option<u8>;
}

pub trait Response: Struct {
    type REQUEST: Request;
    const TYPE_ID: Option<u8>;
}

#[derive(Debug, PartialEq)]
pub(crate) struct Frame<T: Struct> {
    header: Header,
    body: T,
}

impl<T: Struct> Frame<T> {

    
    pub fn from_message(message: T, priority: u8, protocol_version: ProtocolVersion, source_node: NodeID) -> Self where T: Message {
        if let Some(type_id) = T::TYPE_ID {
            Frame::from_parts(
                Header::Message(MessageHeader::new(priority, protocol_version.is_odd(), type_id, source_node)),
                message,
            )
        } else {
            unimplemented!("Resolvation of type id is not supported yet")
        }
    }

    pub fn from_anonymous_message(message: T, priority: u8, protocol_version: ProtocolVersion, discriminator: u16) -> Self where T: Message {
        if let Some(type_id) = T::TYPE_ID {
            Frame::from_parts(
                Header::Anonymous(AnonymousHeader::new(priority, protocol_version.is_odd(), type_id as u8, discriminator)),
                message,
            )
        } else {
            unimplemented!("Resolvation of type id is not supported yet")
        }

    }

    pub fn from_request(request: T, priority: u8, protocol_version: ProtocolVersion, source_node: NodeID, destination_node: NodeID) -> Self where T: Request{
        if let Some(type_id) = T::TYPE_ID {
            Frame::from_parts(
                Header::Service(ServiceHeader::new(priority, protocol_version.is_odd(), type_id, source_node, destination_node, true)),
                request,
            )
        } else {
            unimplemented!("Resolvation of type id is not supported yet")
        }

    }

    pub fn from_response(response: T, priority: u8, protocol_version: ProtocolVersion, source_node: NodeID, destination_node: NodeID) -> Self where T: Response {
        if let Some(type_id) = T::TYPE_ID {
            Frame::from_parts(
                Header::Service(ServiceHeader::new(priority, protocol_version.is_odd(), type_id, source_node, destination_node, false)),
                response,
            )
        } else {
            unimplemented!("Resolvation of type id is not supported yet")
        }

    }

    
    fn from_parts(header: Header, body: T) -> Self {
        Frame{header: header, body: body}
    }
    
    fn into_parts(self) -> (Header, T) {
        (self.header, self.body)
    }
}






#[cfg(test)]
mod tests {

    use *;
    use transfer::TransferFrameID;

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
