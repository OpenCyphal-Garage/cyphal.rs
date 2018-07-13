//! All the parse errors (TODO: Find out if Tokenization errors should be here as well)

use std::fmt;

use Ident;

#[derive(PartialEq, Debug, Clone)]
pub struct ParseError {
    severity: ParseErrorSeverity,
    kind: ParseErrorKind,
}

#[derive(PartialEq, Debug, Clone, Copy, Eq)]
pub enum ParseErrorSeverity {
    Info,
    Warning,
    Error,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParseErrorKind {
    // Errors that may occur during tokenization

    // Errors that may occur during parsing
    UnknownDirectiveName(Ident),
}


impl ParseError {
    /// Create a error with minimum severity
    pub(crate) fn new(kind: ParseErrorKind) -> Self {
        let severity = kind.minimum_severity();
        ParseError {
            kind,
            severity,
        }
    }

}

impl ParseErrorKind {
    pub fn explain(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorKind::UnknownDirectiveName(x) => write!(f, "The identifier \"{}\" is not recognized as a directive", x),
        }
    }

    pub fn minimum_severity(&self) -> ParseErrorSeverity {
        match self {
            ParseErrorKind::UnknownDirectiveName(_) => ParseErrorSeverity::Error,
        }
    }

}