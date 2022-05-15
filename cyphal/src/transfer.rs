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

#[derive(Debug)]
pub struct TransferMetadata<C: embedded_time::Clock> {
    // for tx -> transmission_timeout
    pub timestamp: Timestamp<C>,
    pub priority: Priority,
    pub transfer_kind: TransferKind,
    pub port_id: PortId,
    pub remote_node_id: Option<NodeId>,
    pub transfer_id: TransferId,
}

pub trait Transfer<'a, C: embedded_time::Clock> {
    fn metadata(&'a self) -> &'a TransferMetadata<C>;
    fn payload(&self) -> &'a [u8];
}

/// Application representation of a UAVCAN transfer.
///
/// This will be passed out on successful reception of full transfers,
/// as well as given to objects to encode into the correct transport.
#[derive(Debug)]
pub struct RefTransfer<'a, C: embedded_time::Clock> {
    pub metadata: TransferMetadata<C>,
    pub payload: &'a [u8],
}

// I don't want to impl convert::From because I need to pull in extra data
impl<'a, C: embedded_time::Clock> RefTransfer<'a, C> {
    pub fn from_frame(
        frame: InternalRxFrame<C>,
        timestamp: Timestamp<C>,
        payload: &'a [u8],
    ) -> Self {
        Self {
            metadata: TransferMetadata {
                timestamp,
                priority: frame.priority,
                transfer_kind: frame.transfer_kind,
                port_id: frame.port_id,
                remote_node_id: frame.source_node_id,
                transfer_id: frame.transfer_id,
            },
            payload,
        }
    }
}

impl<'a, C: embedded_time::Clock> Transfer<'a, C> for RefTransfer<'a, C> {
    fn metadata(&'a self) -> &'a TransferMetadata<C> { &self.metadata }
    fn payload(&self) -> &'a [u8] { self.payload }
}


/// Experimental extra transfer type to 
pub struct ManagedTransfer<C: embedded_time::Clock> {
    pub metadata: TransferMetadata<C>,
    payload: *mut [u8],
    callback: alloc::boxed::Box<dyn Fn()>,
}

impl<C: embedded_time::Clock> ManagedTransfer<C> {
    /// SAFETY: any caller *must* ensure that the payload object lives at least until
    /// callback has returned. This is *very* hard to enforce, because synchronization
    /// primitives will always exit their critical sections before the callback returns,
    /// leaving a small gap. Please don't use this.
    ///
    /// The only known good instance is using this with embassy's Channel, because it uses
    /// a mutex, and the specifics of its executor implementation means that the waker will just
    /// return to the task calling the closure. This may be true for any async stuff running in the
    /// same executor, but I would have to spend time to verify that.
    // TODO probably shouldn't need 'static here
    pub unsafe fn from_ref_transfer(transfer: RefTransfer<C>, callback: alloc::boxed::Box<dyn Fn()>) -> Self {
        ManagedTransfer {
            metadata: TransferMetadata {
                timestamp: transfer.metadata.timestamp,
                priority: transfer.metadata.priority,
                transfer_kind: transfer.metadata.transfer_kind,
                port_id: transfer.metadata.port_id,
                remote_node_id: transfer.metadata.remote_node_id,
                transfer_id: transfer.metadata.transfer_id,
            },
            // Unsafe bit, "upgrading lifetime"
            payload: core::mem::transmute(transfer.payload),
            callback: callback,
        }
    }
}

impl<'a, C: embedded_time::Clock> Transfer<'a, C> for ManagedTransfer<C> {
    fn metadata(&'a self) -> &'a TransferMetadata<C> { &self.metadata }
    fn payload(&self) -> &'a [u8] {
        unsafe { core::mem::transmute(self.payload) }
    }
}

impl<C: embedded_time::Clock> Drop for ManagedTransfer<C> {
    fn drop(&mut self) {
        (self.callback)();
    }
}
