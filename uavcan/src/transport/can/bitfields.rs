//! # UAVCAN/CAN bitfield primitives.
//!
//! These types describe the bit patterns of both the CAN ID and tail bytes.
//! As well as providing convenient constructor/accessor functions these types
//! are able to do some of the more basic checks that they are valid.

use bitfield::bitfield;
use num_traits::ToPrimitive;

use crate::types::*;
use crate::Priority;

bitfield! {
    /// Structure declaring bitfields of a message frame.
    #[derive(Copy, Clone, Debug)]
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
        let is_anon = source_id.is_none();
        let source_id = match source_id {
            Some(id) => id,
            // TODO do better than XKCD 221
            None => 4,
        };
        let mut id = CanMessageId(0);
        id.set_priority(priority.to_u8().unwrap());
        id.set_svc(false);
        id.set_anon(is_anon);
        id.set_subject_id(subject_id);
        // TODO set as random
        id.set_source_id(source_id);
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
    /// Structure declaring bitfields of a service frame.
    #[derive(Copy, Clone, Debug)]
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

    // TODO valid check
}

bitfield! {
    /// Tail byte of frame data. Received at end of every frame.
    #[derive(Copy, Clone, Debug)]
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
        let mut byte = TailByte(0);
        byte.set_start_of_transfer(is_start);
        byte.set_end_of_transfer(is_end);
        byte.set_toggle(toggle);
        byte.set_transfer_id(transfer_id);
        byte.0
    }
}