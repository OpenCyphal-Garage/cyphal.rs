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

/// The error returned when attempting to parse something that is not a `PrimitiveType`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotPrimitiveTypeError;

impl FromStr for PrimitiveType {
    type Err = NotPrimitiveTypeError;

    fn from_str(s: &str) -> Result<PrimitiveType, Self::Err> {
        match s {
            "bool" => Ok(PrimitiveType::Bool),

            "uint2" => Ok(PrimitiveType::Uint2),
            "uint3" => Ok(PrimitiveType::Uint3),
            "uint4" => Ok(PrimitiveType::Uint4),
            "uint5" => Ok(PrimitiveType::Uint5),
            "uint6" => Ok(PrimitiveType::Uint6),
            "uint7" => Ok(PrimitiveType::Uint7),
            "uint8" => Ok(PrimitiveType::Uint8),
            "uint9" => Ok(PrimitiveType::Uint9),
            "uint10" => Ok(PrimitiveType::Uint10),
            "uint11" => Ok(PrimitiveType::Uint11),
            "uint12" => Ok(PrimitiveType::Uint12),
            "uint13" => Ok(PrimitiveType::Uint13),
            "uint14" => Ok(PrimitiveType::Uint14),
            "uint15" => Ok(PrimitiveType::Uint15),
            "uint16" => Ok(PrimitiveType::Uint16),
            "uint17" => Ok(PrimitiveType::Uint17),
            "uint18" => Ok(PrimitiveType::Uint18),
            "uint19" => Ok(PrimitiveType::Uint19),
            "uint20" => Ok(PrimitiveType::Uint20),
            "uint21" => Ok(PrimitiveType::Uint21),
            "uint22" => Ok(PrimitiveType::Uint22),
            "uint23" => Ok(PrimitiveType::Uint23),
            "uint24" => Ok(PrimitiveType::Uint24),
            "uint25" => Ok(PrimitiveType::Uint25),
            "uint26" => Ok(PrimitiveType::Uint26),
            "uint27" => Ok(PrimitiveType::Uint27),
            "uint28" => Ok(PrimitiveType::Uint28),
            "uint29" => Ok(PrimitiveType::Uint29),
            "uint30" => Ok(PrimitiveType::Uint30),
            "uint31" => Ok(PrimitiveType::Uint31),
            "uint32" => Ok(PrimitiveType::Uint32),

            "uint33" => Ok(PrimitiveType::Uint33),
            "uint34" => Ok(PrimitiveType::Uint34),
            "uint35" => Ok(PrimitiveType::Uint35),
            "uint36" => Ok(PrimitiveType::Uint36),
            "uint37" => Ok(PrimitiveType::Uint37),
            "uint38" => Ok(PrimitiveType::Uint38),
            "uint39" => Ok(PrimitiveType::Uint39),
            "uint40" => Ok(PrimitiveType::Uint40),
            "uint41" => Ok(PrimitiveType::Uint41),
            "uint42" => Ok(PrimitiveType::Uint42),
            "uint43" => Ok(PrimitiveType::Uint43),
            "uint44" => Ok(PrimitiveType::Uint44),
            "uint45" => Ok(PrimitiveType::Uint45),
            "uint46" => Ok(PrimitiveType::Uint46),
            "uint47" => Ok(PrimitiveType::Uint47),
            "uint48" => Ok(PrimitiveType::Uint48),
            "uint49" => Ok(PrimitiveType::Uint49),
            "uint50" => Ok(PrimitiveType::Uint50),
            "uint51" => Ok(PrimitiveType::Uint51),
            "uint52" => Ok(PrimitiveType::Uint52),
            "uint53" => Ok(PrimitiveType::Uint53),
            "uint54" => Ok(PrimitiveType::Uint54),
            "uint55" => Ok(PrimitiveType::Uint55),
            "uint56" => Ok(PrimitiveType::Uint56),
            "uint57" => Ok(PrimitiveType::Uint57),
            "uint58" => Ok(PrimitiveType::Uint58),
            "uint59" => Ok(PrimitiveType::Uint59),
            "uint60" => Ok(PrimitiveType::Uint60),
            "uint61" => Ok(PrimitiveType::Uint61),
            "uint62" => Ok(PrimitiveType::Uint62),
            "uint63" => Ok(PrimitiveType::Uint63),
            "uint64" => Ok(PrimitiveType::Uint64),

            "int2" => Ok(PrimitiveType::Int2),
            "int3" => Ok(PrimitiveType::Int3),
            "int4" => Ok(PrimitiveType::Int4),
            "int5" => Ok(PrimitiveType::Int5),
            "int6" => Ok(PrimitiveType::Int6),
            "int7" => Ok(PrimitiveType::Int7),
            "int8" => Ok(PrimitiveType::Int8),
            "int9" => Ok(PrimitiveType::Int9),
            "int10" => Ok(PrimitiveType::Int10),
            "int11" => Ok(PrimitiveType::Int11),
            "int12" => Ok(PrimitiveType::Int12),
            "int13" => Ok(PrimitiveType::Int13),
            "int14" => Ok(PrimitiveType::Int14),
            "int15" => Ok(PrimitiveType::Int15),
            "int16" => Ok(PrimitiveType::Int16),
            "int17" => Ok(PrimitiveType::Int17),
            "int18" => Ok(PrimitiveType::Int18),
            "int19" => Ok(PrimitiveType::Int19),
            "int20" => Ok(PrimitiveType::Int20),
            "int21" => Ok(PrimitiveType::Int21),
            "int22" => Ok(PrimitiveType::Int22),
            "int23" => Ok(PrimitiveType::Int23),
            "int24" => Ok(PrimitiveType::Int24),
            "int25" => Ok(PrimitiveType::Int25),
            "int26" => Ok(PrimitiveType::Int26),
            "int27" => Ok(PrimitiveType::Int27),
            "int28" => Ok(PrimitiveType::Int28),
            "int29" => Ok(PrimitiveType::Int29),
            "int30" => Ok(PrimitiveType::Int30),
            "int31" => Ok(PrimitiveType::Int31),
            "int32" => Ok(PrimitiveType::Int32),

            "int33" => Ok(PrimitiveType::Int33),
            "int34" => Ok(PrimitiveType::Int34),
            "int35" => Ok(PrimitiveType::Int35),
            "int36" => Ok(PrimitiveType::Int36),
            "int37" => Ok(PrimitiveType::Int37),
            "int38" => Ok(PrimitiveType::Int38),
            "int39" => Ok(PrimitiveType::Int39),
            "int40" => Ok(PrimitiveType::Int40),
            "int41" => Ok(PrimitiveType::Int41),
            "int42" => Ok(PrimitiveType::Int42),
            "int43" => Ok(PrimitiveType::Int43),
            "int44" => Ok(PrimitiveType::Int44),
            "int45" => Ok(PrimitiveType::Int45),
            "int46" => Ok(PrimitiveType::Int46),
            "int47" => Ok(PrimitiveType::Int47),
            "int48" => Ok(PrimitiveType::Int48),
            "int49" => Ok(PrimitiveType::Int49),
            "int50" => Ok(PrimitiveType::Int50),
            "int51" => Ok(PrimitiveType::Int51),
            "int52" => Ok(PrimitiveType::Int52),
            "int53" => Ok(PrimitiveType::Int53),
            "int54" => Ok(PrimitiveType::Int54),
            "int55" => Ok(PrimitiveType::Int55),
            "int56" => Ok(PrimitiveType::Int56),
            "int57" => Ok(PrimitiveType::Int57),
            "int58" => Ok(PrimitiveType::Int58),
            "int59" => Ok(PrimitiveType::Int59),
            "int60" => Ok(PrimitiveType::Int60),
            "int61" => Ok(PrimitiveType::Int61),
            "int62" => Ok(PrimitiveType::Int62),
            "int63" => Ok(PrimitiveType::Int63),
            "int64" => Ok(PrimitiveType::Int64),

            "void1" => Ok(PrimitiveType::Void1),
            "void2" => Ok(PrimitiveType::Void2),
            "void3" => Ok(PrimitiveType::Void3),
            "void4" => Ok(PrimitiveType::Void4),
            "void5" => Ok(PrimitiveType::Void5),
            "void6" => Ok(PrimitiveType::Void6),
            "void7" => Ok(PrimitiveType::Void7),
            "void8" => Ok(PrimitiveType::Void8),
            "void9" => Ok(PrimitiveType::Void9),
            "void10" => Ok(PrimitiveType::Void10),
            "void11" => Ok(PrimitiveType::Void11),
            "void12" => Ok(PrimitiveType::Void12),
            "void13" => Ok(PrimitiveType::Void13),
            "void14" => Ok(PrimitiveType::Void14),
            "void15" => Ok(PrimitiveType::Void15),
            "void16" => Ok(PrimitiveType::Void16),
            "void17" => Ok(PrimitiveType::Void17),
            "void18" => Ok(PrimitiveType::Void18),
            "void19" => Ok(PrimitiveType::Void19),
            "void20" => Ok(PrimitiveType::Void20),
            "void21" => Ok(PrimitiveType::Void21),
            "void22" => Ok(PrimitiveType::Void22),
            "void23" => Ok(PrimitiveType::Void23),
            "void24" => Ok(PrimitiveType::Void24),
            "void25" => Ok(PrimitiveType::Void25),
            "void26" => Ok(PrimitiveType::Void26),
            "void27" => Ok(PrimitiveType::Void27),
            "void28" => Ok(PrimitiveType::Void28),
            "void29" => Ok(PrimitiveType::Void29),
            "void30" => Ok(PrimitiveType::Void30),
            "void31" => Ok(PrimitiveType::Void31),
            "void32" => Ok(PrimitiveType::Void32),

            "void33" => Ok(PrimitiveType::Void33),
            "void34" => Ok(PrimitiveType::Void34),
            "void35" => Ok(PrimitiveType::Void35),
            "void36" => Ok(PrimitiveType::Void36),
            "void37" => Ok(PrimitiveType::Void37),
            "void38" => Ok(PrimitiveType::Void38),
            "void39" => Ok(PrimitiveType::Void39),
            "void40" => Ok(PrimitiveType::Void40),
            "void41" => Ok(PrimitiveType::Void41),
            "void42" => Ok(PrimitiveType::Void42),
            "void43" => Ok(PrimitiveType::Void43),
            "void44" => Ok(PrimitiveType::Void44),
            "void45" => Ok(PrimitiveType::Void45),
            "void46" => Ok(PrimitiveType::Void46),
            "void47" => Ok(PrimitiveType::Void47),
            "void48" => Ok(PrimitiveType::Void48),
            "void49" => Ok(PrimitiveType::Void49),
            "void50" => Ok(PrimitiveType::Void50),
            "void51" => Ok(PrimitiveType::Void51),
            "void52" => Ok(PrimitiveType::Void52),
            "void53" => Ok(PrimitiveType::Void53),
            "void54" => Ok(PrimitiveType::Void54),
            "void55" => Ok(PrimitiveType::Void55),
            "void56" => Ok(PrimitiveType::Void56),
            "void57" => Ok(PrimitiveType::Void57),
            "void58" => Ok(PrimitiveType::Void58),
            "void59" => Ok(PrimitiveType::Void59),
            "void60" => Ok(PrimitiveType::Void60),
            "void61" => Ok(PrimitiveType::Void61),
            "void62" => Ok(PrimitiveType::Void62),
            "void63" => Ok(PrimitiveType::Void63),
            "void64" => Ok(PrimitiveType::Void64),

            "float16" => Ok(PrimitiveType::Float16),
            "float32" => Ok(PrimitiveType::Float32),
            "float64" => Ok(PrimitiveType::Float64),
            _ => Err(NotPrimitiveTypeError),
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