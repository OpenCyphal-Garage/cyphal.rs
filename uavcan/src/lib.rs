//! First pass is CAN-only!
//! Work on generic transports later.

// Simplest first: CAN-only single-implementation similar to libcanard.
// feed it frames, get transfers, give it transfer, receive frames

#[macro_use]
extern crate num_derive;

use bitfield::bitfield;
use num_traits::{FromPrimitive, ToPrimitive};
use std::collections::{hash_map::Entry, HashMap};

use crc_any::CRC;

const MTU_SIZE: usize = 8;

// TODO use this to implement no_std static, no_std allocator, and std versions
trait SessionManager {}

// Not too sure what to do with these. This is the best option to keep them all seperate.
pub type NodeId = u8;
pub type PortId = u16;
// TODO set type with min/max bounds
pub type TransferId = u8;

// TODO whole timing overhaul
pub type Timestamp = std::time::Instant;

// Naming things is hard
pub enum RxError {
    TransferStartMissingToggle,
    /// Anonymous transfers must only use a single frame
    AnonNotSingleFrame,
    /// Frames that are not last cannot have less than the maximum MTU
    NonLastUnderUtilization,
    /// No type of frame can contain empty data, must always have at least a tail byte
    FrameEmpty,
    /// Id field is formatted incorrectly
    InvalidCanId,
    /// Non-start frame received without session
    NewSessionNoStart,
    /// Session has expired
    Timeout,
    /// Frame is part of new
    InvalidTransferId,
}

// TODO timestamp type or something

// TODO some nameing for the reserved fields
bitfield! {
    struct CanMessageId(u32);
    pub u8, priority, set_priority: 28, 26;
    pub bool, is_svc, set_svc: 25;
    pub bool, is_anon, set_anon: 24;
    /// Reserved bits. Must be set to 0b011 on transmit. On recieve discard if bit 23 is not 0, discard the rest
    pub u8, reserved, set_reserved: 23, 21;
    pub PortId, subject_id, set_subject_id: 20, 8;
    // Set to 0 and discard if not 0
    pub bool, discard, set_discard: 7;
    pub NodeId, source_id, set_source_id: 6, 0;
}

// TODO bounds checks (can these be auto-implemented?)
impl CanMessageId {
    fn new(priority: Priority, subject_id: PortId, source_id: Option<NodeId>) -> u32 {
        let mut id = CanMessageId(0);
        id.set_priority(priority.to_u8().unwrap());
        id.set_svc(false);
        id.set_anon(source_id.is_none());
        id.set_reserved(0b011);
        id.set_subject_id(subject_id);
        id.set_discard(false);
        // TODO set as random
        id.set_source_id(42);
        id.0
    }
}

bitfield! {
    struct CanServiceId(u32);
    pub u8, priority, set_priority: 28, 26;
    pub bool, is_svc, set_svc: 25;
    pub bool, is_req, set_req: 24;
    // Set to 0, discard if not 0
    pub bool, reserved, set_reserved: 23;
    pub PortId, service_id, set_service_id: 22, 14;
    pub NodeId, destination_id, set_destination_id: 13, 7;
    pub NodeId, source_id, set_source_id: 6, 0;
}

impl CanServiceId {
    fn new(
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
        id.set_reserved(false);
        id.set_service_id(service_id);
        id.set_destination_id(destination);
        id.set_source_id(source);
        id.0
    }
}

bitfield! {
    struct TailByte(u8);
    pub bool, start_of_transfer, set_start_of_transfer: 7;
    pub bool, end_of_transfer, set_end_of_transfer: 6;
    pub bool, toggle, set_toggle: 5;
    pub TransferId, transfer_id, set_transfer_id: 4, 0;
}

pub enum TransferKind {
    Message,
    Response,
    Request,
}

// TODO could replace with custom impl's to reduce dependencies
#[derive(FromPrimitive, ToPrimitive)]
pub enum Priority {
    Exceptional,
    Immediate,
    Fast,
    High,
    Nominal,
    Low,
    Slow,
    Optional,
}

pub struct Subscription {
    transfer_kind: TransferKind,
    port_id: PortId,
    extent: usize,
    timeout: core::time::Duration,

    // "Internal" structs
    sessions: HashMap<NodeId, Session>,
}

impl Subscription {
    pub fn new(
        transfer_kind: TransferKind,
        port_id: PortId,
        extent: usize,
        timeout: core::time::Duration,
    ) -> Self {
        Self {
            transfer_kind,
            port_id,
            extent,
            timeout,
            sessions: HashMap::new(),
        }
    }
}

struct Session {
    // Timestamp of first frame
    pub timestamp: Option<Timestamp>,
    pub total_payload_size: usize,
    pub payload: Vec<u8>,
    pub crc: crc_any::CRCu16,
    pub transfer_id: TransferId,
    pub toggle: bool,
}

impl Session {
    fn new(transfer_id: TransferId) -> Self {
        Self {
            timestamp: None,
            total_payload_size: 0,
            payload: Vec::new(),
            crc: crc_any::CRCu16::crc16ccitt_false(),
            transfer_id,
            toggle: false,
        }
    }

    fn update<'a>(
        &mut self,
        frame: InternalRxFrame<'a>,
        timeout: core::time::Duration,
        extent: usize,
    ) -> Result<Option<Transfer>, RxError> {
        // TODO check transport index

        // Check timeouts for session
        if let Some(last_time) = self.timestamp {
            if frame.timestamp - last_time > timeout {
                // Reset session instance
                *self = Self::new(self.transfer_id);

                if !frame.start_of_transfer {
                    return Err(RxError::Timeout);
                }
            }
        }

        // I have no idea why the diff check is > 1 in libcanard...
        if self.transfer_id != frame.transfer_id {
            *self = Self::new(frame.transfer_id);
            if !frame.start_of_transfer {
                return Err(RxError::InvalidTransferId);
            }
        }

        // Pull in frame
        self.accept_frame(frame, extent)
    }

    fn accept_frame<'a>(
        &mut self,
        frame: InternalRxFrame<'a>,
        extent: usize,
    ) -> Result<Option<Transfer>, RxError> {
        // Timestamp only gets updated from first frame
        if frame.start_of_transfer {
            self.timestamp = Some(frame.timestamp);
        }

        // Update CRC if it isn't a single frame
        // NOTE: payload is evaluated even if data is truncated
        let single_frame = frame.start_of_transfer && frame.end_of_transfer;
        if !single_frame {
            self.crc.digest(frame.payload);
        }

        // Read in payload, truncating
        let payload_to_copy = if self.payload.len() + frame.payload.len() > extent {
            self.payload.len() + frame.payload.len() - extent
        } else {
            frame.payload.len()
        };
        self.payload
            .extend_from_slice(&frame.payload[0..payload_to_copy]);
        self.total_payload_size += frame.payload.len();

        if frame.end_of_transfer {
            let mut out = None;
            // Single frames or when our CRC has completed it's check means we've finished the transfer
            // TODO proper check and error for invalid CRC at end of transfer
            if single_frame || self.crc.get_crc() == 0x0000u16 {
                // Don't pass CRC to the user
                let truncated_size = self.total_payload_size - self.payload.len();
                // If we have not already truncated the CRC, remove it from the output
                let real_payload = if !single_frame && truncated_size < 2 {
                    self.payload.len() - 2 - truncated_size
                } else {
                    self.payload.len()
                };
                out = Some(Transfer::from_frame(
                    frame,
                    self.timestamp.unwrap(),
                    &self.payload[0..real_payload],
                ));
            }
            // TODO maybe use a different function for this reset
            *self = Self::new(self.transfer_id);
            Ok(out)
        } else {
            self.toggle = !self.toggle;
            Ok(None)
        }
    }
}

pub struct Transfer {
    pub timestamp: Timestamp,
    pub priority: Priority,
    pub transfer_kind: TransferKind,
    pub port_id: PortId,
    pub remote_node_id: Option<NodeId>,
    pub transfer_id: TransferId,
    // TODO replace with reference in final memory model
    pub payload: Vec<u8>,
}

// I don't want to impl convert::From because I need to pull in extra data
impl Transfer {
    fn from_frame(frame: InternalRxFrame, timestamp: Timestamp, payload: &[u8]) -> Self {
        Self {
            timestamp: timestamp,
            priority: frame.priority,
            transfer_kind: frame.transfer_kind,
            port_id: frame.port_id,
            remote_node_id: frame.source_node_id,
            transfer_id: frame.transfer_id,
            payload: Vec::from(payload),
        }
    }
}

// TODO convert to embedded-hal PR type
/// Extended CAN frame
pub struct CanFrame<'a> {
    pub timestamp: Timestamp,
    pub id: u32,
    pub payload: &'a [u8],
}

/// Internal representation of a received frame.
struct InternalRxFrame<'a> {
    timestamp: Timestamp,
    priority: Priority,
    transfer_kind: TransferKind,
    port_id: PortId,
    source_node_id: Option<NodeId>,
    destination_node_id: Option<NodeId>,
    transfer_id: TransferId,
    is_svc: bool,
    start_of_transfer: bool,
    end_of_transfer: bool,
    toggle: bool,
    payload: &'a [u8],
}

impl<'a> InternalRxFrame<'a> {
    /// Construct internal frame as a message type
    fn as_message(
        timestamp: Timestamp,
        priority: Priority,
        subject_id: PortId,
        source_node_id: Option<NodeId>,
        transfer_id: TransferId,
        start: bool,
        end: bool,
        toggle: bool,
        payload: &'a [u8],
    ) -> Self {
        Self {
            timestamp,
            priority,
            transfer_kind: TransferKind::Message,
            port_id: subject_id,
            source_node_id,
            destination_node_id: None,
            transfer_id,
            is_svc: false,
            start_of_transfer: start,
            end_of_transfer: end,
            toggle,
            payload,
        }
    }

    /// Construct internal frame as a service type
    fn as_service(
        timestamp: Timestamp,
        priority: Priority,
        transfer_kind: TransferKind,
        service_id: PortId,
        source_node_id: NodeId,
        destination_node_id: NodeId,
        transfer_id: TransferId,
        start: bool,
        end: bool,
        toggle: bool,
        payload: &'a [u8],
    ) -> Self {
        Self {
            timestamp,
            priority,
            transfer_kind,
            port_id: service_id,
            source_node_id: Some(source_node_id),
            destination_node_id: Some(destination_node_id),
            transfer_id,
            is_svc: true,
            start_of_transfer: start,
            end_of_transfer: end,
            toggle,
            payload,
        }
    }
}

pub struct Node {
    // TODO this is transport level type
    id: Option<NodeId>,

    // TODO no-std-ify
    subscriptions: Vec<Subscription>,
}

impl Node {
    pub fn new(id: Option<NodeId>) -> Self {
        Self {
            id,
            subscriptions: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }

    // TODO unsubscribe?

    /// Attempts to process frame. Returns error when unable to parse frame as valid UAVCAN v1
    fn rx_process_frame<'a>(&self, frame: &'a CanFrame) -> Result<InternalRxFrame<'a>, RxError> {
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

            let transfer_kind = if id.is_req() {
                TransferKind::Request
            } else {
                TransferKind::Response
            };

            return Ok(InternalRxFrame::as_service(
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
                frame.payload,
            ));
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

            // We care about these reserved bits. Naming is funky
            if (id.reserved() & 0b100 > 0) || id.discard() {
                return Err(RxError::InvalidCanId);
            }

            return Ok(InternalRxFrame::as_message(
                frame.timestamp,
                Priority::from_u8(id.priority()).unwrap(),
                id.subject_id(),
                source_node_id,
                tail_byte.transfer_id(),
                tail_byte.start_of_transfer(),
                tail_byte.end_of_transfer(),
                tail_byte.toggle(),
                frame.payload,
            ));
        }
    }

    fn rx_accept_frame<'a>(
        &mut self,
        sub: usize,
        frame: InternalRxFrame<'a>,
    ) -> Result<Option<Transfer>, RxError> {
        let subscription = &mut self.subscriptions[sub];

        if let Some(source_node_id) = frame.source_node_id {
            let mut session = match subscription.sessions.entry(source_node_id) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    // We didn't receive the start of transfer frame
                    // Transfers must be sent/received in order.
                    if !frame.start_of_transfer {
                        return Err(RxError::NewSessionNoStart);
                    }

                    // Create a new session
                    entry.insert(Session::new(frame.transfer_id))
                }
            };

            // Call another function to manage session state
            return session.update(frame, subscription.timeout, subscription.extent);
        } else {
            // Truncate payload if subscription specifies a smaller extent
            let payload_size = core::cmp::min(subscription.extent, frame.payload.len() - 1);
            // Anonymous transfer, no need to worry about sessions, return transfer immediately
            return Ok(Some(Transfer {
                timestamp: frame.timestamp,
                priority: frame.priority,
                transfer_kind: frame.transfer_kind,
                port_id: frame.port_id,
                remote_node_id: None,
                transfer_id: frame.transfer_id,
                payload: Vec::from(&frame.payload[0..payload_size]),
            }));
        }
    }

    /// Attempts to receive frame. Returns error when frame is invalid, Some(Transfer) at the end of
    /// a transfer, and None if we haven't finished the transfer.
    pub fn try_receive_frame<'a>(
        &mut self,
        frame: &'a CanFrame,
    ) -> Result<Option<Transfer>, RxError> {
        // TODO check extended ID mask etc.
        let frame = self.rx_process_frame(frame)?;

        for (i, subscription) in self.subscriptions.iter().enumerate() {
            if subscription.port_id == frame.port_id {
                return self.rx_accept_frame(i, frame);
            }
        }

        Ok(None)
    }

    fn transmit(&self, transfer: &Transfer) -> Vec<CanFrame> {
        Vec::new()
    }
}
