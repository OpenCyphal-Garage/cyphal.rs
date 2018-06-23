//! Everything that can be changed by changing protocol version.

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum ProtocolVersion {
    Version0 = 0,
}

impl ProtocolVersion {
    /// Returns true if the version number is odd
    pub(crate) fn is_odd(&self) -> bool {
        *self as u8 % 2 == 1
    }
}
