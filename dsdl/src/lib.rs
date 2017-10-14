#[macro_use]
extern crate nom;
pub mod parse;

#[derive(Debug, PartialEq, Eq)]
pub struct Comment(String);

impl<'a> From<&'a str> for Comment {
    fn from(s: &'a str) -> Comment {
        Comment(String::from(s))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ident(String);

impl<'a> From<&'a str> for Ident {
    fn from(s: &'a str) -> Ident {
        Ident(String::from(s))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CastMode {
    Saturated,
    Truncated,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArrayInfo {
    Single,
    Dynamic(usize),
    Static(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldType {
    pub cast_mode: CastMode,
    pub array: ArrayInfo,
    pub name: Ident,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayField {
    cast_mode: CastMode,
    
}

#[derive(Debug, PartialEq, Eq)]
pub enum Attribute {

}
    
    
