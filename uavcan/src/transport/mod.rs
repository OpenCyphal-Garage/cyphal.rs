//! Transport-specific functionality.
//!
//! For now I'm only supporting CAN bus, but TBD is more transports.
//!
//! Theoretically to implement this you should only have to implement two
//! types, one that impls SessionMetadata, and one that impls a yet-to-be
//! made trait. We'll see how that shakes out though.
mod can;

use crate::internal::InternalRxFrame;
use crate::RxError;
use crate::NodeId;

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
    // TODO unsized issue? not sure how I can specify the type of the node
    /// Process a frame, returning the internal transport-independant representation,
    /// or errors if invalid.
    fn rx_process_frame<'a>(node_id: &Option<NodeId>, frame: &'a Self::Frame) -> Result<Option<InternalRxFrame<'a>>, RxError>;

    // Returns a series of transport frames to be transmitted.
    //
    // This is the only way I know how to transmit over generic transports.
    // The iterator here can be collected() into a higher-level storage
    // type later on. Probably if I want to add any storage, I'll add it
    // in Node.
    //fn transmit<'a>(transfer: &crate::transfer::Transfer) -> Self::FrameIter;
}
