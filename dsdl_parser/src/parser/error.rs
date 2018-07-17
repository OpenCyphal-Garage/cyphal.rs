//! All the parse errors (TODO: Find out if Tokenization errors should be here as well)

use std::fmt;

use lalrpop_util::ErrorRecovery as LalrpopErrorRecovery;
use lalrpop_util::ParseError as LalrpopParseError;

use parser::lexer::Token;
use Ident;

#[derive(PartialEq, Debug, Clone)]
pub struct ParseError {
    severity: ParseErrorSeverity,
    kind: ParseErrorKind,
    location: Option<usize>,
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
    InvalidToken,


    // Errors that may occur during parsing
    UnknownDirectiveName(Ident),

    UnexpectedToken(Token),

    /// Indicate that the attempted parsed type defintiion was invalid.
    /// More information can usually be found in the errors vector.
    InvalidTypeDefinition,
}


impl ParseError {
    /// Create an error with minimum required severity.
    ///
    /// Severity can be "upgraded" at a later point, but never "downgraded".
    pub(crate) fn new(kind: ParseErrorKind, location: Option<usize>) -> Self {
        let severity = kind.minimum_severity();
        ParseError {
            kind,
            severity,
            location,
        }
    }

    pub fn severity(&self) -> ParseErrorSeverity {
        self.severity
    }

}

impl ParseErrorKind {
    pub fn explain(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorKind::InvalidToken => write!(f, "Encountered something that is not recognized as a valid token"),
            ParseErrorKind::UnknownDirectiveName(x) => write!(f, "The identifier \"{}\" is not recognized as a directive", x),
            ParseErrorKind::UnexpectedToken(x) => write!(f, "The token \"{}\" is not expected in this context", x),
            ParseErrorKind::InvalidTypeDefinition => write!(f, "Type definition was not valid"),
        }
    }

    pub fn minimum_severity(&self) -> ParseErrorSeverity {
        match self {
            ParseErrorKind::InvalidToken => ParseErrorSeverity::Error,
            ParseErrorKind::UnknownDirectiveName(_) => ParseErrorSeverity::Error,
            ParseErrorKind::UnexpectedToken(_) => ParseErrorSeverity::Error,
            ParseErrorKind::InvalidTypeDefinition => ParseErrorSeverity::Error,
        }
    }

}

impl From<LalrpopParseError<usize, Token, ParseError>> for ParseError {
    fn from(e: LalrpopParseError<usize, Token, ParseError>) -> Self {
        match e {
            LalrpopParseError::InvalidToken{location} => ParseError::new(ParseErrorKind::InvalidToken, Some(location)),
            LalrpopParseError::UnrecognizedToken{token: Some(token), ..} => ParseError::new(ParseErrorKind::UnexpectedToken(token.1), Some(token.0)),
            LalrpopParseError::UnrecognizedToken{token: None, ..} => unimplemented!("TODO: Find out how to handle an unknown unexpexted token from the parser"),
            LalrpopParseError::User{error} => error,
            _ => unimplemented!("TODO: Fill out conversion between LalrpopParseError and ParseError")
        }
    }
}

impl From<ParseError> for LalrpopParseError<usize, Token, ParseError> {
    fn from(e: ParseError) -> Self {
        LalrpopParseError::User{error: e}
    }
}