//! Everything related to the uavcan idenfitifer `Ident`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// An Identifier (name)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(String);

impl<'a> From<&'a str> for Ident {
    fn from(s: &'a str) -> Ident {
        Ident(String::from(s))
    }
}

impl FromStr for Ident {
    type Err = ();

    fn from_str(s: &str) -> Result<Ident, Self::Err> {
        Ok(Ident(String::from(s)))
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<Ident> for String {
    fn from(i: Ident) -> String {
        i.0
    }
}

impl From<String> for Ident {
    fn from(s: String) -> Ident {
        Ident(s)
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}