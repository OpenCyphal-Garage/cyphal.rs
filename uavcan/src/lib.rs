//! # UAVCAN implementation
//!
//! The intent with this implementation right now is to present a transport
//! and session-management agnostic interface for UAVCAN. What I'm working on
//! here is not meant to implement the higher-level protocol features, such
//! as automatic heartbeat publication. It is simply meant to manage ingesting
//! and producing raw frames to go on the bus. There is room to provide
//! application-level constructs in this crate, but that's not what I'm working
//! on right now.
//!
//! ## Comparison to canadensis
//!
//! The only other Rust UAVCAN implementation with any real progess at the
//! moment is canadensis. I *believe* that it is fully functional but I haven't
//! verified that.
//!
//! canadensis seems to be providing a more specific implementation (CAN-only)
//! that provides more application level features (e.g. a Node w/ Heartbeat
//! publishing) that relies on a global allocator. The intent (or experiment)
//! here is to provide a single unified interface for different transports
//! and storage backends. Application level functionality can live on top of
//! this. I can see issues with this running into issues in multi-threaded
//! environments, but I'll get to those when I get to them.

#[macro_use]
extern crate num_derive;

pub mod transfer;
pub mod transport;
pub mod types;

pub use node::Node;
pub use transfer::{TransferKind};

pub mod session;
mod internal;
mod node;

use types::*;

// TODO handle invalid transport frames.
/// Protocol errors possible from receiving incoming frames.
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
// TODO how could I represent more priorities for different transports?
/// Protocol-level priorities.
///
/// Transports are supposed to be able to support more than these base 8
/// priorities, but there is currently no API for that.
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
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

/// Simple subscription type to
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
