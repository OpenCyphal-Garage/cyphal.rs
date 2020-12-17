//! # UAVCAN implementation
//!
//! My first implementation is very specifically a std-based CAN-transport
//! implementation. Organization of modules is poor right now, but as I
//! refactor to add generic capabilities it will improve.

#[macro_use]
extern crate num_derive;

pub mod transfer;
pub mod transport;
pub mod types;

pub use node::Node;
pub use transfer::{TransferKind};
pub use transport::CanFrame;

mod session;
mod internal;
mod node;

use types::*;
use session::Session;
use std::collections::HashMap;

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
    /// Internal SessionManager error
    SessionError(session::SessionError),
}

// TODO could replace with custom impl's to reduce dependencies
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
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
        }
    }
}

impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        return self.transfer_kind == other.transfer_kind && self.port_id == other.port_id;
    }
}
