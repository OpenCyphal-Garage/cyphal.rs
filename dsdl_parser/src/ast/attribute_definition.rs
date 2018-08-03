//! Everything related to uavcan attribute definitions.
//!
//! An attribute definition can either be a const definition or field definition

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use ast::cast_mode::CastMode;
use ast::ty::Ty;
use ast::ty::CompositeType;
use ast::array::ArrayInfo;
use ast::ident::Ident;
use ast::lit::Lit;
use ast::file_name::FileName;

/// An attribute definition is either a `FieldDefintion` or a `ConstDefinition`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeDefinition {
    Field(FieldDefinition),
    Const(ConstDefinition),
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
    pub array: Option<ArrayInfo>,
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
    pub literal: Lit,
}



impl AttributeDefinition {
    pub(crate) fn normalize(self, file_name: &FileName) -> Option<Self> {
        match self {
            AttributeDefinition::Field(def) => Some(AttributeDefinition::Field(def.normalize(file_name))),
            // 2. Remove all constant definitions
            AttributeDefinition::Const(_) => None,
        }
    }
}

impl FieldDefinition {
    pub(crate) fn normalize(self, file_name: &FileName) -> Self {
        match self {
            // 3. Ensure that all cast specifiers are explicitly defined; if not, add default cast specifiers.
            FieldDefinition{cast_mode: None, field_type: Ty::Primitive(primitive_type), array, name} =>
                if primitive_type.is_void() {
                    FieldDefinition{
                        cast_mode: None,
                        field_type: Ty::Primitive(primitive_type),
                        array: array.map(|x| x.normalize()),
                        name: name,
                    }
                } else {
                    FieldDefinition{
                        cast_mode: Some(CastMode::Saturated),
                        field_type: Ty::Primitive(primitive_type),
                        array: array.map(|x| x.normalize()),
                        name: name,
                    }
                },
            // 5. For nested data structures, replace all short names with full names.
            FieldDefinition{field_type: Ty::Composite(CompositeType{namespace: None, name: type_name}), cast_mode, array, name: field_name} =>
                FieldDefinition{
                    cast_mode: cast_mode,
                    field_type: Ty::Composite(CompositeType{namespace: Some(Ident::from(file_name.clone().namespace)), name: type_name}),
                    array: array.map(|x| x.normalize()),
                    name: field_name,
                },
            x => x,
        }
    }
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

impl Display for AttributeDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            AttributeDefinition::Field(ref field_def) => write!(f, "{}", field_def),
            AttributeDefinition::Const(ref const_def) => write!(f, "{}", const_def),
        }
    }
}

impl Display for FieldDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self.cast_mode {
            Some(ref x) => write!(f, "{} ", x)?,
            None => ()
        };

        write!(f, "{}", self.field_type)?;

        match self.array {
            Some(ref x) => write!(f, "{}", x)?,
            None => ()
        };

        match self.name {
            Some(ref x) => write!(f, " {}", x),
            None => Ok(()),
        }
    }
}

impl Display for ConstDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self.cast_mode {
            Some(ref x) => write!(f, "{} ", x)?,
            None => ()
        };

        write!(f, "{} {} = {}", self.field_type, self.name, self.literal)
    }
}
