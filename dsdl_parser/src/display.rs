use std::fmt::Error;
use std::fmt::Formatter;
use std::fmt::Display;

use *;

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.namespace.as_str() != "" {
            write!(f, "{}.", self.namespace)?;
        }
        if let Some(ref id) = self.id {
            write!(f, "{}.", id)?;
        }
        write!(f, "{}", self.name)?;
        if let Some(ref version) = self.version {
            write!(f, ".{}", version)?;
        }
        write!(f, ".uavcan")            
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
    

impl Display for CastMode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            CastMode::Saturated => write!(f, "saturated"),
            CastMode::Truncated => write!(f, "truncated"),
        }
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for Directive {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Directive::Union => write!(f, "@union"),
        }
    }
}
    
impl Display for ArrayInfo {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            ArrayInfo::Single => write!(f, ""),
            ArrayInfo::Dynamic(ref num) => write!(f, "[<={}]", num),
            ArrayInfo::Static(ref num) => write!(f, "[{}]", num),
        }
    }
}

impl Display for Const {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Const::Dec(ref x) => write!(f, "{}", x),
            Const::Hex(ref x) => write!(f, "0x{}", x),
            Const::Bin(ref x) => write!(f, "0b{}", x),
            Const::Bool(ref x) => write!(f, "{}", match *x {true => "true", false => "false"}),
            Const::Char(ref x) => write!(f, "{}", x),
        }
    }
}

impl Display for FieldDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.cast_mode {
            Some(ref x) => write!(f, "{} ", x)?,
            None => ()
        };

        write!(f, "{}{}", self.field_type, self.array)?;

        match self.name {
            Some(ref x) => write!(f, " {}", x),
            None => Ok(()),
        }
    }
}

impl Display for ConstDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.cast_mode {
            Some(ref x) => write!(f, "{} ", x)?,
            None => ()
        };

        write!(f, "{} {} = {}", self.field_type, self.name, self.constant)
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Ty::Primitive(ref ty) => write!(f, "{}", ty),
            Ty::Composite(ref ty) => write!(f, "{}", ty),
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Line::Empty => write!(f, ""),
            Line::Comment(ref comment) => write!(f, "#{}", comment),
            Line::Definition(ref def, ref opt_comment) => {
                match *opt_comment {
                    Some(ref comment) => write!(f, "{} #{}", def, comment),
                    None => write!(f, "{}", def),
                }
            },
            Line::Directive(ref dir, ref opt_comment) => {
                match *opt_comment {
                    Some(ref comment) => write!(f, "{} #{}", dir, comment),
                    None => write!(f, "{}", dir),
                }
            },
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "File: {}\n{}", self.name, self.definition)
    }
}

impl Display for AttributeDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            AttributeDefinition::Field(ref field_def) => write!(f, "{}", field_def),
            AttributeDefinition::Const(ref const_def) => write!(f, "{}", const_def),
        }
    }
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            TypeDefinition::Message(ref x) => write!(f, "{}", x),
            TypeDefinition::Service(ref x) => write!(f, "{}", x),
        }
    }
}

impl Display for MessageDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for (i, line) in self.0.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", line)?;
            } else {
                write!(f, "\n{}", line)?;
            }
        }
        Ok(())
    }
}

impl Display for ServiceDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}\n---\n{}", self.request, self.response)
    }
}

impl Display for CompositeType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.namespace {
            Some(ref namespace) => write!(f, "{}.{}", namespace, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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







#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn display_line() {
        assert_eq!(format!("{}", Line::Empty), "");   

        assert_eq!(format!("{}", Line::Comment(Comment(String::from(" test comment")))), "# test comment");   

        assert_eq!(format!("{}",
                           Line::Definition(
                               AttributeDefinition::Field(FieldDefinition{
                                   cast_mode: None,
                                   field_type: Ty::Primitive(PrimitiveType::Uint32),
                                   array: ArrayInfo::Single,
                                   name: Some(Ident(String::from("uptime_sec"))),
                               }),
                               None)),
                   "uint32 uptime_sec"
        );

        assert_eq!(format!("{}",
                           Line::Definition(
                               AttributeDefinition::Field(FieldDefinition{
                                   cast_mode: None,
                                   field_type: Ty::Primitive(PrimitiveType::Uint32),
                                   array: ArrayInfo::Single,
                                   name: Some(Ident(String::from("uptime_sec"))),
                               }),
                               Some(Comment(String::from(" test comment"))))),
                   "uint32 uptime_sec # test comment"
        );

        assert_eq!(format!("{}",
                           Line::Definition(
                               AttributeDefinition::Const(ConstDefinition{
                                   cast_mode: None,
                                   field_type: Ty::Primitive(PrimitiveType::Uint2),
                                   name: Ident(String::from("HEALTH_OK")),
                                   constant: Const::Dec(String::from("0")),
                               }),
                               Some(Comment(String::from(" test comment"))))),
                   "uint2 HEALTH_OK = 0 # test comment"
        );

        assert_eq!(format!("{}",
                           Line::Directive(
                               Directive::Union,
                               Some(Comment(String::from(" test comment"))))),
                   "@union # test comment"
        );
                           
           
    }
}
