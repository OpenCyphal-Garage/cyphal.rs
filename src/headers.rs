use bit_field::BitField;

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
        let mut id = 0;
        id.set_bits(0..7, self.source_node as u32);
        id.set_bit(7, false);
        id.set_bits(8..24, self.type_id as u32);
        id.set_bits(24..29, self.priority as u32);
        return id;
    }
    fn from_id(id: u32) -> Self {
        Self{
            priority: id.get_bits(24..29) as u8,
            type_id: id.get_bits(8..24) as u16,
            source_node: id.get_bits(0..7) as u8,
        }
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bits(0..5, priority);
    }
    fn get_priority(&self) -> u8 {
        self.priority.get_bits(0..5)
    }    
}

impl UavcanHeader for AnonymousFrameHeader {
    fn to_id(&self) -> u32 {
        let mut id = 0;
        id.set_bits(0..7, 0);
        id.set_bit(7, false);
        id.set_bits(8..10, self.type_id as u32);
        id.set_bits(10..24, self.discriminator as u32);
        id.set_bits(24..29, self.priority as u32);
        return id;
    }
    fn from_id(id: u32) -> Self {
        Self{
            priority: id.get_bits(24..29) as u8,
            type_id: id.get_bits(8..10) as u8,
            discriminator: id.get_bits(10..24) as u16,
        }
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bits(0..5, priority);
    }
    fn get_priority(&self) -> u8{
        self.priority.get_bits(0..5)
    }
}

impl UavcanHeader for ServiceFrameHeader {
    fn to_id(&self) -> u32 {
        let mut id = 0;
        id.set_bits(0..7, self.source_node as u32);
        id.set_bit(7, true);
        id.set_bits(8..15, self.destination_node as u32);
        id.set_bit(15, self.request_not_response);
        id.set_bits(16..24, self.type_id as u32);
        id.set_bits(24..29, self.priority as u32);
        return id;
    }
    fn from_id(id: u32) -> Self {
        Self{
            priority: id.get_bits(24..29) as u8,
            type_id: id.get_bits(16..24) as u8,
            request_not_response: id.get_bit(15),
            destination_node: id.get_bits(8..14) as u8,
            source_node: id.get_bits(0..7) as u8,
        }
    }
    fn set_priority(&mut self, priority: u8) {
        self.priority.set_bits(0..5, priority);
    }
    fn get_priority(&self) -> u8{
        self.priority.get_bits(0..5)
    }
}
