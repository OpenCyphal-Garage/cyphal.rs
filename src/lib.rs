#![no_std]

#[macro_use]
extern crate uavcan_indexable_derive;

extern crate bit;

mod can_frame;
mod types;
mod crc;
mod parser;
mod message_builder;

use core::iter::Iterator;
use core::convert::{From, Into};

use bit::BitIndex;

/// The TransportFrame is uavcan cores main interface to the outside world
///
/// This will in >99% of situations be a CAN2.0B frame
/// But in theory both CAN-FD and other protocols which gives
/// similar guarantees as CAN can also be used
pub trait TransportFrame {
    fn get_tail_byte(&self) -> TailByte {
        TailByte::from(*self.get_data().last().unwrap())
    }
    fn is_start_frame(&self) -> bool {
        self.get_tail_byte().start_of_transfer
    }
    fn is_end_frame(&self) -> bool {
        self.get_tail_byte().end_of_transfer
    }
    fn is_single_frame(&self) -> bool {
        self.is_end_frame() && self.is_start_frame()
    }

    /// with_data(id: u32, data: &[u]) -> TransportFrame creates a TransportFrame
    /// with an 28 bits ID and data between 0 and the return value ofget_max_data_length()
    fn with_data(id: u32,  data: &[u8]) -> Self;
    fn get_max_data_length(&self) -> usize;
    fn get_data(&self) -> &[u8];
    fn get_id(&self) -> u32;
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

pub trait TransportFrameHeader {
    fn to_id(&self) -> u32;
    fn from_id(u32) -> Self;
    fn set_priority(&mut self, priority: u8);
    fn get_priority(&self) -> u8;
}

pub trait UavcanIndexable {
    fn number_of_primitive_fields(&self) -> usize;
    fn primitive_field_as_mut(&mut self, field_number: usize) -> Option<&mut UavcanPrimitiveField>;
    fn primitive_field(&self, field_number: usize) -> Option<&UavcanPrimitiveField>;
}


/// An UavcanPrimitiveField is a field of a flatted out uavcan struct
///
/// It's a superset of Primitive Data Types from the uavcan protocol
/// also containing both constant and variable size arrays.
///
/// All primitive data types have 1 primitive fields,
/// All composite data structures have the same number of primtiive fields
/// as the sum of their members. Except the variable length array.
/// This array has number of primitive fields as their members (elements)+1
pub trait UavcanPrimitiveField{
    fn is_constant_size(&self) -> bool;
    /// get_size(&self) -> usize returns the number of primitive data types in this field
    ///
    /// for primtiive data types (non-array) it will return 1
    fn get_size(&self) -> usize;
    /// get_size_mut(&self) -> Option<&mut usize> returns a mutable reference to the size
    /// if the field is of variable size, or None if the field is constant size 
    fn get_size_mut(&self) -> Option<&mut usize>;
    fn primitive_type_as_mut(&mut self, index: usize) -> Option<&mut UavcanPrimitiveType>;
    fn primitive_type(&self, index: usize) -> Option<&UavcanPrimitiveType>;
}

pub trait UavcanPrimitiveType{
    fn bitlength(&self) -> usize;
    fn set_from_bytes(&mut self, buffer: &[u8]);
}

#[derive(Default)]
struct MessageFrameHeader {
    priority: u8,
    type_id: u16,
    source_node: u8,
}

#[derive(Default)]
struct AnonymousFrameHeader {
    priority: u8,
    discriminator: u16,
    type_id: u8,
}

#[derive(Default)]
struct ServiceFrameHeader {
    priority: u8,
    type_id: u8,
    request_not_response: bool,
    destination_node: u8,
    source_node: u8,
}

impl TransportFrameHeader for MessageFrameHeader {
    fn to_id(&self) -> u32 {
        return ((self.priority as u32) << 24)&(0x1f000000) | ((self.type_id as u32) << 8)&(0x00ffff00) | (0u32 << 7) | ((self.source_node as u32))&(0x0000007f);
    }
    fn from_id(id: u32) -> Self {
        Self{
            priority: id.bit_range(24..29) as u8,
            type_id: id.bit_range(8..24) as u16,
            source_node: id.bit_range(0..7) as u8,
        }
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bit_range(0..5, priority);
    }
    fn get_priority(&self) -> u8 {
        self.priority.bit_range(0..5)
    }    
}

impl TransportFrameHeader for AnonymousFrameHeader {
    fn to_id(&self) -> u32 {
        return ((self.priority as u32) << 24)&(0x1f000000) | ((self.discriminator as u32) << 10)&(0x00fffc00) | ((self.type_id as u32) << 10)&(0x00000300) | (0u32 << 7);
    }
    fn from_id(id: u32) -> Self {
        Self{
            priority: id.bit_range(24..29) as u8,
            type_id: id.bit_range(8..10) as u8,
            discriminator: id.bit_range(10..24) as u16,
        }
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bit_range(0..5, priority);
    }
    fn get_priority(&self) -> u8{
        self.priority.bit_range(0..5)
    }
}

impl TransportFrameHeader for ServiceFrameHeader {
    fn to_id(&self) -> u32 {
        return ((self.priority as u32) << 24)&(0x1f000000) | ((self.type_id as u32) << 16)&(0x00ff0000) | ((self.request_not_response as u32) << 15) | ((self.destination_node as u32) << 8)&(0x00007f00) | (1u32 << 7) | ((self.source_node as u32) << 0)&(0x0000007f);
    }
    fn from_id(id: u32) -> Self {
        Self{
            priority: id.bit_range(24..29) as u8,
            type_id: id.bit_range(16..24) as u8,
            request_not_response: id.bit(15),
            destination_node: id.bit_range(8..14) as u8,
            source_node: id.bit_range(0..7) as u8,
        }
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bit_range(0..5, priority);
    }
    fn get_priority(&self) -> u8{
        self.priority.bit_range(0..5)
    }
}

#[derive(Default)]
pub struct UavcanFrame<H: TransportFrameHeader, B: UavcanIndexable> {
    header: H,
    body: B,
}

impl<H: TransportFrameHeader, B: UavcanIndexable> UavcanFrame<H, B> {
    fn from_parts(header: H, body: B) -> Self{
        UavcanFrame{header: header, body: body}
    }    
    fn get_header(&self) -> &H {
        &self.header
    }
    fn get_structure(&self) -> &B {
        &self.body
    }
    fn get_structure_as_mut(&mut self) -> &mut B {
        &mut self.body
    }

}




#[cfg(test)]
mod tests {
    use core::fmt::*;
    use crc;

    use {
        UavcanIndexable,
        UavcanPrimitiveField,
        UavcanPrimitiveType,
    };
    
    use types::{
        Uint2,
        Uint3,
        Uint16,
        Uint32,
    };
    
    #[test]
    fn uavcan_sized_length_derivation() {

        #[derive(UavcanIndexable)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }
        
        impl NodeStatus {
            fn new() -> NodeStatus{
                NodeStatus {
                    uptime_sec: 0.into(),
                    health: 0.into(),
                    mode: 0.into(),
                    sub_mode: 0.into(),
                    vendor_specific_status_code: 0.into(),
                }
            }
        }

        #[derive(UavcanIndexable)]
        struct TestComposite {
            ns1: NodeStatus,
            ns2: NodeStatus,
        }

        impl TestComposite {
            fn new() -> TestComposite {
                TestComposite{
                    ns1: NodeStatus::new(),
                    ns2: NodeStatus::new(),
                }
            }
        }

        #[derive(UavcanIndexable)]
        struct TestComposite2 {
            ns1: NodeStatus,
            tc: TestComposite,
            ns2: NodeStatus,
        }

        impl TestComposite2 {
            fn new() -> TestComposite2 {
                TestComposite2{
                    ns1: NodeStatus::new(),
                    tc: TestComposite::new(),
                    ns2: NodeStatus::new(),
                }
            }
        }

        
        assert_eq!(NodeStatus::new().number_of_primitive_fields(), 5);
        assert_eq!(TestComposite::new().number_of_primitive_fields(), 10);
        assert_eq!(TestComposite2::new().number_of_primitive_fields(), 20);
        
        
    }

    #[test]
    fn uavcan_index_primitive_field() {

        #[derive(UavcanIndexable)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }


        impl NodeStatus {
            fn new() -> NodeStatus{
                NodeStatus {
                    uptime_sec: 0.into(),
                    health: 0.into(),
                    mode: 0.into(),
                    sub_mode: 0.into(),
                    vendor_specific_status_code: 0.into(),
                }
            }
        }
        
        
        let mut node_status = NodeStatus::new();

        node_status.primitive_field_as_mut(0).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[1, 0, 0, 0]);
        node_status.primitive_field_as_mut(1).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[2]);
        node_status.primitive_field_as_mut(2).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[3]);
        node_status.primitive_field_as_mut(3).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[4]);
        node_status.primitive_field_as_mut(4).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[5, 0]);

        node_status.health.primitive_field_as_mut(0).unwrap().primitive_type_as_mut(0).unwrap().set_from_bytes(&[2, 0, 0, 0]);

        assert_eq!(node_status.uptime_sec, 1.into());
        assert_eq!(node_status.health, 2.into());
        assert_eq!(node_status.mode, 3.into());
        assert_eq!(node_status.sub_mode, 4.into());
        assert_eq!(node_status.vendor_specific_status_code, 5.into());

    }



    
    
}

