//! Everything related to the uavcan comment `Comment`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// A comment
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment(String);

impl AsRef<str> for Comment {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Errors that may occur when parsing a `Comment`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseCommentError {
    NoStartingHash,
    ContainsEol,
}

impl FromStr for Comment {
    type Err = ParseCommentError;

    fn from_str(s: &str) -> Result<Comment, Self::Err> {
        if !s.starts_with("#") {
            Err(ParseCommentError::NoStartingHash)
        } else if s.contains("\n") || s.contains("\r") {
            Err(ParseCommentError::ContainsEol)
        } else {
            Ok(Comment(String::from(&s[1..])))
        }
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}
