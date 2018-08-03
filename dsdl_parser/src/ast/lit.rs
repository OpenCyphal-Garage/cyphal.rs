//! Everything related to the uavcan literal `Lit`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// A sign of a signed literal (dec int, hex int, float etc)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    /// An explicit positive sign (+)
    Positive,
    /// An explicit negative sign (-)
    Negative,
    /// No explicit sign
    Implicit,
}

/// A constant must be a primitive scalar type (i.e., arrays and nested data structures are not allowed as constant types).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lit {

    /// Integer zero (0) or Integer literal in base 10, starting with a non-zero character. E.g., 123, -12.
    Dec{sign: Sign, value: String},

    /// Integer literal in base 16 prefixed with 0x. E.g., 0x123, -0x12, +0x123.
    Hex{sign: Sign, value: String},

    /// Integer literal in base 8 prefixed with 0o. E.g., 0o123, -0o777, +0o777.
    Oct{sign: Sign, value: String},

    /// Integer literal in base 2 prefixed with 0b. E.g., 0b1101, -0b101101, +0b101101.
    Bin{sign: Sign, value: String},

    /// Boolean true or false.
    Bool(bool),

    /// Single ASCII character, ASCII escape sequence, or ASCII hex literal in single quotes. E.g., 'a', '\x61', '\n'.
    Char(String),

    /// Floating point literal. Fractional part with an optional exponent part, e.g., 15.75, 1.575E1, 1575e-2, -2.5e-3, +25E-4. Not-a-number (NaN), positive infinity, and negative infinity are intentionally not supported in order to maximize cross-platform compatibility.
    Float{sign: Sign, value: String},
}

impl Display for Sign {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Sign::Implicit => write!(f, ""),
            Sign::Positive => write!(f, "+"),
            Sign::Negative => write!(f, "-"),
        }
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Lit::Dec{sign: s, value: v} => write!(f, "{}{}", s, v),
            Lit::Hex{sign: s, value: v} => write!(f, "{}0x{}", s, v),
            Lit::Bin{sign: s, value: v} => write!(f, "{}0b{}", s, v),
            Lit::Oct{sign: s, value: v} => write!(f, "{}0o{}", s, v),
            Lit::Bool(x) => write!(f, "{}", match x {true => "true", false => "false"}),
            Lit::Char(x) => write!(f, "'{}'", x),
            Lit::Float{sign: s, value: v} => write!(f, "{}{}", s, v),
        }
    }
}

/// Errors that may occur when parsing `Lit`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseLitError {
    /// String started with "(+|-)0x" and encountered a char that is not a valid hexadecimal digit (0-f).
    NotValidHex(usize, char),

    /// String started with "(+|-)0o" and encountered a char that is not a valid octal digit (0-7).
    NotValidOct(usize, char),

    /// String started with "(+|-)0b" and encountered a char that is not a valid binary digit (0-1).
    NotValidBin(usize, char),

    /// A non valid char inside single quotes (') was encountered.
    NotValidChar(usize, char),

    /// A char that is not valid in decimal literals was encountered.
    NotValidDec(usize, char),

    /// A char that is not valid in a float literals was encountered.
    NotValidFloat(usize, char),


    /// A marker variant that tells the compiler that users of this enum cannot match it exhaustively.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl FromStr for Lit {
    type Err = ParseLitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "true" {
            Ok(Lit::Bool(true))
        } else if s == "false" {
            Ok(Lit::Bool(false))
        } else if s.starts_with("0x") {
            if let Some((pos, c)) = s.chars().enumerate().skip(2).find(|(_, c)| !is_hex_digit(*c)) {
                Err(ParseLitError::NotValidHex(pos, c))
            } else {
                Ok(Lit::Hex{sign: Sign::Implicit, value: String::from(&s[2..])})
            }
        } else if s.starts_with("+0x") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_hex_digit(*c)) {
                Err(ParseLitError::NotValidHex(pos, c))
            } else {
                Ok(Lit::Hex{sign: Sign::Positive, value: String::from(&s[3..])})
            }
        } else if s.starts_with("-0x") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_hex_digit(*c)) {
                Err(ParseLitError::NotValidHex(pos, c))
            } else {
                Ok(Lit::Hex{sign: Sign::Negative, value: String::from(&s[3..])})
            }
        } else if s.starts_with("0o") {
            if let Some((pos, c)) = s.chars().enumerate().skip(2).find(|(_, c)| !is_oct_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Oct{sign: Sign::Implicit, value: String::from(&s[2..])})
            }
        } else if s.starts_with("+0o") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_oct_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Oct{sign: Sign::Positive, value: String::from(&s[3..])})
            }
        } else if s.starts_with("-0o") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_oct_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Oct{sign: Sign::Negative, value: String::from(&s[3..])})
            }
        } else if s.starts_with("0b") {
            if let Some((pos, c)) = s.chars().enumerate().skip(2).find(|(_, c)| !is_bin_digit(*c)) {
                Err(ParseLitError::NotValidBin(pos, c))
            } else {
                Ok(Lit::Bin{sign: Sign::Implicit, value: String::from(&s[2..])})
            }
        } else if s.starts_with("+0b") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_bin_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Bin{sign: Sign::Positive, value: String::from(&s[3..])})
            }
        } else if s.starts_with("-0b") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_bin_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Bin{sign: Sign::Negative, value: String::from(&s[3..])})
            }
        } else if s.contains(".") || s.contains("e") || s.contains("E") {
            // TODO: More sanitization needs to be done. Only one e or E and one . should be allowed
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !allowed_in_float_literal(*c)) {
                Err(ParseLitError::NotValidFloat(pos, c))
            } else {
                if s.starts_with('+') {
                    Ok(Lit::Float{sign: Sign::Positive, value: String::from(&s[1..])})
                } else if s.starts_with('-') {
                    Ok(Lit::Float{sign: Sign::Negative, value: String::from(&s[1..])})
                } else {
                    Ok(Lit::Float{sign: Sign::Implicit, value: String::from(s)})
                }
            }
        } else if s.starts_with("'") && s.ends_with("'") && s.len() > 2 {
            // TODO: More sanitization of chars
            Ok(Lit::Char(String::from(&s[1..s.len()-1])))
        } else {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_numeric(*c)) {
                Err(ParseLitError::NotValidDec(pos, c))
            } else {
                if s.starts_with('+') {
                    Ok(Lit::Dec{sign: Sign::Positive, value: String::from(&s[1..])})
                } else if s.starts_with('-') {
                    Ok(Lit::Dec{sign: Sign::Negative, value: String::from(&s[1..])})
                } else {
                    Ok(Lit::Dec{sign: Sign::Implicit, value: String::from(s)})
                }
            }
        }
    }

}

// Helper functions for parsing literals

fn allowed_in_float_literal(c: char) -> bool {
    c == 'e'
        || c == 'E'
        || c == '.'
        || c == '+'
        || c == '-'
        || is_numeric(c)
}

fn is_numeric(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_hex_digit(c: char) -> bool {
    (c >= '0' && c <= '9')
        || (c >= 'a' && c <= 'f')
        || (c >= 'A' && c <= 'F')
}

fn is_oct_digit(c: char) -> bool {
    c >= '0' && c <= '7'
}

fn is_bin_digit(c: char) -> bool {
    c >= '0' && c <= '1'
}
