//! A parser for the DSDL (Data structure description language) used in [uavcan](http://uavcan.org)
//!
//! For full description have a look at the [specification](http://uavcan.org/Specification/3._Data_structure_description_language/)
//!
//! ## Examples
//! ### Parse DSDL directory
//!
//! ```
//! use dsdl_parser::DSDL;
//!
//! assert!(DSDL::read("tests/dsdl/").is_ok());
//!
//! ```
//!
//! ### Parse single file
//!
//! ```
//! use dsdl_parser::DSDL;
//! 
//! assert!(DSDL::read("tests/dsdl/uavcan/protocol/341.NodeStatus.uavcan").is_ok());
//! 
//! ```
//!
//! ### Display a file
//!
//! ```
//! use dsdl_parser::DSDL;
//!
//! let dsdl = DSDL::read("./tests/dsdl/").unwrap();
//!
//! println!("{}", dsdl.get_file("uavcan.protocol.GetNodeInfo").unwrap());
//! 
//! ```
//!
//! ### Calculate data type signature
//!
//! ```
//! use dsdl_parser::DSDL;
//!
//! let dsdl = DSDL::read("./tests/dsdl/").unwrap();
//!
//! assert_eq!(dsdl.data_type_signature("uavcan.protocol.GetNodeInfo").unwrap(), 0xee468a8121c46a9e);
//! ```



#[macro_use]
extern crate log;

extern crate lalrpop_util;



mod lexer;
mod ast;
mod parser;
mod crc;





// Re-export ast

pub use ast::ty::Ty;
pub use ast::ty::CompositeType;
pub use ast::ty::PrimitiveType;

pub use ast::ident::Ident;

pub use ast::comment::Comment;

pub use ast::directive::Directive;

pub use ast::attribute_definition::AttributeDefinition;
pub use ast::attribute_definition::ConstDefinition;
pub use ast::attribute_definition::FieldDefinition;

pub use ast::file_name::FileName;
pub use ast::file_name::Version;

pub use ast::array::ArrayInfo;

pub use ast::cast_mode::CastMode;

pub use ast::lit::Lit;
pub use ast::lit::Sign;

pub use ast::type_definition::TypeDefinition;
pub use ast::type_definition::MessageDefinition;
pub use ast::type_definition::ServiceDefinition;

pub use ast::line::Line;

pub use ast::file::File;
pub use ast::file::NormalizedFile;

pub use ast::dsdl::DSDL;





#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use *;
    #[test]
    fn read_node_status() {
        let dsdl = DSDL::read("tests/dsdl/uavcan/protocol/341.NodeStatus.uavcan").unwrap();
        
        assert_eq!(dsdl.get_file(&String::from("NodeStatus")).unwrap(),
                   &File {
                       name: FileName {
                           id: Some(341),
                           namespace: String::from(""),
                           name: String::from("NodeStatus"),
                           version: None,
                       },
                       definition: TypeDefinition::Message(MessageDefinition(vec!(
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Abstract node status information.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Any UAVCAN node is required to publish this message periodically.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Publication period may vary within these limits.").unwrap()),
                           Line::Comment(Comment::from_str("# It is NOT recommended to change it at run time.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), name: Ident::from_str("MAX_BROADCASTING_PERIOD_MS").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("1000")} }), comment: None},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), name: Ident::from_str("MIN_BROADCASTING_PERIOD_MS").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("2")} }), comment: None},
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# If a node fails to publish this message in this amount of time, it should be considered offline.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), name: Ident::from_str("OFFLINE_TIMEOUT_MS").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("3000")} }), comment: None},
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Uptime counter should never overflow.").unwrap()),
                           Line::Comment(Comment::from_str("# Other nodes may detect that a remote node has restarted when this value goes backwards.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint32), array: None, name: Some(Ident::from_str("uptime_sec").unwrap()) }), comment: None},
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Abstract node health.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident::from_str("HEALTH_OK").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("0")} }), comment: Some(Comment::from_str("# The node is functioning properly.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident::from_str("HEALTH_WARNING").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("1")} }), comment: Some(Comment::from_str("# A critical parameter went out of range or the node encountered a minor failure.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident::from_str("HEALTH_ERROR").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("2")} }), comment: Some(Comment::from_str("# The node encountered a major failure.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), name: Ident::from_str("HEALTH_CRITICAL").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("3")} }), comment: Some(Comment::from_str("# The node suffered a fatal malfunction.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint2), array: None, name: Some(Ident::from_str("health").unwrap()) }), comment: None},
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Current mode.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Mode OFFLINE can be actually reported by the node to explicitly inform other network").unwrap()),
                           Line::Comment(Comment::from_str("# participants that the sending node is about to shutdown. In this case other nodes will not").unwrap()),
                           Line::Comment(Comment::from_str("# have to wait OFFLINE_TIMEOUT_MS before they detect that the node is no longer available.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Reserved values can be used in future revisions of the specification.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident::from_str("MODE_OPERATIONAL").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("0")} }), comment: Some(Comment::from_str("# Normal operating mode.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident::from_str("MODE_INITIALIZATION").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("1")} }), comment: Some(Comment::from_str("# Initialization is in progress; this mode is entered immediately after startup.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident::from_str("MODE_MAINTENANCE").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("2")} }), comment: Some(Comment::from_str("# E.g. calibration, the bootloader is running, etc.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident::from_str("MODE_SOFTWARE_UPDATE").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("3")} }), comment: Some(Comment::from_str("# New software/firmware is being loaded.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Const(ConstDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), name: Ident::from_str("MODE_OFFLINE").unwrap(), literal: Lit::Dec{sign: Sign::Implicit, value: String::from("7")} }), comment: Some(Comment::from_str("# The node is no longer available.").unwrap())},
                           Line::Definition{definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), array: None, name: Some(Ident::from_str("mode").unwrap()) }), comment: None},
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Not used currently, keep zero when publishing, ignore when receiving.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint3), array: None, name: Some(Ident::from_str("sub_mode").unwrap()) }), comment: None},
                           Line::Empty,
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Comment(Comment::from_str("# Optional, vendor-specific node status code, e.g. a fault code or a status bitmask.").unwrap()),
                           Line::Comment(Comment::from_str("#").unwrap()),
                           Line::Definition{definition: AttributeDefinition::Field(FieldDefinition { cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Uint16), array: None, name: Some(Ident::from_str("vendor_specific_status_code").unwrap()) }), comment: None},
                       ))),}
        );
    }
}

