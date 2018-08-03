//! Everything related to the uavcan type `Ty`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use ast::ident::Ident;

/// An Uavcan data type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty{
    Primitive(PrimitiveType),
    Composite(CompositeType),
}

impl Ty {
    pub fn is_void(&self) -> bool {
        match *self{
            Ty::Primitive(ref x) => x.is_void(),
            Ty::Composite(_) => false,
        }
    }
}

impl From<PrimitiveType> for Ty {
    fn from(t: PrimitiveType) -> Ty {
        Ty::Primitive(t)
    }
}

impl From<CompositeType> for Ty {
    fn from(t: CompositeType) -> Ty {
        Ty::Composite(t)
    }
}

/// A CompositeType is what the uavcan specification refers to as "Nested data structures"
///
/// In short if it's not a primitive data type (or arrays of primitive data types) it's a `CompositeType`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositeType {
    pub namespace: Option<Ident>,
    pub name: Ident,
}

impl FromStr for CompositeType {
    type Err = ();

    fn from_str(s: &str) -> Result<CompositeType, Self::Err> {
        if s.contains('.') {
            let mut split = s.rsplitn(2, '.');
            let name = Ident::from(String::from(split.next().unwrap()));
            let namespace = match split.next() {
                Some(x) => Some(Ident::from(String::from(x))),
                None => None,
            };
            Ok(CompositeType {
                namespace: namespace,
                name: name,
            })
        } else {
            Ok(CompositeType {
                namespace: None,
                name: Ident::from(String::from(s))
            })
        }
    }
}

/// An Uavcan `PrimitiveDataType`
///
/// These types are assumed to be built-in. They can be directly referenced from any data type of any namespace.
/// The DSDL compiler should implement these types using the native types of the target programming language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,

    Uint2,  Uint3,  Uint4,  Uint5,  Uint6,  Uint7,  Uint8,
    Uint9,  Uint10, Uint11, Uint12, Uint13, Uint14, Uint15, Uint16,
    Uint17, Uint18, Uint19, Uint20, Uint21, Uint22, Uint23, Uint24,
    Uint25, Uint26, Uint27, Uint28, Uint29, Uint30, Uint31, Uint32,
    Uint33, Uint34, Uint35, Uint36, Uint37, Uint38, Uint39, Uint40,
    Uint41, Uint42, Uint43, Uint44, Uint45, Uint46, Uint47, Uint48,
    Uint49, Uint50, Uint51, Uint52, Uint53, Uint54, Uint55, Uint56,
    Uint57, Uint58, Uint59, Uint60, Uint61, Uint62, Uint63, Uint64,

    Int2,  Int3,  Int4,  Int5,  Int6,  Int7,  Int8,
    Int9,  Int10, Int11, Int12, Int13, Int14, Int15, Int16,
    Int17, Int18, Int19, Int20, Int21, Int22, Int23, Int24,
    Int25, Int26, Int27, Int28, Int29, Int30, Int31, Int32,
    Int33, Int34, Int35, Int36, Int37, Int38, Int39, Int40,
    Int41, Int42, Int43, Int44, Int45, Int46, Int47, Int48,
    Int49, Int50, Int51, Int52, Int53, Int54, Int55, Int56,
    Int57, Int58, Int59, Int60, Int61, Int62, Int63, Int64,

    Float16, Float32, Float64,

    Void1,  Void2,  Void3,  Void4,  Void5,  Void6,  Void7,  Void8,
    Void9,  Void10, Void11, Void12, Void13, Void14, Void15, Void16,
    Void17, Void18, Void19, Void20, Void21, Void22, Void23, Void24,
    Void25, Void26, Void27, Void28, Void29, Void30, Void31, Void32,
    Void33, Void34, Void35, Void36, Void37, Void38, Void39, Void40,
    Void41, Void42, Void43, Void44, Void45, Void46, Void47, Void48,
    Void49, Void50, Void51, Void52, Void53, Void54, Void55, Void56,
    Void57, Void58, Void59, Void60, Void61, Void62, Void63, Void64,
}

impl PrimitiveType {
    pub fn is_void(&self) -> bool {
        match *self {
            PrimitiveType::Void1 => true,
            PrimitiveType::Void2 => true,
            PrimitiveType::Void3 => true,
            PrimitiveType::Void4 => true,
            PrimitiveType::Void5 => true,
            PrimitiveType::Void6 => true,
            PrimitiveType::Void7 => true,
            PrimitiveType::Void8 => true,
            PrimitiveType::Void9 => true,
            PrimitiveType::Void10 => true,
            PrimitiveType::Void11 => true,
            PrimitiveType::Void12 => true,
            PrimitiveType::Void13 => true,
            PrimitiveType::Void14 => true,
            PrimitiveType::Void15 => true,
            PrimitiveType::Void16 => true,
            PrimitiveType::Void17 => true,
            PrimitiveType::Void18 => true,
            PrimitiveType::Void19 => true,
            PrimitiveType::Void20 => true,
            PrimitiveType::Void21 => true,
            PrimitiveType::Void22 => true,
            PrimitiveType::Void23 => true,
            PrimitiveType::Void24 => true,
            PrimitiveType::Void25 => true,
            PrimitiveType::Void26 => true,
            PrimitiveType::Void27 => true,
            PrimitiveType::Void28 => true,
            PrimitiveType::Void29 => true,
            PrimitiveType::Void30 => true,
            PrimitiveType::Void31 => true,
            PrimitiveType::Void32 => true,
            PrimitiveType::Void33 => true,
            PrimitiveType::Void34 => true,
            PrimitiveType::Void35 => true,
            PrimitiveType::Void36 => true,
            PrimitiveType::Void37 => true,
            PrimitiveType::Void38 => true,
            PrimitiveType::Void39 => true,
            PrimitiveType::Void40 => true,
            PrimitiveType::Void41 => true,
            PrimitiveType::Void42 => true,
            PrimitiveType::Void43 => true,
            PrimitiveType::Void44 => true,
            PrimitiveType::Void45 => true,
            PrimitiveType::Void46 => true,
            PrimitiveType::Void47 => true,
            PrimitiveType::Void48 => true,
            PrimitiveType::Void49 => true,
            PrimitiveType::Void50 => true,
            PrimitiveType::Void51 => true,
            PrimitiveType::Void52 => true,
            PrimitiveType::Void53 => true,
            PrimitiveType::Void54 => true,
            PrimitiveType::Void55 => true,
            PrimitiveType::Void56 => true,
            PrimitiveType::Void57 => true,
            PrimitiveType::Void58 => true,
            PrimitiveType::Void59 => true,
            PrimitiveType::Void60 => true,
            PrimitiveType::Void61 => true,
            PrimitiveType::Void62 => true,
            PrimitiveType::Void63 => true,
            PrimitiveType::Void64 => true,
            _ => false,
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

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Ty::Primitive(ref ty) => write!(f, "{}", ty),
            Ty::Composite(ref ty) => write!(f, "{}", ty),
        }
    }
}

impl Display for CompositeType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self.namespace {
            Some(ref namespace) => write!(f, "{}.{}", namespace, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", match *self {
            PrimitiveType::Bool => "bool",

            PrimitiveType::Uint2 => "uint2",
            PrimitiveType::Uint3 => "uint3",
            PrimitiveType::Uint4 => "uint4",
            PrimitiveType::Uint5 => "uint5",
            PrimitiveType::Uint6 => "uint6",
            PrimitiveType::Uint7 => "uint7",
            PrimitiveType::Uint8 => "uint8",
            PrimitiveType::Uint9 => "uint9",
            PrimitiveType::Uint10 => "uint10",
            PrimitiveType::Uint11 => "uint11",
            PrimitiveType::Uint12 => "uint12",
            PrimitiveType::Uint13 => "uint13",
            PrimitiveType::Uint14 => "uint14",
            PrimitiveType::Uint15 => "uint15",
            PrimitiveType::Uint16 => "uint16",
            PrimitiveType::Uint17 => "uint17",
            PrimitiveType::Uint18 => "uint18",
            PrimitiveType::Uint19 => "uint19",
            PrimitiveType::Uint20 => "uint20",
            PrimitiveType::Uint21 => "uint21",
            PrimitiveType::Uint22 => "uint22",
            PrimitiveType::Uint23 => "uint23",
            PrimitiveType::Uint24 => "uint24",
            PrimitiveType::Uint25 => "uint25",
            PrimitiveType::Uint26 => "uint26",
            PrimitiveType::Uint27 => "uint27",
            PrimitiveType::Uint28 => "uint28",
            PrimitiveType::Uint29 => "uint29",
            PrimitiveType::Uint30 => "uint30",
            PrimitiveType::Uint31 => "uint31",
            PrimitiveType::Uint32 => "uint32",

            PrimitiveType::Uint33 => "uint33",
            PrimitiveType::Uint34 => "uint34",
            PrimitiveType::Uint35 => "uint35",
            PrimitiveType::Uint36 => "uint36",
            PrimitiveType::Uint37 => "uint37",
            PrimitiveType::Uint38 => "uint38",
            PrimitiveType::Uint39 => "uint39",
            PrimitiveType::Uint40 => "uint40",
            PrimitiveType::Uint41 => "uint41",
            PrimitiveType::Uint42 => "uint42",
            PrimitiveType::Uint43 => "uint43",
            PrimitiveType::Uint44 => "uint44",
            PrimitiveType::Uint45 => "uint45",
            PrimitiveType::Uint46 => "uint46",
            PrimitiveType::Uint47 => "uint47",
            PrimitiveType::Uint48 => "uint48",
            PrimitiveType::Uint49 => "uint49",
            PrimitiveType::Uint50 => "uint50",
            PrimitiveType::Uint51 => "uint51",
            PrimitiveType::Uint52 => "uint52",
            PrimitiveType::Uint53 => "uint53",
            PrimitiveType::Uint54 => "uint54",
            PrimitiveType::Uint55 => "uint55",
            PrimitiveType::Uint56 => "uint56",
            PrimitiveType::Uint57 => "uint57",
            PrimitiveType::Uint58 => "uint58",
            PrimitiveType::Uint59 => "uint59",
            PrimitiveType::Uint60 => "uint60",
            PrimitiveType::Uint61 => "uint61",
            PrimitiveType::Uint62 => "uint62",
            PrimitiveType::Uint63 => "uint63",
            PrimitiveType::Uint64 => "uint64",

            PrimitiveType::Int2 => "int2",
            PrimitiveType::Int3 => "int3",
            PrimitiveType::Int4 => "int4",
            PrimitiveType::Int5 => "int5",
            PrimitiveType::Int6 => "int6",
            PrimitiveType::Int7 => "int7",
            PrimitiveType::Int8 => "int8",
            PrimitiveType::Int9 => "int9",
            PrimitiveType::Int10 => "int10",
            PrimitiveType::Int11 => "int11",
            PrimitiveType::Int12 => "int12",
            PrimitiveType::Int13 => "int13",
            PrimitiveType::Int14 => "int14",
            PrimitiveType::Int15 => "int15",
            PrimitiveType::Int16 => "int16",
            PrimitiveType::Int17 => "int17",
            PrimitiveType::Int18 => "int18",
            PrimitiveType::Int19 => "int19",
            PrimitiveType::Int20 => "int20",
            PrimitiveType::Int21 => "int21",
            PrimitiveType::Int22 => "int22",
            PrimitiveType::Int23 => "int23",
            PrimitiveType::Int24 => "int24",
            PrimitiveType::Int25 => "int25",
            PrimitiveType::Int26 => "int26",
            PrimitiveType::Int27 => "int27",
            PrimitiveType::Int28 => "int28",
            PrimitiveType::Int29 => "int29",
            PrimitiveType::Int30 => "int30",
            PrimitiveType::Int31 => "int31",
            PrimitiveType::Int32 => "int32",

            PrimitiveType::Int33 => "int33",
            PrimitiveType::Int34 => "int34",
            PrimitiveType::Int35 => "int35",
            PrimitiveType::Int36 => "int36",
            PrimitiveType::Int37 => "int37",
            PrimitiveType::Int38 => "int38",
            PrimitiveType::Int39 => "int39",
            PrimitiveType::Int40 => "int40",
            PrimitiveType::Int41 => "int41",
            PrimitiveType::Int42 => "int42",
            PrimitiveType::Int43 => "int43",
            PrimitiveType::Int44 => "int44",
            PrimitiveType::Int45 => "int45",
            PrimitiveType::Int46 => "int46",
            PrimitiveType::Int47 => "int47",
            PrimitiveType::Int48 => "int48",
            PrimitiveType::Int49 => "int49",
            PrimitiveType::Int50 => "int50",
            PrimitiveType::Int51 => "int51",
            PrimitiveType::Int52 => "int52",
            PrimitiveType::Int53 => "int53",
            PrimitiveType::Int54 => "int54",
            PrimitiveType::Int55 => "int55",
            PrimitiveType::Int56 => "int56",
            PrimitiveType::Int57 => "int57",
            PrimitiveType::Int58 => "int58",
            PrimitiveType::Int59 => "int59",
            PrimitiveType::Int60 => "int60",
            PrimitiveType::Int61 => "int61",
            PrimitiveType::Int62 => "int62",
            PrimitiveType::Int63 => "int63",
            PrimitiveType::Int64 => "int64",

            PrimitiveType::Void1 => "void1",
            PrimitiveType::Void2 => "void2",
            PrimitiveType::Void3 => "void3",
            PrimitiveType::Void4 => "void4",
            PrimitiveType::Void5 => "void5",
            PrimitiveType::Void6 => "void6",
            PrimitiveType::Void7 => "void7",
            PrimitiveType::Void8 => "void8",
            PrimitiveType::Void9 => "void9",
            PrimitiveType::Void10 => "void10",
            PrimitiveType::Void11 => "void11",
            PrimitiveType::Void12 => "void12",
            PrimitiveType::Void13 => "void13",
            PrimitiveType::Void14 => "void14",
            PrimitiveType::Void15 => "void15",
            PrimitiveType::Void16 => "void16",
            PrimitiveType::Void17 => "void17",
            PrimitiveType::Void18 => "void18",
            PrimitiveType::Void19 => "void19",
            PrimitiveType::Void20 => "void20",
            PrimitiveType::Void21 => "void21",
            PrimitiveType::Void22 => "void22",
            PrimitiveType::Void23 => "void23",
            PrimitiveType::Void24 => "void24",
            PrimitiveType::Void25 => "void25",
            PrimitiveType::Void26 => "void26",
            PrimitiveType::Void27 => "void27",
            PrimitiveType::Void28 => "void28",
            PrimitiveType::Void29 => "void29",
            PrimitiveType::Void30 => "void30",
            PrimitiveType::Void31 => "void31",
            PrimitiveType::Void32 => "void32",

            PrimitiveType::Void33 => "void33",
            PrimitiveType::Void34 => "void34",
            PrimitiveType::Void35 => "void35",
            PrimitiveType::Void36 => "void36",
            PrimitiveType::Void37 => "void37",
            PrimitiveType::Void38 => "void38",
            PrimitiveType::Void39 => "void39",
            PrimitiveType::Void40 => "void40",
            PrimitiveType::Void41 => "void41",
            PrimitiveType::Void42 => "void42",
            PrimitiveType::Void43 => "void43",
            PrimitiveType::Void44 => "void44",
            PrimitiveType::Void45 => "void45",
            PrimitiveType::Void46 => "void46",
            PrimitiveType::Void47 => "void47",
            PrimitiveType::Void48 => "void48",
            PrimitiveType::Void49 => "void49",
            PrimitiveType::Void50 => "void50",
            PrimitiveType::Void51 => "void51",
            PrimitiveType::Void52 => "void52",
            PrimitiveType::Void53 => "void53",
            PrimitiveType::Void54 => "void54",
            PrimitiveType::Void55 => "void55",
            PrimitiveType::Void56 => "void56",
            PrimitiveType::Void57 => "void57",
            PrimitiveType::Void58 => "void58",
            PrimitiveType::Void59 => "void59",
            PrimitiveType::Void60 => "void60",
            PrimitiveType::Void61 => "void61",
            PrimitiveType::Void62 => "void62",
            PrimitiveType::Void63 => "void63",
            PrimitiveType::Void64 => "void64",

            PrimitiveType::Float16 => "float16",
            PrimitiveType::Float32 => "float32",
            PrimitiveType::Float64 => "float64",
        })
    }
}
