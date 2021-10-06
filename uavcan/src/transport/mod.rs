//! Transport-specific functionality.
//!
//! The current iteration requires 2 different implementations:
//! - SessionMetadata trait
//! - Transport trait
//!
//! Take a look at the CAN implementation for an example.

// Declaring all of the sub transport modules here.
pub mod can;

use crate::internal::InternalRxFrame;
use crate::NodeId;
use crate::{RxError, TxError};

/// Describes any transport-specific metadata required to construct a session.
///
/// In the example of CAN, you need to keep track of the toggle bit,
/// as well as the CRC for multi-frame transfers. This trait lets us pull that
/// code out of the generic processing and into more modular implementations.
pub trait SessionMetadata<C: embedded_time::Clock> {
    /// Create a fresh instance of session metadata;
    fn new() -> Self;

    /// Update metadata with incoming frame's information.
    ///
    /// If the frame is valid, returns Some(length of payload to ingest)
    fn update(&mut self, frame: &InternalRxFrame<C>) -> Option<usize>;

    /// Final check to see if transfer was successful.
    fn is_valid(&self, frame: &InternalRxFrame<C>) -> bool;
}

/// This trait is to be implemented on a unit struct, in order to be specified
/// for different transport types.
pub trait Transport<C: embedded_time::Clock> {
    type Frame;
    type FrameIter<'a>: Iterator;

    /// Max payload which can be transported per frame/packet.
    const MAX_PAYLOAD_SIZE: usize;

    /// Process a frame, returning the internal transport-independant representation,
    /// or errors if invalid.
    fn rx_process_frame<'a>(
        node_id: &Option<NodeId>,
        frame: &'a Self::Frame,
    ) -> Result<Option<InternalRxFrame<'a, C>>, RxError>;

    /// Prepare an iterator of frames to send out on the wire.
    fn transmit<'a>(
        transfer: &'a crate::transfer::Transfer<C>,
    ) -> Result<Self::FrameIter<'a>, TxError>;
}
