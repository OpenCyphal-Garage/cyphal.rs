use num_traits::FromPrimitive;
use std::collections::hash_map::Entry;

use crate::types::*;
use crate::RxError;
use crate::internal::InternalRxFrame;
use crate::Subscription;
use crate::transfer::Transfer;
use crate::transport::{CanFrame, CanMessageId, CanServiceId, TailByte, MTU_SIZE};
use crate::{TransferKind, Priority};
use crate::session::Session;

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

            if !id.valid() {
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