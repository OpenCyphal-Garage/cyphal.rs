use bitfield::bitfield;
use num_traits::ToPrimitive;

use crate::types::*;
use crate::Priority;

pub const MTU_SIZE: usize = 8;

// TODO convert to embedded-hal PR type
/// Extended CAN frame
pub struct CanFrame {
    pub timestamp: Timestamp,
    pub id: u32,
    pub payload: Vec<u8>,
}

pub struct CanMetadata {
    toggle: bool,
    crc: crc_any::CRCu16,
}

impl super::SessionMetadata for CanMetadata {
    fn new() -> Self {
        Self {
            // Toggle starts off true, but we compare against the opposite value.
            toggle: false,
            crc: crc_any::CRCu16::crc16ccitt_false(),
        }
    }

    fn update(&mut self, frame: &crate::internal::InternalRxFrame) -> Option<usize> {
        // Single frame transfers don't need to be validated
        if frame.start_of_transfer && frame.end_of_transfer {
            return Some(frame.payload.len());
        }

        // Pull in everything but the tail byte
        self.crc.digest(&frame.payload[0..frame.payload.len() - 2]);
        self.toggle = !self.toggle;

        let toggle = TailByte(frame.payload[frame.payload.len()]).toggle();

        if toggle == self.toggle {
            Some(frame.payload.len() - 1)
        } else {
            None
        }
    }
    
    fn is_valid(&self, frame: &crate::internal::InternalRxFrame) -> bool {
        if frame.start_of_transfer && frame.end_of_transfer {
            return true;
        }

        if self.crc.get_crc() == 0x0000u16 {
            return true;
        }

        return false;
    }
}

bitfield! {
    /// Structure declaring bitfields of a message frame.
    ///
    /// Reserved fields rsvd0 and 3 must be cleared, and the frame
    /// discarded if they aren't. rsvd1 and 2 must be set, but can be
    /// ignored on reception.
    pub struct CanMessageId(u32);
    /// Priority level.
    pub u8, priority, set_priority: 28, 26;
    /// Is this a service?
    pub bool, is_svc, set_svc: 25;
    /// Is this an anonymous message (i.e. no source ID)?
    pub bool, is_anon, set_anon: 24;
    /// Port ID of the message being sent.
    pub PortId, subject_id, set_subject_id: 20, 8;
    /// Node ID of the message's source.
    pub NodeId, source_id, set_source_id: 6, 0;
    /// Reserved field.
    pub bool, rsvd0, set_rsvd0: 23;
    /// Reserved field.
    pub bool, rsvd1, set_rsvd1: 22;
    /// Reserved field.
    pub bool, rsvd2, set_rsvd2: 21;
    /// Reserved field.
    pub bool, rsvd3, set_rsvd3: 7;
}

impl CanMessageId {
    // TODO bounds checks (can these be auto-implemented?)
    pub fn new(priority: Priority, subject_id: PortId, source_id: Option<NodeId>) -> u32 {
        let mut id = CanMessageId(0);
        id.set_priority(priority.to_u8().unwrap());
        id.set_svc(false);
        id.set_anon(source_id.is_none());
        id.set_subject_id(subject_id);
        // TODO set as random
        id.set_source_id(42);
        // Set reserved fields
        id.set_rsvd0(false);
        id.set_rsvd1(true);
        id.set_rsvd2(true);
        id.set_rsvd3(false);
        // Return data
        id.0
    }

    /// Is this a message or a service ID?
    pub fn is_message(&self) -> bool {
        !self.is_svc()
    }

    /// Is this a valid message ID?
    pub fn valid(&self) -> bool {
        if self.is_svc() {
            return false;
        }
        if self.rsvd0() || self.rsvd3() {
            return false;
        }

        return true;
    }
}

bitfield! {
    /// Structure declaring bitfields of a service frame
    ///
    /// Reserved field rsvd0 must be cleared, and the frame discarded if not.
    pub struct CanServiceId(u32);
    /// Priority level.
    pub u8, priority, set_priority: 28, 26;
    /// Is this a service message?
    pub bool, is_svc, set_svc: 25;
    /// Is this a request? (or a response?)
    pub bool, is_req, set_req: 24;
    /// Reserved bit, must be set to 0
    pub bool, rsvd0, set_rsvd0: 23;
    /// Service port ID
    pub PortId, service_id, set_service_id: 22, 14;
    /// Destination node ID
    pub NodeId, destination_id, set_destination_id: 13, 7;
    /// Source node ID
    pub NodeId, source_id, set_source_id: 6, 0;
}

impl CanServiceId {
    pub fn new(
        priority: Priority,
        is_request: bool,
        service_id: PortId,
        destination: NodeId,
        source: NodeId,
    ) -> u32 {
        let mut id = CanServiceId(0);
        id.set_priority(priority.to_u8().unwrap());
        id.set_svc(true);
        id.set_req(is_request);
        id.set_rsvd0(false);
        id.set_service_id(service_id);
        id.set_destination_id(destination);
        id.set_source_id(source);
        id.0
    }
}

bitfield! {
    /// Tail byte of frame data. Received at end of every frame.
    pub struct TailByte(u8);
    /// Is this the start of the transfer?
    pub bool, start_of_transfer, set_start_of_transfer: 7;
    /// Is this the end of the transfer?
    pub bool, end_of_transfer, set_end_of_transfer: 6;
    /// Toggle bit to ensure messages are received in order.
    pub bool, toggle, set_toggle: 5;
    /// Transfer ID to ensure the correct messages are being received.
    pub TransferId, transfer_id, set_transfer_id: 4, 0;
}

impl TailByte {
    pub fn new(is_start: bool, is_end: bool, toggle: bool, transfer_id: u8) -> u8 {
        // TODO am I doing dumb here?
        assert!(transfer_id < 32);

        let mut byte = TailByte(0);
        byte.set_start_of_transfer(is_start);
        byte.set_end_of_transfer(is_end);
        byte.set_toggle(toggle);
        byte.set_transfer_id(transfer_id);
        byte.0
    }
}