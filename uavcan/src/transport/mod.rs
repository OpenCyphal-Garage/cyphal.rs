//! Transport-specific functionality.
//!
//! For now I'm only supporting CAN bus, but TBD is more transports.
//!
//! Theoretically to implement this you should only have to implement two
//! types, one that impls SessionMetadata, and one that impls a yet-to-be
//! made trait. We'll see how that shakes out though.
mod can;

use crate::internal::InternalRxFrame;
use crate::session::SessionManager;
use crate::RxError;

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

// TODO does this go with Node or stay here?
/// Trait to be implemented for Node, declaring a transport implementation.
pub trait Transport {
    type Frame;

    // TODO not sure if I can use lifetimes in my impls properly
    /// Process a frame, returning the internal transport-independant representation,
    /// or errors if invalid.
    fn rx_process_frame<'a, S: SessionManager>(node: &mut crate::node::Node<S, Self>, frame: Self::Frame) -> Result<Option<InternalRxFrame<'a>>, RxError>;
}
