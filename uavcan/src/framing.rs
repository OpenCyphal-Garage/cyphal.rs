#[derive(Debug, PartialEq, Eq)]
pub enum DeframingResult {
    Ok,
    Finished,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeframingError {
    FirstFrameNotStartFrame,
    FrameAfterEndFrame,
    IDError,
    ToggleError,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuildError {
    CRCError,
    NotFinishedParsing,
}