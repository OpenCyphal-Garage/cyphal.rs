use transfer::{
    TransferFrame,
    TransferID,
};

#[derive(Debug, PartialEq, Eq)]
pub enum DeframingResult<T> {
    Ok,
    Finished(T),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeframingError {
    FirstFrameNotStartFrame,
    FrameAfterEndFrame,
    IDError,
    ToggleError,
    CRCError,
}

/// The `Framer` takes a `UavcanFrame` (`uavcan::Frame`) and turns it into multiple `TransferFrame`s.
pub trait Framer<S: ::Struct> {
    fn new(uavcan_structure: ::Frame<S>, transfer_id: TransferID) -> Self;
    fn next_frame<T: TransferFrame>(&mut self) -> Option<T>;
}

/// The `Deframer` takes multiple `TransferFrame`s and turn they into a  `UavcanFrame` (`uavcan::Frame`).
pub trait Deframer<S: ::Struct> {
    fn new() -> Self;
    fn add_frame<T: TransferFrame>(&mut self, frame: T) -> Result<DeframingResult<::Frame<S>>, DeframingError>;
}