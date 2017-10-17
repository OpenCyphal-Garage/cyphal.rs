#[macro_use]
extern crate nom;

#[macro_use]
extern crate log;

use std::io::Read;

use std::fs;

use std::path::Path;

use std::str;
use std::str::FromStr;

use std::collections::HashMap;

use nom::IResult;

mod parse;
mod display;
mod normalize;

pub use normalize::NormalizedFile;

/// The `DSDL` struct contains a number of data type definition
#[derive(Debug, PartialEq, Eq)]
pub struct DSDL {
    pub files: HashMap<String, File>,
}

impl DSDL {
    /// Reads `DSDL` definition recursively if path is a directory. Reads one `DSDL` definition if path is a definition. 
    pub fn read<P: AsRef<Path>>(path: P) -> std::io::Result<DSDL> {
        let mut dsdl = DSDL{files: HashMap::new()};

        DSDL::read_uavcan_files(path.as_ref(), String::new(), &mut dsdl.files)?;

        Ok(dsdl)
    }

    fn read_uavcan_files(path: &Path, namespace: String, files: &mut HashMap<String, File>) -> std::io::Result<()> {
        let uavcan_path = if namespace.as_str() == "" {
            String::from(path.file_name().unwrap().to_str().unwrap())
        } else {
            namespace.clone() + "." + path.file_name().unwrap().to_str().unwrap()
        };
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let current_path = entry?.path();
                DSDL::read_uavcan_files(&current_path, uavcan_path.clone(), files)?;
            }
        } else if let IResult::Done(_i, file_name) = parse::file_name(uavcan_path.as_bytes()) {
            let mut file = fs::File::open(path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let bytes_slice = bytes.into_boxed_slice();
            let (remaining, definition) = parse::type_definition(&bytes_slice).unwrap();
            
            assert!(remaining == &b""[..], "Parsing failed at file: {}, with the following data remaining: {}", uavcan_path, str::from_utf8(remaining).unwrap());
                                
            let qualified_name = if file_name.namespace.as_str() == "" {
                file_name.name.clone()
            } else {
                file_name.namespace.clone() + "." + file_name.name.as_str()
            };
            files.insert(qualified_name, File{name: file_name, definition: definition});
        } else {
            warn!("The file, {}, was not recognized as a DSDL file. DSDL files need to have the .uavcan extension", uavcan_path);
        }
        
        Ok(())
    }
}

/// Uniquely defines a DSDL file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileName {
    pub id: Option<String>,
    pub namespace: String,
    pub name: String,
    pub version: Option<Version>,
}

impl FromStr for FileName {
    type Err = String;

    fn from_str(s: &str) -> Result<FileName, Self::Err> {
        let mut split = s.rsplit('.').peekable();
        
        if let Some("uavcan") = split.next() {
        } else {
            return Err(String::from("Error while parsing, {}, does file end with .uavcan?"));
        }

        let version = match u32::from_str(split.peek().ok_or(String::from("Error while parsing, {}, bad formated filename"))?) {
            Ok(minor_version) => {
                split.next().unwrap(); // remove minor version
                let major_version = u32::from_str(split.next().ok_or(String::from("Error while parsing, {}, is versioning formated correctly?\n    versioning should be formated as <major>.<minor>"))?).unwrap();
                Some(Version{major: major_version, minor: minor_version})
            },
            Err(_) => None,
        };

        let name = String::from(split.next().unwrap());

        let id = if let IResult::Done(_i, o) = parse::id(split.peek().unwrap_or(&"").as_bytes()) {
            split.next().unwrap();
            Some(o)
        } else {
            None
        };

        let mut namespace = String::from(split.next().unwrap_or(""));
        while let Some(namespace_part) = split.next() {
            namespace = String::from(namespace_part) + "." + namespace.as_str();
        }

        Ok(FileName{id: id, namespace: namespace, name: name, version: version})
    }
}


/// A dsdl file version
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

/// A DSDL file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub name: FileName,
    pub definition: TypeDefinition,
}

/// A DSDL type definition.
///
/// Each DSDL definition specifies exactly one data structure that can be used for message broadcasting
/// or a pair of structures that can be used for service invocation data exchange.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDefinition {
    Message(MessageDefinition),
    Service(ServiceDefinition),
}

impl From<MessageDefinition> for TypeDefinition {
    fn from(d: MessageDefinition) -> Self {
        TypeDefinition::Message(d)
    }
}

impl From<ServiceDefinition> for TypeDefinition {
    fn from(d: ServiceDefinition) -> Self {
        TypeDefinition::Service(d)
    }
}



/// An Uavcan message definition
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageDefinition(pub Vec<Line>);

/// An Uavcan service definition
///
/// Since a service invocation consists of two network exchange operations,
/// the DSDL definition for a service must define two structures:
///
/// - Request part - for request transfer (client to server).
/// - Response part - for response transfer (server to client).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceDefinition{
    /// The request part - for request transfer (client to server)
    pub request: MessageDefinition,
    /// The response part - for response transfer (server to client)
    pub response: MessageDefinition,
}

/// A `Line` in a DSDL `File`
///
/// A data structure definition consists of attributes and directives.
/// Any line of the definition file may contain at most one attribute definition or at most one directive.
/// The same line cannot contain an attribute definition and a directive at the same time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Empty,
    Comment(Comment),
    Definition(AttributeDefinition, Option<Comment>),
    Directive(Directive, Option<Comment>),
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
            let name = Ident(String::from(split.next().unwrap()));
            let namespace = match split.next() {
                Some(x) => Some(Ident(String::from(x))),
                None => None,
            };
            Ok(CompositeType {
                namespace: namespace,
                name: name,
            })
        } else {
            Ok(CompositeType {
                namespace: None,
                name: Ident(String::from(s))
            })
        }
    }
}

/// A comment
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment(String);

impl<'a> From<&'a str> for Comment {
    fn from(s: &'a str) -> Comment {
        Comment(String::from(s))
    }
}

/// An Uavcan Directive
///
/// A directive is a single case-sensitive word starting with an “at sign” (@),
/// possibly followed by space-separated arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Directive {
    Union,
}

impl FromStr for Directive {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Directive, Self::Err> {
        match s {
            "@union" => Ok(Directive::Union),
            "union" => Ok(Directive::Union),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ServiceResponseMarker {}

/// An Identifier (name)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(String);

impl<'a> From<&'a str> for Ident {
    fn from(s: &'a str) -> Ident {
        Ident(String::from(s))
    }
}

/// Used to determin size of e.g. DynamicArray or a StaticArray
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size(u64);

impl FromStr for Size {
    type Err = std::num::ParseIntError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Size(u64::from_str(s)?))
    }
}

impl From<u8> for Size {
    fn from(i: u8) -> Size {
        Size(u64::from(i))
    }
}

impl From<u16> for Size {
    fn from(i: u16) -> Size {
        Size(u64::from(i))
    }
}

impl From<u32> for Size {
    fn from(i: u32) -> Size {
        Size(u64::from(i))
    }
}

impl From<u64> for Size {
    fn from(i: u64) -> Size {
        Size(i)
    }
}

/// A constant must be a primitive scalar type (i.e., arrays and nested data structures are not allowed as constant types).
///
/// A constant must be assigned with a constant initializer, which must be one of the following:
/// 
/// - Integer zero (0).
/// - Integer literal in base 10, starting with a non-zero character. E.g., 123, -12.
/// - Integer literal in base 16 prefixed with 0x. E.g., 0x123, -0x12.
/// - Integer literal in base 2 prefixed with 0b. E.g., 0b1101, -0b101101.
/// - Integer literal in base 8 prefixed with 0o. E.g., 0o123, -0o777.
/// - Floating point literal. Fractional part with an optional exponent part, e.g., 15.75, 1.575E1, 1575e-2, -2.5e-3, 25E-4. Note that the use of infinity and NAN (not-a-number) is discouraged as it may not be supported on all platforms.
/// - Boolean true or false.
/// - Single ASCII character, ASCII escape sequence, or ASCII hex literal in single quotes. E.g., 'a', '\x61', '\n'.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Const {
    Dec(String),
    Hex(String),
    Bin(String),
    Bool(bool),
    Char(String),
}

/// Cast mode defines the rules of conversion from the native value of a certain programming language to the serialized field value.
///
/// Cast mode may be left undefined, in which case the default will be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastMode {
    /// This is the default cast mode, which will be used if the attribute definition does not specify the cast mode explicitly.
    ///
    /// For integers, it prevents an integer overflow - for example, attempting to write 0x44 to a 4-bit field will result in a bitfield value of 0x0F.
    /// For floating point values, it prevents overflow when casting to a lower precision floating point representation -
    /// for example, 65536.0 will be converted to a float16 as 65504.0; infinity will be preserved.
    Saturated,

    ///  For integers, it discards the excess most significant bits - for example, attempting to write 0x44 to a 4-bit field will produce 0x04.
    /// For floating point values, overflow during downcasting will produce an infinity.
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

/// Uavcan array information
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayInfo {
    /// Not an array (i.e. `uint2`)
    Single,
    /// Dynamic array on the less than form (i.e. `uint2[<5]`)
    DynamicLess(Size),
    /// Dynamic array on the less or equal form (i.e. `uint2[<=5]`)
    DynamicLeq(Size),
    /// Static array on the less or equal form (i.e. `uint2[5]`)
    Static(Size),
}


/// A Field definition
///
/// Field definition patterns
/// - `cast_mode field_type field_name`
/// - `cast_mode field_type[X] field_name`
/// - `cast_mode field_type[<X] field_name`
/// - `cast_mode field_type[<=X] field_name`
/// - `void_type`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldDefinition {
    pub cast_mode: Option<CastMode>,
    pub field_type: Ty,
    pub array: ArrayInfo,
    pub name: Option<Ident>,
}

/// A constant definition
///
/// Constant definition patterns
/// - `cast_mode constant_type constant_name = constant_initializer`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstDefinition {
    pub cast_mode: Option<CastMode>,
    pub field_type: Ty,
    pub name: Ident,
    pub constant: Const,
}

/// An attribute definition is either a `FieldDefintion` or a `ConstDefinition`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeDefinition {
    Field(FieldDefinition),
    Const(ConstDefinition),
}

impl From<FieldDefinition> for AttributeDefinition {
    fn from(d: FieldDefinition) -> Self {
        AttributeDefinition::Field(d)
    }
}

impl From<ConstDefinition> for AttributeDefinition {
    fn from(d: ConstDefinition) -> Self {
        AttributeDefinition::Const(d)
    }
}

/// An Uavcan data type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty{
    Primitive(PrimitiveType),
    Composite(CompositeType),
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

impl FromStr for PrimitiveType {
    type Err = ();
    
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
            _ => Err(()),
        }
    }
}

impl PrimitiveType {
    fn is_void(&self) -> bool {
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


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn read_node_status() {
        let dsdl = DSDL::read("tests/dsdl/uavcan/protocol/341.NodeStatus.uavcan").unwrap();
        
        assert_eq!(dsdl.files.get(&String::from("NodeStatus")).unwrap(),
                   &File {
                       name: FileName {
                           id: Some(String::from("341")),
                           namespace: String::from(""),
                           name: String::from("NodeStatus"),
                           version: None,
                       },
                       definition: TypeDefinition::Message(MessageDefinition(vec!(
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Abstract node status information."))),
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Any UAVCAN node is required to publish this message periodically."))),
                           Line::Comment(Comment(String::new())),
                           Line::Empty,
                           Line::Comment(Comment(String::from(""))),
                           Line::Comment(Comment(String::from(" Publication period may vary within these limits."))),
                           Line::Comment(Comment(String::from(" It is NOT recommended to change it at run time."))),
                           Line::Comment(Comment(String::new())),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), name: Ident(String::from("MAX_BROADCASTING_PERIOD_MS")), constant: Const::Dec(String::from("1000")) }), None),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), name: Ident(String::from("MIN_BROADCASTING_PERIOD_MS")), constant: Const::Dec(String::from("2")) }), None),
                           Line::Empty,
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" If a node fails to publish this message in this amount of time, it should be considered offline."))),
                           Line::Comment(Comment(String::new())),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), name: Ident(String::from("OFFLINE_TIMEOUT_MS")), constant: Const::Dec(String::from("3000")) }), None),
                           Line::Empty,
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Uptime counter should never overflow."))),
                           Line::Comment(Comment(String::from(" Other nodes may detect that a remote node has restarted when this value goes backwards."))),
                           Line::Comment(Comment(String::new())),
                           Line::Definition(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint32), array: ArrayInfo::Single, name: Some(Ident(String::from("uptime_sec"))) }), None),
                           Line::Empty,
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Abstract node health."))),
                           Line::Comment(Comment(String::from(""))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_OK")), constant: Const::Dec(String::from("0")) }), Some(Comment(String::from(" The node is functioning properly.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_WARNING")), constant: Const::Dec(String::from("1")) }), Some(Comment(String::from(" A critical parameter went out of range or the node encountered a minor failure.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_ERROR")), constant: Const::Dec(String::from("2")) }), Some(Comment(String::from(" The node encountered a major failure.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_CRITICAL")), constant: Const::Dec(String::from("3")) }), Some(Comment(String::from(" The node suffered a fatal malfunction.")))),
                           Line::Definition(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), array: ArrayInfo::Single, name: Some(Ident(String::from("health"))) }), None),
                           Line::Empty,
                           Line::Comment(Comment(String::from(""))),
                           Line::Comment(Comment(String::from(" Current mode."))),
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Mode OFFLINE can be actually reported by the node to explicitly inform other network"))),
                           Line::Comment(Comment(String::from(" participants that the sending node is about to shutdown. In this case other nodes will not"))),
                           Line::Comment(Comment(String::from(" have to wait OFFLINE_TIMEOUT_MS before they detect that the node is no longer available."))),
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Reserved values can be used in future revisions of the specification."))),
                           Line::Comment(Comment(String::new())),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident(String::from("MODE_OPERATIONAL")), constant: Const::Dec(String::from("0")) }), Some(Comment(String::from(" Normal operating mode.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident(String::from("MODE_INITIALIZATION")), constant: Const::Dec(String::from("1")) }), Some(Comment(String::from(" Initialization is in progress; this mode is entered immediately after startup.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident(String::from("MODE_MAINTENANCE")), constant: Const::Dec(String::from("2")) }), Some(Comment(String::from(" E.g. calibration, the bootloader is running, etc.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident(String::from("MODE_SOFTWARE_UPDATE")), constant: Const::Dec(String::from("3")) }), Some(Comment(String::from(" New software/firmware is being loaded.")))),
                           Line::Definition(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident(String::from("MODE_OFFLINE")), constant: Const::Dec(String::from("7")) }), Some(Comment(String::from(" The node is no longer available.")))),
                           Line::Definition(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), array: ArrayInfo::Single, name: Some(Ident(String::from("mode"))) }), None),
                           Line::Empty,
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Not used currently, keep zero when publishing, ignore when receiving."))),
                           Line::Comment(Comment(String::new())),
                           Line::Definition(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), array: ArrayInfo::Single, name: Some(Ident(String::from("sub_mode"))) }), None),
                           Line::Empty,
                           Line::Comment(Comment(String::new())),
                           Line::Comment(Comment(String::from(" Optional, vendor-specific node status code, e.g. a fault code or a status bitmask."))),
                           Line::Comment(Comment(String::new())),
                           Line::Definition(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), array: ArrayInfo::Single, name: Some(Ident(String::from("vendor_specific_status_code"))) }), None),
                       ))),}
        );


        
        //assert_eq!(dsdl, DSDL::open("tests/dsdl/uavcan/protocol/1.GetNodeInfo.uavcan").unwrap());
        
    }
}

