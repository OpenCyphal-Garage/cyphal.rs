#[macro_use]
extern crate nom;

use std::io::Read;

use std::fs;

use std::path::Path;

use std::str;
use std::str::FromStr;

use std::collections::HashMap;

mod parse;



#[derive(Debug, PartialEq, Eq)]
pub struct DSDL {
    pub files: HashMap<String, File>,
}

impl DSDL {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<DSDL> {
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
        } else {
            let mut file = fs::File::open(path)?;
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let bytes_slice = bytes.into_boxed_slice();
            let (remaining, definition) = parse::type_definition(&bytes_slice).unwrap();
            
            assert_eq!(remaining, &b""[..], "Parsing failed at file: {}, with the following data remaining: {}", uavcan_path, str::from_utf8(remaining).unwrap());
            
            let mut partial_name = file_name.rsplit('.');
            if partial_name.next() == Some("uavcan") {
                let type_name = String::from(partial_name.next().unwrap());
                let id = match partial_name.next() {
                    Some(s) => Some(String::from(s)),
                    None => None,
                };
                let qualified_name = if namespace == "" {
                    type_name.clone()
                } else {
                    namespace.clone() + "." + type_name.as_str()
                };
                files.insert(qualified_name, File{id: id, namespace: namespace.clone(), name: type_name, definition: definition});
            }
        }
    
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    id: Option<String>,
    namespace: String,
    name: String,
    definition: TypeDefinition,
}

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


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageDefinition(Vec<Line>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceDefinition{
    request: MessageDefinition,
    response: MessageDefinition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line (Option<AttributeDefinition>, Option<Comment>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedPath {
    namespace: Ident,
    name: Ident,
}
        
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    Short(Ident),
    Qualified(QualifiedPath)
}    




#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment(String);

impl<'a> From<&'a str> for Comment {
    fn from(s: &'a str) -> Comment {
        Comment(String::from(s))
    }
}

pub struct ServiceResponseMarker {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(String);

impl<'a> From<&'a str> for Ident {
    fn from(s: &'a str) -> Ident {
        Ident(String::from(s))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Num(String);

impl FromStr for Num {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: fix sanitizing
        Ok(Num(String::from(s)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Dec(String),
    Hex(String),
    Bin(String),
    Bool(bool),
}
// TODO: consider using this instead of Num for array lengths


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayInfo {
    Single,
    Dynamic(Num),
    Static(Num),
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoidDefinition {
    pub field_type: PrimitiveType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldDefinition {
    pub cast_mode: Option<CastMode>,
    pub field_type: Ty,
    pub array: ArrayInfo,
    pub name: Ident,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstDefinition {
    pub cast_mode: Option<CastMode>,
    pub field_type: Ty,
    pub name: Ident,
    pub constant: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeDefinition {
    Field(FieldDefinition),
    Const(ConstDefinition),
    Void(VoidDefinition), 
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

impl From<VoidDefinition> for AttributeDefinition {
    fn from(d: VoidDefinition) -> Self {
        AttributeDefinition::Void(d)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            "uint16" => Ok(PrimitiveType::Uint16),
            "uint32" => Ok(PrimitiveType::Uint32),
            "uint40" => Ok(PrimitiveType::Uint40),
            "uint48" => Ok(PrimitiveType::Uint48),
            "uint56" => Ok(PrimitiveType::Uint56),
            "uint64" => Ok(PrimitiveType::Uint64),
            
            "void2" => Ok(PrimitiveType::Void2),
            "void3" => Ok(PrimitiveType::Void3),
            "void6" => Ok(PrimitiveType::Void6),
            "void22" => Ok(PrimitiveType::Void22),
            "void32" => Ok(PrimitiveType::Void32),
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
    
    use nom::{
        IResult,
    };
    
    #[test]
    fn read_node_status() {
        let dsdl = DSDL::open("tests/dsdl/uavcan/protocol/341.NodeStatus.uavcan").unwrap();
        
        assert_eq!(dsdl.files.get(&String::from("NodeStatus")).unwrap(),
                   &File {
                       id: Some(String::from("341")),
                       namespace: String::from(""),
                       name: String::from("NodeStatus"),
                       definition: TypeDefinition::Message(MessageDefinition(vec!(
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Abstract node status information.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Any UAVCAN node is required to publish this message periodically.")))),
                           Line(None, Some(Comment(String::new()))), Line(None, Some(Comment(String::from("")))),
                           Line(None, Some(Comment(String::from(" Publication period may vary within these limits.")))),
                           Line(None, Some(Comment(String::from(" It is NOT recommended to change it at run time.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint16), name: Ident(String::from("MAX_BROADCASTING_PERIOD_MS")), constant: Value::Dec(String::from("1000")) })), None),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint16), name: Ident(String::from("MIN_BROADCASTING_PERIOD_MS")), constant: Value::Dec(String::from("2")) })), None),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" If a node fails to publish this message in this amount of time, it should be considered offline.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint16), name: Ident(String::from("OFFLINE_TIMEOUT_MS")), constant: Value::Dec(String::from("3000")) })), None),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Uptime counter should never overflow.")))),
                           Line(None, Some(Comment(String::from(" Other nodes may detect that a remote node has restarted when this value goes backwards.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(Some(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint32), array: ArrayInfo::Single, name: Ident(String::from("uptime_sec")) })), None),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Abstract node health.")))), Line(None, Some(Comment(String::from("")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_OK")), constant: Value::Dec(String::from("0")) })), Some(Comment(String::from(" The node is functioning properly.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_WARNING")), constant: Value::Dec(String::from("1")) })), Some(Comment(String::from(" A critical parameter went out of range or the node encountered a minor failure.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_ERROR")), constant: Value::Dec(String::from("2")) })), Some(Comment(String::from(" The node encountered a major failure.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint2), name: Ident(String::from("HEALTH_CRITICAL")), constant: Value::Dec(String::from("3")) })), Some(Comment(String::from(" The node suffered a fatal malfunction.")))),
                           Line(Some(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint2), array: ArrayInfo::Single, name: Ident(String::from("health")) })), None), Line(None, Some(Comment(String::from("")))),
                           Line(None, Some(Comment(String::from(" Current mode.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Mode OFFLINE can be actually reported by the node to explicitly inform other network")))),
                           Line(None, Some(Comment(String::from(" participants that the sending node is about to shutdown. In this case other nodes will not")))),
                           Line(None, Some(Comment(String::from(" have to wait OFFLINE_TIMEOUT_MS before they detect that the node is no longer available.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Reserved values can be used in future revisions of the specification.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), name: Ident(String::from("MODE_OPERATIONAL")), constant: Value::Dec(String::from("0")) })), Some(Comment(String::from(" Normal operating mode.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), name: Ident(String::from("MODE_INITIALIZATION")), constant: Value::Dec(String::from("1")) })), Some(Comment(String::from(" Initialization is in progress; this mode is entered immediately after startup.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), name: Ident(String::from("MODE_MAINTENANCE")), constant: Value::Dec(String::from("2")) })), Some(Comment(String::from(" E.g. calibration, the bootloader is running, etc.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), name: Ident(String::from("MODE_SOFTWARE_UPDATE")), constant: Value::Dec(String::from("3")) })), Some(Comment(String::from(" New software/firmware is being loaded.")))),
                           Line(Some(AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), name: Ident(String::from("MODE_OFFLINE")), constant: Value::Dec(String::from("7")) })), Some(Comment(String::from(" The node is no longer available.")))),
                           Line(Some(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), array: ArrayInfo::Single, name: Ident(String::from("mode")) })), None),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Not used currently, keep zero when publishing, ignore when receiving.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(Some(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint3), array: ArrayInfo::Single, name: Ident(String::from("sub_mode")) })), None),
                           Line(None, Some(Comment(String::new()))),
                           Line(None, Some(Comment(String::from(" Optional, vendor-specific node status code, e.g. a fault code or a status bitmask.")))),
                           Line(None, Some(Comment(String::new()))),
                           Line(Some(AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::PrimitiveType(PrimitiveType::Uint16), array: ArrayInfo::Single, name: Ident(String::from("vendor_specific_status_code")) })), None),
                       ))),}
        );


        
        //assert_eq!(dsdl, DSDL::open("tests/dsdl/uavcan/protocol/1.GetNodeInfo.uavcan").unwrap());
        
    }
}

