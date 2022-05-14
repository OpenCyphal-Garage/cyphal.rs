//! This module describes the transport-agnostic concept of a transfer,
//! which boils down to some metadata to uniquely identify it, as well
//! as a serialized buffer of data, which encodes DSDL-based data.

use crate::internal::InternalRxFrame;
use crate::time::Timestamp;
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
#[derive(Debug)]
pub struct Transfer<'a, C: embedded_time::Clock> {
    // for tx -> transmission_timeout
    pub timestamp: Timestamp<C>,
    pub priority: Priority,
    pub transfer_kind: TransferKind,
    pub port_id: PortId,
    pub remote_node_id: Option<NodeId>,
    pub transfer_id: TransferId,
    pub payload: &'a [u8],
}

// I don't want to impl convert::From because I need to pull in extra data
impl<'a, C: embedded_time::Clock> Transfer<'a, C> {
    pub fn from_frame(
        frame: InternalRxFrame<C>,
        timestamp: Timestamp<C>,
        payload: &'a [u8],
    ) -> Self {
        Self {
            timestamp,
            priority: frame.priority,
            transfer_kind: frame.transfer_kind,
            port_id: frame.port_id,
            remote_node_id: frame.source_node_id,
            transfer_id: frame.transfer_id,
            payload,
        }
    }
}
