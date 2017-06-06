use bit::BitIndex;

use {
    UavcanHeader
};

#[derive(Default, PartialEq, Debug)]
pub struct MessageFrameHeader {
    priority: u8,
    type_id: u16,
    source_node: u8,
}

#[derive(Default, PartialEq, Debug)]
pub struct AnonymousFrameHeader {
    priority: u8,
    discriminator: u16,
    type_id: u8,
}

#[derive(Default, PartialEq, Debug)]
pub struct ServiceFrameHeader {
    priority: u8,
    type_id: u8,
    request_not_response: bool,
    destination_node: u8,
    source_node: u8,
}

impl UavcanHeader for MessageFrameHeader {
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

impl UavcanHeader for AnonymousFrameHeader {
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

impl UavcanHeader for ServiceFrameHeader {
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
