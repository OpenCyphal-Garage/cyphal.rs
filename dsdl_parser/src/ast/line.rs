//! Everything related to the uavcan line `Line`

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use ast::file_name::FileName;
use ast::comment::Comment;
use ast::attribute_definition::AttributeDefinition;
use ast::directive::Directive;

/// A `Line` in a DSDL `File`
///
/// A data structure definition consists of attributes and directives.
/// Any line of the definition file may contain at most one attribute definition or at most one directive.
/// The same line cannot contain an attribute definition and a directive at the same time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Empty,
    Comment(Comment),
    Definition{definition: AttributeDefinition, comment: Option<Comment>},
    Directive{directive: Directive, comment: Option<Comment>},
}

impl Line {
    /// returns true if the `Line` is empty
    pub fn is_empty(&self) -> bool {
        match *self {
            Line::Empty => true,
            _ => false,
        }
    }

    /// returns true if the `Line` contains a directive
    pub fn is_directive(&self) -> bool {
        match *self {
            Line::Directive{..} => true,
            _ => false,
        }
    }

    /// returns true if the `Line` contains a definiition
    pub fn is_definition(&self) -> bool {
        match *self {
            Line::Definition{..} => true,
            _ => false,
        }
    }

    /// returns true if the `Line` is nothing but a comment
    pub fn is_comment(&self) -> bool {
        match *self {
            Line::Comment(_) => true,
            _ => false,
        }
    }

    /// returns true if the `Line` contains a comment
    pub fn has_comment(&self) -> bool {
        match *self {
            Line::Comment(_) => true,
            Line::Directive{comment: Some(_), ..} => true,
            Line::Definition{comment: Some(_), ..} => true,
            _ => false,
        }
    }

}

impl Line {
    pub(crate) fn normalize(self, file_name: &FileName) -> Option<Self> {
        // 1. Remove comments.
        match self {
            Line::Empty => None,
            Line::Comment(_) => None,
            Line::Definition{definition, ..} => match definition.normalize(file_name) {
                Some(norm_def) => Some(Line::Definition { definition: norm_def, comment: None }),
                None => None,
            },
            Line::Directive{directive, ..} => Some(Line::Directive{directive, comment: None}),
        }
    }
}


impl Display for Line {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Line::Empty => write!(f, ""),
            Line::Comment(ref comment) => write!(f, "#{}", comment),
            Line::Definition{definition, comment} => {
                match comment {
                    Some(comment) => write!(f, "{} #{}", definition, comment),
                    None => write!(f, "{}", definition),
                }
            },
            Line::Directive{directive, comment} => {
                match comment {
                    Some(comment) => write!(f, "{} #{}", directive, comment),
                    None => write!(f, "{}", directive),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use *;

    use std::str::FromStr;

    #[test]
    fn display_line() {
        assert_eq!(format!("{}", Line::Empty), "");

        assert_eq!(format!("{}", Line::Comment(Comment::from_str("# test comment").unwrap())), "# test comment");

        assert_eq!(format!("{}",
                           Line::Definition{
                               definition: AttributeDefinition::Field(FieldDefinition{
                                   cast_mode: None,
                                   field_type: Ty::Primitive(PrimitiveType::Uint32),
                                   array: None,
                                   name: Some(Ident::from_str("uptime_sec").unwrap()),
                               }),
                               comment: None}),
                   "uint32 uptime_sec"
        );

        assert_eq!(format!("{}",
                           Line::Definition{
                               definition: AttributeDefinition::Field(FieldDefinition{
                                   cast_mode: None,
                                   field_type: Ty::Primitive(PrimitiveType::Uint32),
                                   array: None,
                                   name: Some(Ident::from_str("uptime_sec").unwrap()),
                               }),
                               comment: Some(Comment::from_str("# test comment").unwrap())}),
                   "uint32 uptime_sec # test comment"
        );

        assert_eq!(format!("{}",
                           Line::Definition{
                               definition: AttributeDefinition::Const(ConstDefinition{
                                   cast_mode: None,
                                   field_type: Ty::Primitive(PrimitiveType::Uint2),
                                   name: Ident::from_str("HEALTH_OK").unwrap(),
                                   literal: Lit::Dec{sign: Sign::Implicit, value: String::from("0")},
                               }),
                               comment: Some(Comment::from_str("# test comment").unwrap())}),
                   "uint2 HEALTH_OK = 0 # test comment"
        );

        assert_eq!(format!("{}",
                           Line::Directive{
                               directive: Directive::Union,
                               comment: Some(Comment::from_str("# test comment").unwrap())}),
                   "@union # test comment"
        );


    }
}
