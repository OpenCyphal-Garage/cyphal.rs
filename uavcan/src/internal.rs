//! Internal types for passing data around. Overly verbose
//! and not useful to the user, thus not publicly visible.

use crate::time::Timestamp;
use crate::transfer::TransferKind;
use crate::types::*;
use crate::Priority;

/// Internal representation of a received frame.
///
/// This is public so externally-defined SessionManagers can use it.
#[derive(Debug)]
pub struct InternalRxFrame<'a, C: embedded_time::Clock> {
    pub timestamp: Timestamp<C>,
    pub priority: Priority,
    pub transfer_kind: TransferKind,
    pub port_id: PortId,
    pub source_node_id: Option<NodeId>,
    pub destination_node_id: Option<NodeId>,
    pub transfer_id: TransferId,
    pub is_svc: bool,
    pub start_of_transfer: bool,
    pub end_of_transfer: bool,
    pub payload: &'a [u8],
}

impl<'a, C: embedded_time::Clock> InternalRxFrame<'a, C> {
    /// Construct internal frame as a message type
    pub fn as_message(
        timestamp: Timestamp<C>,
        priority: Priority,
        subject_id: PortId,
        source_node_id: Option<NodeId>,
        transfer_id: TransferId,
        start: bool,
        end: bool,
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
            payload,
        }
    }

    /// Construct internal frame as a service type
    pub fn as_service(
        timestamp: Timestamp<C>,
        priority: Priority,
        transfer_kind: TransferKind,
        service_id: PortId,
        source_node_id: NodeId,
        destination_node_id: NodeId,
        transfer_id: TransferId,
        start: bool,
        end: bool,
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
            payload,
        }
    }
}
