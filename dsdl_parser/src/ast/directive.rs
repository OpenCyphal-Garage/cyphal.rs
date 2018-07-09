//! Everything related to the uavcan directive `Directive`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use ast::ident::Ident;

/// An Uavcan Directive
///
/// A directive is a single case-sensitive word starting with an “at sign” (@),
/// possibly followed by space-separated arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Directive {
    /// This directive instructs the DSDL compiler that the current message or the current part of a service data type (request or response) is a tagged union.
    /// A tagged union is a data structure that may encode either of its fields at a time.
    /// Such a data structure contains one implicit field, a union tag that indicates what particular field the data structure is holding at the moment.
    /// Unions are required to have at least two fields.
    Union,

    /// A marker variant that tells the compiler that users of this enum cannot match it exhaustively.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Directive {
    pub(crate) fn try_from(ident: Ident) -> Result<Self, ParseDirectiveError> {
        // Until TryFrom trait is stabilized
        match ident.as_ref() {
            "union" => Ok(Directive::Union),
            _ => Err(ParseDirectiveError::NotDirective(String::from(ident))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseDirectiveError {
    NotDirective(String),
}

impl FromStr for Directive {
    type Err = ParseDirectiveError;

    fn from_str(s: &str) -> Result<Directive, Self::Err> {
        match s {
            "@union" => Ok(Directive::Union),
            "union" => Ok(Directive::Union),
            _ => Err(ParseDirectiveError::NotDirective(String::from(s))),
        }
    }
}

impl Display for Directive {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Directive::Union => write!(f, "@union"),
            Directive::__Nonexhaustive => unreachable!("The `_Nonexhaustive` variant should never be created"),
        }
    }
}
