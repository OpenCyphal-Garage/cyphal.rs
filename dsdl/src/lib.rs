#[macro_use]
extern crate nom;

use std::str::FromStr;

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

impl FromStr for CastMode {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "saturated" => Ok(CastMode::Saturated),
            "truncated" => Ok(CastMode::Truncated),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArrayInfo {
    Single,
    Dynamic(u32),
    Static(u32),
}



#[derive(Debug, PartialEq, Eq)]
pub struct FieldType {
    pub cast_mode: Option<CastMode>,
    pub field_type: Ty,
    pub array: ArrayInfo,
    pub name: Ident,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayField {
    cast_mode: CastMode,
    
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ty{
    PrimitiveType(PrimitiveType),
    Path(Ident),
}

impl From<Ident> for Ty {
    fn from(t: Ident) -> Ty {
        Ty::Path(t)
    }
}

impl From<PrimitiveType> for Ty {
    fn from(t: PrimitiveType) -> Ty {
        Ty::PrimitiveType(t)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,
    
    Uint2,
    Uint3,
    Uint4,
    Uint5,
    Uint6,
    Uint7,
    Uint8,
    Uint9,
    Uint10,
    Uint11,
    Uint12,
    Uint13,
    Uint14,
    Uint15,
    Uint16,
    Uint17,
    Uint18,
    Uint19,
    Uint20,
    Uint21,
    Uint22,
    Uint23,
    Uint24,
    Uint25,
    Uint26,
    Uint27,
    Uint28,
    Uint29,
    Uint30,
    Uint31,
    Uint32,
    Uint33,
    Uint34,
    Uint35,
    Uint36,
    Uint37,
    Uint38,
    Uint39,
    Uint40,
    Uint41,
    Uint42,
    Uint43,
    Uint44,
    Uint45,
    Uint46,
    Uint47,
    Uint48,
    Uint49,
    Uint50,
    Uint51,
    Uint52,
    Uint53,
    Uint54,
    Uint55,
    Uint56,
    Uint57,
    Uint58,
    Uint59,
    Uint60,
    Uint61,
    Uint62,
    Uint63,
    Uint64,

    Int2,
    Int3,
    Int4,
    Int5,
    Int6,
    Int7,
    Int8,
    Int9,
    Int10,
    Int11,
    Int12,
    Int13,
    Int14,
    Int15,
    Int16,
    Int17,
    Int18,
    Int19,
    Int20,
    Int21,
    Int22,
    Int23,
    Int24,
    Int25,
    Int26,
    Int27,
    Int28,
    Int29,
    Int30,
    Int31,
    Int32,
    Int33,
    Int34,
    Int35,
    Int36,
    Int37,
    Int38,
    Int39,
    Int40,
    Int41,
    Int42,
    Int43,
    Int44,
    Int45,
    Int46,
    Int47,
    Int48,
    Int49,
    Int50,
    Int51,
    Int52,
    Int53,
    Int54,
    Int55,
    Int56,
    Int57,
    Int58,
    Int59,
    Int60,
    Int61,
    Int62,
    Int63,
    Int64,

    Float16,
    Float32,
    Float64,
    
    Void1,
    Void2,
    Void3,
    Void4,
    Void5,
    Void6,
    Void7,
    Void8,
    Void9,
    Void10,
    Void11,
    Void12,
    Void13,
    Void14,
    Void15,
    Void16,
    Void17,
    Void18,
    Void19,
    Void20,
    Void21,
    Void22,
    Void23,
    Void24,
    Void25,
    Void26,
    Void27,
    Void28,
    Void29,
    Void30,
    Void31,
    Void32,
    Void33,
    Void34,
    Void35,
    Void36,
    Void37,
    Void38,
    Void39,
    Void40,
    Void41,
    Void42,
    Void43,
    Void44,
    Void45,
    Void46,
    Void47,
    Void48,
    Void49,
    Void50,
    Void51,
    Void52,
    Void53,
    Void54,
    Void55,
    Void56,
    Void57,
    Void58,
    Void59,
    Void60,
    Void61,
    Void62,
    Void63,
    Void64,
}
        
impl PrimitiveType {
    fn new<'a>(s: &'a str) -> PrimitiveType {
        match s {
            "uint2" => PrimitiveType::Uint2,
            "uint3" => PrimitiveType::Uint3,
            "uint4" => PrimitiveType::Uint4,
            "uint5" => PrimitiveType::Uint5,
            "uint6" => PrimitiveType::Uint6,
            "uint7" => PrimitiveType::Uint7,
            "uint8" => PrimitiveType::Uint8,
            "uint16" => PrimitiveType::Uint16,
            "uint32" => PrimitiveType::Uint32,
            _ => panic!("{} is not a valid PrimitiveType", s),
        }
    }
}
