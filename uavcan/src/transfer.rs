//! This module describes the transport-agnostic concept of a transfer,
//! which boils down to some metadata to uniquely identify it, as well
//! as a serialized buffer of data, which encodes DSDL-based data.

use crate::internal::InternalRxFrame;
use crate::types::*;
use crate::Priority;

/// Protocol-level transfer types.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum TransferKind {
    Message,
    Response,
    Request,
}

/// Application representation of a UAVCAN transfer.
///
/// This will be passed out on successful reception of full transfers,
/// as well as given to objects to encode into the correct transport.
#[derive(Clone, Debug)]
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
    pub fn from_frame(frame: InternalRxFrame, timestamp: Timestamp, payload: &[u8]) -> Self {
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
