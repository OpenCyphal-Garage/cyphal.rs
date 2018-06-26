//! Everything that can be changed by changing protocol version.

pub mod version0;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum ProtocolVersion {
    Version0 = 0,
    Version1 = 1,
}

/// The protocol compatibility specifier
///
/// We split the uavcan protocol versions into two categories. Odd and even.
/// The Uavcan frame header contains a protocol version bit.
/// This bit tells if the encoding results from a odd or even protocol.
///
/// The default protocol can either be odd or even. We call this version parity.
/// This `ProtocolCompatibility` type contains the strategy of what to do when a received message do not have the same version parity as the default protocol in use.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum ProtocolCompatibility {
    /// Refuse to deframe transfer frames where parity doesn't match the default protocol version.
    None,

    /// Deframe transfer frames accoding to the one version newer protocol version when parity doesn't match the default protocol version.
    /// If the one version newer protocol doesn't exist yet, behave equal to `ProtocolCompatibility::None`.
    Newer,

    /// Deframe transfer frames accoding to the one version older protocol version when parity doesn't match the default protocol version.
    /// If the one version older protocol doesn't exist (this is version 0), behave equal to `ProtocolCompatibility::None`.
    Older,
}

impl ProtocolVersion {
    /// Returns true if the version number is odd
    pub(crate) fn is_odd(&self) -> bool {
        *self as u8 % 2 == 1
    }
}
