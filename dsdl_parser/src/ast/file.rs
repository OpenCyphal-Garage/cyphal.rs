//! Everything related to uavcan file `File`.

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;


use crc::CRC64WE as CRC;

use ast::file_name::FileName;
use ast::type_definition::TypeDefinition;

/// A DSDL file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub name: FileName,
    pub definition: TypeDefinition,
}

/// A normalized `File`
///
/// The underlying file can't be exposed in a mutable way. Any change to the normalized format would "denormalize" it.
/// The `NormalizedFile` can be used to calculate the DSDL signature
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedFile(File);

impl NormalizedFile {
    /// Return a reference to the underlying `File`
    pub fn as_file<'a>(&'a self) -> &'a File {
        &self.0
    }

    /// Turn the `NormalizedFile` into the underlying `File`
    pub fn into_file(self) -> File {
        self.0
    }

    /// Calculate the DSDL signature
    pub fn dsdl_signature(&self) -> u64 {
        let mut crc = CRC::new();
        crc.add(format!("{}", self).as_bytes());
        crc.value()
    }

}

impl File {
    /// Normalizes a file according to the uavcan specification.
    pub fn normalize(self) -> NormalizedFile {
        let definition = self.definition.normalize(&self.name);
        NormalizedFile(File{name: self.name, definition: definition})
    }
}

impl Display for NormalizedFile {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}\n{}", self.as_file().name, self.as_file().definition)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "File: {}\n{}", self.name, self.definition)
    }
}
