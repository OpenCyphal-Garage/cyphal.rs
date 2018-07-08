use *;
use crc::CRC64WE as CRC;

/// A normalized `File`
///
/// The underlying file can't be exposed in a mutable way. Any change to the normalized format would "denormalize" it.
/// The `NormalizedFile` can be used to calculate the DSDL signature
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedFile(File);

impl NormalizedFile {
    /// Return a reference to the underlying `File`
    pub fn as_file<'a>(&'a self) -> &'a File {
        &self.0
    }

    /// Turn the `NormalizedFile` into the underlying `File`
    pub fn into_file(self) -> File {
        self.0
    }

    /// Calculate the DSDL signature
    pub fn dsdl_signature(&self) -> u64 {
        let mut crc = CRC::new();
        crc.add(format!("{}", self).as_bytes());
        crc.value()
    }
        
}

impl File {
    /// Normalizes a file according to the uavcan specification.
    pub fn normalize(self) -> NormalizedFile {
        let definition = self.definition.normalize(&self.name);
        NormalizedFile(File{name: self.name, definition: definition})
    }
}

impl TypeDefinition {
    fn normalize(self, file_name: &FileName) -> Self {
        match self {
            TypeDefinition::Message(x) => TypeDefinition::Message(x.normalize(file_name)),
            TypeDefinition::Service(x) => TypeDefinition::Service(x.normalize(file_name)),
        }
    }
}

impl ServiceDefinition {
    fn normalize(self, file_name: &FileName) -> Self {
        ServiceDefinition{request: self.request.normalize(file_name), response: self.response.normalize(file_name)}
    }
}

impl MessageDefinition {
    fn normalize(self, file_name: &FileName) -> Self {
        let mut normalized_lines = Vec::new();
        for line in self.0 {
            match line.normalize(file_name) {
                Some(x) => normalized_lines.push(x),
                None => (),
            }
        }
        MessageDefinition(normalized_lines)        
    }
}

impl Line {
    fn normalize(self, file_name: &FileName) -> Option<Self> {
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

impl AttributeDefinition {
    fn normalize(self, file_name: &FileName) -> Option<Self> {
        match self {
            AttributeDefinition::Field(def) => Some(AttributeDefinition::Field(def.normalize(file_name))),
            // 2. Remove all constant definitions
            AttributeDefinition::Const(_) => None,
        }
    }
}

impl FieldDefinition {
    fn normalize(self, file_name: &FileName) -> Self {
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

impl ArrayInfo {
    fn normalize(self) -> Self {
        // 4. For dynamic arrays, replace the max length specifier in the form [<X] to the form [<=Y]
        match self {
            ArrayInfo::DynamicLess(num) => ArrayInfo::DynamicLeq(num-1),
            x => x,
        }
    }
}
