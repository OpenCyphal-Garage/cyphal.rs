//! Configurable options relating to compilation

use std::str::FromStr;

/// Makes certain things in the compilation process configurable. `CompileConfig::default()` is generally safe to use.
pub struct CompileConfig {
    /// Compile data type signatures for types `#[DataTypeSignature = "0x12345678"]`
    pub data_type_signature: bool,
    
    /// Sets strategy for deriving the `Default` trait
    pub derive_default: DeriveDefault,
}

impl Default for CompileConfig {
    fn default() -> CompileConfig {
        CompileConfig {
            data_type_signature: false,
            derive_default: DeriveDefault::default(),
        }
    }
}

/// Strategy for deriving the `Default` trait
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeriveDefault {

    /// Derive default for structs only when the struct consists of primitive types or arrays of primitive types.
    PrimitiveTypes,
}

pub enum ParseDeriveDefaultError {
    NotVariant,
}

impl Default for DeriveDefault {
    fn default() -> DeriveDefault {
        DeriveDefault::PrimitiveTypes
    }
}

impl FromStr for DeriveDefault {
    type Err = ParseDeriveDefaultError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s{
            "primitive-types" => Ok(DeriveDefault::PrimitiveTypes),
            _ => Err(ParseDeriveDefaultError::NotVariant),
        }
    }
}
