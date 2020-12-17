mod can;

use crate::internal::InternalRxFrame;

pub trait SessionMetadata {
    /// Create a fresh instance of session metadata;
    fn new() -> Self;

    /// Update metadata with incoming frame's information
    fn update(&mut self, frame: &InternalRxFrame);

    // TODO I think I need to add some validate function here or something
    // Probably a final version run at the end of a transfer, and an
    // intermediate one on each frame
}
