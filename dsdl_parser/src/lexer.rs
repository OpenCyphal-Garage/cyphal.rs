use std::str::CharIndices;
use std::iter::Peekable;

use std::str::FromStr;

use {
    PrimitiveType,
    CastMode,
    Lit,
    Comment,
    Ident,
};

pub type Spanned<Token, Loc, Error> = Result<(Loc, Token, Loc), Error>;

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Token {
    /// A comment
    Comment(Comment),

    /// A composite type name, const name, field name or directive name.
    Ident(Ident),

    /// A Literal
    Lit(Lit),

    /// A `PrimitiveType` keyword like `void2` or `int43`
    PrimitiveType(PrimitiveType),

    /// A `CastMode` keyword like `saturated` or `truncated`.
    CastMode(CastMode),

    /// Left bracket `[`, used for starting an array description.
    LeftBracket,
    /// Right bracket `]`, used for ending an array description.
    RightBracket,
    /// Less than `<`. Used for specifying dynamic array max length.
    Less,
    /// Less than or equal `<=`. Used for specifying dynamic array max length.
    LessEq,

    /// The Constant Assigment. An equal sign, `=`, not directly following `<`.
    Eq,

    /// The Directive Marker. `@`.
    DirectiveMarker,

    /// The Service Response Marker, `---` on a dedicated line
    ServiceResponseMarker,

    /// End of line
    Eol,
}

#[derive(Debug, PartialEq)]
pub enum LexicalError {

}

pub(crate) struct Lexer<'input> {
    input: &'input str,
    chars: Peekable<CharIndices<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer{
            input,
            chars: input.char_indices().peekable()
        }
    }

    /// Move through the char indices iterator untill predicate is no longer true.
    /// After calling this function the next element will be the next one where predicate is not true
    /// Returns number of elements that was skipped
    fn skip_while<P: Fn(char) -> bool>(&mut self, predicate: P) -> usize {
        let mut thrown = 0;
        loop {
            match self.chars.peek().clone() {
                Some((_, c)) if predicate(*c) => (),
                _ => return thrown,
            }
            self.chars.next();
            thrown += 1;
        }
    }

    fn look(&self, pos: usize) -> Option<(char)> {
        self.input.chars().nth(pos)
    }

    /// Test a predicate on a position, returns the default value if no such element exists
    fn test_pos_or<P: Fn(char) -> bool>(&self, pos: usize, default: bool, predicate: P) -> bool {
        self.look(pos).map_or(default, predicate)
    }

}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_while(is_whitespace);

        match self.chars.next() {
            // Comments
            Some((i, '#')) => {
                let length = self.skip_while(|c| c!='\r' && c!='\n') + 1;
                let comment = Comment::from_str(&self.input[i..i+length]).expect("Only legal symbols for comments are included");
                Some(Ok((i, Token::Comment(comment), i + length + 1)))
            },


            // EOL
            Some((i, '\r')) if self.test_pos_or(i+1, false, |c| c == '\n') => {
                assert_eq!(self.chars.next(), Some((i + 1, '\n'))); // Pop out the '\n'
                Some(Ok((i, Token::Eol, i + 2)))
            },
            Some((i, '\n')) => Some(Ok((i, Token::Eol, i+1))),
            Some((i, '\r')) => unimplemented!("TODO: Insert error \\r without \\n"),


            // Symbols
            Some((i, '@')) => Some(Ok((i, Token::DirectiveMarker, i+1))),

            Some((i, '-')) if self.test_pos_or(i+1, false, |c| c == '-')  => {
                match self.skip_while(|c| c == '-') {
                    2 => Some(Ok((i, Token::ServiceResponseMarker, i + 3))),
                    _x => unimplemented!("TODO: Insert error Wrong number of - while parsing ServiceResponseMarker"),
                }
            },

            Some((i, '[')) => Some(Ok((i, Token::LeftBracket, i+1))),
            Some((i, ']')) => Some(Ok((i, Token::RightBracket, i+1))),
            Some((i, '<')) if self.look(i+1).map_or(false, |c| c == '=') => {
                assert_eq!(self.chars.next(), Some((i + 1, '='))); // Pop out the '='
                Some(Ok((i, Token::LessEq, i + 2)))
            },
            Some((i, '<')) => Some(Ok((i, Token::Less, i+1))),

            Some((i, '=')) => Some(Ok((i, Token::Eq, i+1))),


            // All literals except true and false will match this rule
            Some((i, c)) if allowed_as_literal_start(c) => {
                let literal_length = self.skip_while(allowed_in_literal) + 1;
                match Lit::from_str(&self.input[i..i+literal_length]) {
                    Ok(lit) => Some(Ok((i, Token::Lit(lit), i+literal_length))),
                    Err(e) => unimplemented!("TODO: propage errors for bad formated literals: {:?}", e)
                }
            }


            // Identifiers and primitive types and all keywords (including true/false) will match this rule
            Some((i, c)) if allowed_as_identifier_start(c) => {
                let identifier_length = self.skip_while(allowed_in_identifier) + 1;
                let ident_str = &self.input[i..i+identifier_length];

                if let Ok(primitive_type) = PrimitiveType::from_str(ident_str) {
                    return Some(Ok((i, Token::PrimitiveType(primitive_type), i+identifier_length)));
                };

                match ident_str {
                    "truncated" => Some(Ok((i, Token::CastMode(CastMode::Truncated), i+identifier_length))),
                    "saturated" => Some(Ok((i, Token::CastMode(CastMode::Saturated), i+identifier_length))),
                    "true" => Some(Ok((i, Token::Lit(Lit::Bool(true)), i+identifier_length))),
                    "false" => Some(Ok((i, Token::Lit(Lit::Bool(false)), i+identifier_length))),
                    other => Some(Ok((i, Token::Ident(Ident::from_str(other).expect("Only legal symbols for identifiers are included")), i+identifier_length))),
                }
            }

            Some(_) => unimplemented!(),
            None => None, // End of file
        }
    }
}

// FromStr impls for tokens

/// Errors that may occur when parsing `Lit`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseLitError {
    /// Decimal number starts with `+0`
    DecStartsWithPosZero,

    /// Decimal number starts with `-0`
    DecStartsWithNegZero,

    /// Decimal number starts with `0`
    DecStartsWithZero,

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
                Ok(Lit::Hex(String::from(s)))
            }
        } else if s.starts_with("+0x") || s.starts_with("-0x") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_hex_digit(*c)) {
                Err(ParseLitError::NotValidHex(pos, c))
            } else {
                Ok(Lit::Hex(String::from(s)))
            }
        } else if s.starts_with("0o") {
            if let Some((pos, c)) = s.chars().enumerate().skip(2).find(|(_, c)| !is_oct_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Oct(String::from(s)))
            }
        } else if s.starts_with("+0o") || s.starts_with("-0o") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_oct_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Oct(String::from(s)))
            }
        } else if s.starts_with("0b") {
            if let Some((pos, c)) = s.chars().enumerate().skip(2).find(|(_, c)| !is_bin_digit(*c)) {
                Err(ParseLitError::NotValidBin(pos, c))
            } else {
                Ok(Lit::Bin(String::from(s)))
            }
        } else if s.starts_with("+0b") || s.starts_with("-0b") {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_bin_digit(*c)) {
                Err(ParseLitError::NotValidOct(pos, c))
            } else {
                Ok(Lit::Bin(String::from(s)))
            }
        } else if s.contains(".") || s.contains("e") || s.contains("E") {
            // TODO: More sanitization needs to be done. Only one e or E and one . should be allowed
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !allowed_in_float_literal(*c)) {
                Err(ParseLitError::NotValidFloat(pos, c))
            } else {
                Ok(Lit::Float(String::from(s)))
            }
        } else if s.starts_with("-0") {
            Err(ParseLitError::DecStartsWithNegZero)
        } else if s.starts_with("+0") {
            Err(ParseLitError::DecStartsWithPosZero)
        } else if s.starts_with("0") {
            Err(ParseLitError::DecStartsWithZero)
        } else if s.starts_with("'") && s.ends_with("'") {
            // TODO: More sanitization of chars
            Ok(Lit::Char(String::from(&s[1..s.len()-1])))
        } else {
            if let Some((pos, c)) = s.chars().enumerate().skip(3).find(|(_, c)| !is_numeric(*c)) {
                Err(ParseLitError::NotValidDec(pos, c))
            } else {
                Ok(Lit::Dec(String::from(s)))
            }
        }
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

// Helper functions for parsing

fn allowed_as_identifier_start(c: char) -> bool {
    (c >= 'a' && c <= 'z')
        || (c >= 'A' && c <= 'Z')

}

fn allowed_in_identifier(c: char) -> bool {
    (c >= 'a' && c <= 'z')
        || (c >= 'A' && c <= 'Z')
        || (c >= '0' && c <= '9')
        || c == '_'
        || c == '-'
        || c == '.'
}

fn allowed_as_literal_start(c: char) -> bool {
    c == '+'
    || c == '-'
    || c == '\''
    || is_numeric(c)
}

fn allowed_in_literal(c: char) -> bool {
    c == '+'
    || c == '-'
    || c == '.'
    || c == '\''
    || c == '\\'
    || (c >= 'a' && c <= 'z')
    || (c >= 'A' && c <= 'Z')
    || is_numeric(c)
}

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
}

fn is_oct_digit(c: char) -> bool {
    c >= '0' && c <= '7'
}

fn is_bin_digit(c: char) -> bool {
    c >= '0' && c <= '1'
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn tokenize_bool_literal() {
        let mut lexer = Lexer::new("true true false false true false");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(true)));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(true)));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(false)));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(false)));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(true)));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(false)));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_symbols() {
        // array related symbols are tested in seperate test
        let mut lexer = Lexer::new(
"@
=
---
---"
        );

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::DirectiveMarker);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eq);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::ServiceResponseMarker);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::ServiceResponseMarker);

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_cast_mode() {
        let mut lexer = Lexer::new("truncated saturated saturated truncated");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::CastMode(CastMode::Truncated));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::CastMode(CastMode::Saturated));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::CastMode(CastMode::Saturated));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::CastMode(CastMode::Truncated));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_array_symbols() {
        let mut lexer = Lexer::new("[<=123][456][<789]");
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::LeftBracket);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::LessEq);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Dec(String::from("123"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::RightBracket);

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::LeftBracket);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Dec(String::from("456"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::RightBracket);

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::LeftBracket);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Less);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Dec(String::from("789"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::RightBracket);

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_comment() {
        let mut lexer = Lexer::new(
"#This is a comment\n#This is a comment\r\n#This is a comment
#This is a longer comment
# This is a comment
#"
        );

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Comment(Comment(String::from("This is a comment"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Comment(Comment(String::from("This is a comment"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Comment(Comment(String::from("This is a comment"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Comment(Comment(String::from("This is a longer comment"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Comment(Comment(String::from(" This is a comment"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Eol);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Comment(Comment(String::from(""))));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_literal() {
        let mut lexer = Lexer::new("12354 -12 +12 0x123 -0x12 +0x123 0b1101 -0b101101 +0b101101 -0o123 0o777 +0o777 15.75 1.575E1 1575e-2 -2.5e-3 +25e-4 true false 'a' '\\x61' '\\n'");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Dec(String::from("12354"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Dec(String::from("-12"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Dec(String::from("+12"))));

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Hex(String::from("0x123"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Hex(String::from("-0x12"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Hex(String::from("+0x123"))));

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bin(String::from("0b1101"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bin(String::from("-0b101101"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bin(String::from("+0b101101"))));

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Oct(String::from("-0o123"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Oct(String::from("0o777"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Oct(String::from("+0o777"))));

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Float(String::from("15.75"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Float(String::from("1.575E1"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Float(String::from("1575e-2"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Float(String::from("-2.5e-3"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Float(String::from("+25e-4"))));

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(true)));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Bool(false)));

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Char(String::from("a"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Char(String::from("\\x61"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Lit(Lit::Char(String::from("\\n"))));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_directive() {
        let mut lexer = Lexer::new("@union");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::DirectiveMarker);
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("union"))));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_field_names() {
        let mut lexer = Lexer::new("variable23 var_iable23");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("variable23"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("var_iable23"))));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_const_name() {
        let mut lexer = Lexer::new("CONST CONST23 CON_ST CON_ST1_2345");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("CONST"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("CONST23"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("CON_ST"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("CON_ST1_2345"))));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_composite_type_name() {
        let mut lexer = Lexer::new("TypeName TypeName1234 uavcan.protocol.TypeName uavcan.protocol.TypeName1234");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("TypeName"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("TypeName1234"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("uavcan.protocol.TypeName"))));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::Ident(Ident(String::from("uavcan.protocol.TypeName1234"))));

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_primitive_type() {
        let mut lexer = Lexer::new("uint2 int3 void16 uint32 float64");

        assert_eq!(lexer.next().unwrap().unwrap().1, Token::PrimitiveType(PrimitiveType::Uint2));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::PrimitiveType(PrimitiveType::Int3));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::PrimitiveType(PrimitiveType::Void16));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::PrimitiveType(PrimitiveType::Uint32));
        assert_eq!(lexer.next().unwrap().unwrap().1, Token::PrimitiveType(PrimitiveType::Float64));

        assert_eq!(lexer.next(), None);
    }
}