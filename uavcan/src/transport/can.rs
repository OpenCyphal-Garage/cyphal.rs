//! UAVCAN/CAN transport implementation.
//!
//! Untested.

use bitfield::bitfield;
use num_traits::{ToPrimitive, FromPrimitive};
use arrayvec::ArrayVec;

use crate::types::*;
use crate::Priority;

pub const MTU_SIZE: usize = 8;

use super::Transport;
use crate::session::SessionManager;
use crate::RxError;
use crate::internal::InternalRxFrame;
use crate::TransferKind;
use crate::NodeId;

/// Unit struct for declaring transport type
struct Can;

// I don't like that I have to do this.
// *Not* doing this would rely on GAT's 
impl<S: SessionManager> crate::Node<S, Can> {
    fn transmit<'a>(transfer: &'a crate::transfer::Transfer) -> CanIter<'a> {
        CanIter::new(
            transfer,
            Some(1),
        )
    }
}

impl Transport for Can {
    type Frame = CanFrame;


    fn rx_process_frame<'a>(node_id: &Option<NodeId>, frame: &'a Self::Frame) -> Result<Option<InternalRxFrame<'a>>, RxError> {
        // Frames cannot be empty. They must at least have a tail byte.
        // NOTE: libcanard specifies this as only for multi-frame transfers but uses
        // this logic.
        if frame.payload.len() == 0 {
            return Err(RxError::FrameEmpty);
        }

        // Pull tail byte from payload
        let tail_byte = TailByte(*frame.payload.last().unwrap());

        // Protocol version states SOT must have toggle set
        if tail_byte.start_of_transfer() && !tail_byte.toggle() {
            return Err(RxError::TransferStartMissingToggle);
        }
        // Non-last frames must use the MTU fully
        if tail_byte.end_of_transfer() && frame.payload.len() < MTU_SIZE {
            return Err(RxError::NonLastUnderUtilization);
        }

        if CanServiceId(frame.id).is_svc() {
            // Handle services
            let id = CanServiceId(frame.id);

            // Ignore frames not meant for us
            if node_id.is_none() || id.destination_id() != node_id.unwrap() {
                return Ok(None);
            }

            let transfer_kind = if id.is_req() {
                TransferKind::Request
            } else {
                TransferKind::Response
            };

            return Ok(Some(InternalRxFrame::as_service(
                frame.timestamp,
                Priority::from_u8(id.priority()).unwrap(),
                transfer_kind,
                id.service_id(),
                id.source_id(),
                id.destination_id(),
                tail_byte.transfer_id(),
                tail_byte.start_of_transfer(),
                tail_byte.end_of_transfer(),
                tail_byte.toggle(),
                &frame.payload,
            )));
        } else {
            // Handle messages
            let id = CanMessageId(frame.id);

            // We can ignore ID in anonymous transfers
            let source_node_id = if id.is_anon() {
                // Anonymous transfers can only be single-frame transfers
                if !(tail_byte.start_of_transfer() && tail_byte.end_of_transfer()) {
                    return Err(RxError::AnonNotSingleFrame);
                }

                Some(id.source_id())
            } else {
                None
            };

            if !id.valid() {
                return Err(RxError::InvalidCanId);
            }

            return Ok(Some(InternalRxFrame::as_message(
                frame.timestamp,
                Priority::from_u8(id.priority()).unwrap(),
                id.subject_id(),
                source_node_id,
                tail_byte.transfer_id(),
                tail_byte.start_of_transfer(),
                tail_byte.end_of_transfer(),
                tail_byte.toggle(),
                &frame.payload,
            )));
        }
    }

    //fn transmit<'a>(transfer: &'t crate::transfer::Transfer) -> CanIter<'a> {
    //    CanIter::new(transfer, Some(1))
    //}
}

struct CanIter<'a> {
    transfer: &'a crate::transfer::Transfer,
    frame_id: u32,
    payload_offset: usize,
    crc: crc_any::CRCu16,
    crc_left: u8,
    toggle: bool,
    is_start: bool,
}

impl<'a> CanIter<'a> {
    fn new(transfer: &'a crate::transfer::Transfer, node_id: Option<NodeId>) -> Self {
        // TODO return errors here, e.g. if anon but sending service message
        let frame_id = match transfer.transfer_kind {
            TransferKind::Message => {

                CanMessageId::new(
                    transfer.priority,
                    transfer.port_id,
                    node_id
                ).to_u32().unwrap()
            }
            TransferKind::Request => {
                CanServiceId::new(
                    transfer.priority,
                    true,
                    transfer.port_id,
                    transfer.remote_node_id.unwrap(),
                    node_id.unwrap()
                ).to_u32().unwrap()
            }
            TransferKind::Response => {
                CanServiceId::new(
                    transfer.priority,
                    false,
                    transfer.port_id,
                    transfer.remote_node_id.unwrap(),
                    node_id.unwrap()
                ).to_u32().unwrap()
            }
        };

        Self {
            transfer,
            frame_id,
            payload_offset: 0,
            crc: crc_any::CRCu16::crc16ccitt_false(),
            crc_left: 2,
            toggle: true,
            is_start: true,
        }
    }
}

impl<'a> Iterator for CanIter<'a> {
    type Item = CanFrame;

    // I'm sure I could take an optimization pass at the logic here
    fn next(&mut self) -> Option<Self::Item> {
        let mut frame = CanFrame {
            timestamp: std::time::Instant::now(),
            id: self.frame_id,
            payload: ArrayVec::new(),
        };

        let bytes_left = self.transfer.payload.len() - self.payload_offset;
        let is_end = bytes_left <= 7;
        let copy_len = core::cmp::min(bytes_left, 7);

        if self.is_start && is_end {
            // Single frame transfer, no CRC
            frame.payload.copy_from_slice(&self.transfer.payload[0..copy_len]);
            self.payload_offset += bytes_left;
            unsafe {
                frame.payload.push_unchecked(TailByte::new(
                    true,
                    true,
                    true,
                    self.transfer.transfer_id
                ).to_u8().unwrap())
            }
        } else {
            // Nothing left to transmit, we are done
            if bytes_left == 0 && self.crc_left == 0 {
                return None;
            }

            // Handle CRC
            let out_data = &self.transfer.payload[self.payload_offset..self.payload_offset + copy_len];
            self.crc.digest(out_data);
            frame.payload.copy_from_slice(out_data);

            // Finished with our data, now we deal with crc
            // (we can't do anything if bytes_left == 7, so ignore that case)
            if bytes_left < 7 {
                let crc = &self.crc.get_crc().to_be_bytes();
                if self.crc_left == 2 {
                    if 7 - bytes_left >= 2 {
                        // Iter doesn't work. Internal type is &u8 but extend
                        // expects u8
                        frame.payload.push(crc[0]);
                        frame.payload.push(crc[1]);
                        self.crc_left = 0;
                    } else {
                        unsafe {
                            frame.payload.push_unchecked(crc[0]);
                        }
                    }
                    match frame.payload.try_extend_from_slice(crc) {
                        Ok(()) => self.crc_left = 0,
                        Err(_) => self.crc_left -= 1,
                    }
                } else if self.crc_left == 1 {
                    unsafe {
                        frame.payload.push_unchecked(crc[1]);
                    }
                }
            }

            // Advance state of iter
            self.toggle = !self.toggle;
        }

        self.is_start = false;

        Some(frame)
    }

    // TODO impl size_hint for frames remaining
}

// TODO convert to embedded-hal PR type
/// Extended CAN frame (the only one supported by UAVCAN/CAN)
pub struct CanFrame {
    pub timestamp: Timestamp,
    pub id: u32,
    pub payload: ArrayVec<[u8; 8]>,
}

/// Keeps track of toggle bit and CRC during frame processing.
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
        let is_anon = source_id.is_none();
        let source_id = match source_id {
            Some(id) => id,
            // chosen at random :)
            // TODO
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