//! Everything related to the uavcan cast mode `CastMode`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// Cast mode defines the rules of conversion from the native value of a certain programming language to the serialized field value.
///
/// Cast mode may be left undefined, in which case the default will be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastMode {
    /// This is the default cast mode, which will be used if the attribute definition does not specify the cast mode explicitly.
    ///
    /// For integers, it prevents an integer overflow - for example, attempting to write 0x44 to a 4-bit field will result in a bitfield value of 0x0F.
    /// For floating point values, it prevents overflow when casting to a lower precision floating point representation -
    /// for example, 65536.0 will be converted to a float16 as 65504.0; infinity will be preserved.
    Saturated,

    ///  For integers, it discards the excess most significant bits - for example, attempting to write 0x44 to a 4-bit field will produce 0x04.
    /// For floating point values, overflow during downcasting will produce an infinity.
    Truncated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseCastModeError {
    NotCastMode(String),
}

impl FromStr for CastMode {
    type Err = ParseCastModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "saturated" => Ok(CastMode::Saturated),
            "truncated" => Ok(CastMode::Truncated),
            _ => Err(ParseCastModeError::NotCastMode(String::from(s))),
        }
    }
}

impl Display for CastMode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            CastMode::Saturated => write!(f, "saturated"),
            CastMode::Truncated => write!(f, "truncated"),
        }
    }
}
