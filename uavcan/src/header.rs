//! Types related to the Uavcan frame header.

use bit_field::BitField;

use node::NodeID;

use transfer::TransferFrameID;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Header {
    Message(MessageHeader),
    Anonymous(AnonymousHeader),
    Service(ServiceHeader),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MessageHeader(u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AnonymousHeader(u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ServiceHeader(u32);

impl MessageHeader {
    pub fn new(priority: u8, odd_protocol_version: bool, message_type_id: u16, source_node: NodeID) -> Self {
        let mut id = 0;
        id.set_bits(0..7, u32::from(source_node));
        id.set_bit(7, false);
        id.set_bits(8..24, u32::from(message_type_id));
        id.set_bit(24, odd_protocol_version);
        id.set_bits(25..29, u32::from(priority));
        MessageHeader(id)
    }
}

impl AnonymousHeader {
    pub fn new(priority: u8, odd_protocol_version: bool, message_type_id: u8, discriminator: u16) -> Self {
        let mut id = 0;
        id.set_bit(7, false);
        id.set_bits(8..10, u32::from(message_type_id));
        id.set_bits(10..24, u32::from(discriminator));
        id.set_bit(24, odd_protocol_version);
        id.set_bits(25..29, u32::from(priority));
        AnonymousHeader(id)
    }
}

impl ServiceHeader {
    pub fn new(priority: u8, odd_protocol_version: bool, service_type_id: u8, source_node: NodeID, destination_node: NodeID, request_not_response: bool) -> Self {
        let mut id = 0;
        id.set_bits(0..7, u32::from(source_node));
        id.set_bit(7, true);
        id.set_bits(8..15, u32::from(destination_node));
        id.set_bit(15, request_not_response);
        id.set_bits(16..24, u32::from(service_type_id));
        id.set_bit(24, odd_protocol_version);
        id.set_bits(25..29, u32::from(priority));
        ServiceHeader(id)
    }
}

impl From<MessageHeader> for Header {
    fn from(h: MessageHeader) -> Header {
        Header::Message(h)
    }
}

impl From<AnonymousHeader> for Header {
    fn from(h: AnonymousHeader) -> Header {
        Header::Anonymous(h)
    }
}

impl From<ServiceHeader> for Header {
    fn from(h: ServiceHeader) -> Header {
        Header::Service(h)
    }
}

impl From<MessageHeader> for u32 {
    fn from(h: MessageHeader) -> u32 {
        h.0
    }
}

impl From<AnonymousHeader> for u32 {
    fn from(h: AnonymousHeader) -> u32 {
        h.0
    }
}

impl From<ServiceHeader> for u32 {
    fn from(h: ServiceHeader) -> u32 {
        h.0
    }
}

impl From<Header> for u32 {
    fn from(h: Header) -> u32 {
        match h {
            Header::Message(mh) => mh.into(),
            Header::Anonymous(ah) => ah.into(),
            Header::Service(sh) => sh.into(),
        }
    }
}

// We know that TrasferFrameID is a legit header as it's checked on construction.
impl From<TransferFrameID> for Header {
    fn from(id: TransferFrameID) -> Header {
        // TODO: When TryFrom/TryInto is stabilized piggyback on `TryFrom<u32> for Header`.
        let id_u32 = u32::from(id);
        if id_u32.get_bit(7) {
            Header::Service(ServiceHeader(id_u32))
        } else if id_u32.get_bits(0..7) == 0 {
            Header::Anonymous(AnonymousHeader(id_u32))
        } else {
            Header::Message(MessageHeader(id_u32))
        }
    }
}

impl From<Header> for TransferFrameID {
    fn from(h: Header) -> TransferFrameID {
        TransferFrameID::new(u32::from(h))
    }
}

