//! Transport-specific functionality.
//!
//! For now I'm only supporting CAN bus, but TBD is more transports.
//!
//! The current iteration requires 3 different implementations:
//! - SessionMetadata trait
//! - Transport trait
//! - impl crate::Node<S, TransportType> { fn transmit() }
//!
//! The last implementation is reuired because I haven't found a way
//! to adequately describe a generic transmit function inside of the
//! Transport trait. I suspect that to do it will require GATs, which
//! aren't stable and may not be for a while. See the CAN implementation
//! for an example of how to implement this.

// Declaring all of the sub transport modules here.
mod can;

use crate::internal::InternalRxFrame;
use crate::RxError;
use crate::NodeId;

/// Describes any transport-specific metadata required to construct a session.
///
/// In the example of CAN, you need to keep track of the toggle bit,
/// as well as the CRC for multi-frame transfers. This trait lets us pull that
/// code out of the generic processing and into more modular implementations.
pub trait SessionMetadata {
    /// Create a fresh instance of session metadata;
    fn new() -> Self;

    /// Update metadata with incoming frame's information.
    ///
    /// If the frame is valid, returns Some(length of payload to ingest)
    fn update(&mut self, frame: &InternalRxFrame) -> Option<usize>;

    /// Final check to see if transfer was successful.
    // TODO maybe this doesn't need a frame?
    fn is_valid(&self, frame: &InternalRxFrame) -> bool;
}

/// This trait is to be implemented on a unit struct, in order to be specified
/// for different transport types.
pub trait Transport {
    type Frame;

    /// Process a frame, returning the internal transport-independant representation,
    /// or errors if invalid.
    fn rx_process_frame<'a>(node_id: &Option<NodeId>, frame: &'a Self::Frame) -> Result<Option<InternalRxFrame<'a>>, RxError>;

    // TODO find a way to specify this function here, may require GATs
    //fn transmit<'a>(transfer: &crate::transfer::Transfer) -> Self::FrameIter;
}
