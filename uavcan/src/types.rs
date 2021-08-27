//! # Placeholder module for various types I'm not sure how to sort yet.

// Not too sure what to do with these. This is the best option to keep them all seperate.

// TODO unsure of how to manage this as a generic concept.
//      They are both transport-specific types, and I would like to make them
//      work nicely with that, but I can't think of any good API. Also does
//      saving a couple bytes here really matter that much vs. just making it
//      the max size anyways?
pub type NodeId = u16;
pub type PortId = u16;

// TODO set type with min/max bounds
pub type TransferId = u8;

// TODO whole timing overhaul
pub type Timestamp = std::time::Instant;
