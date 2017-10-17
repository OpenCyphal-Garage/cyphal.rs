use *;


#[derive(Clone, Debug, PartialEq, Eq)]
/// A normalized `File`
///
/// The underlying file can't be exposed in a mutable way. Any change to the normalized format would "denormalize" it.
/// The `NormalizedFile` can be used to calculate the DSDL signature
///
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
        
}

impl File {
    /// Normalizes a file according to the uavcan specification.
    pub fn normalize(self) -> NormalizedFile {
        NormalizedFile(File{name: self.name, definition: self.definition.normalize()})
    }
}

impl TypeDefinition {
    fn normalize(self) -> Self {
        match self {
            TypeDefinition::Message(x) => TypeDefinition::Message(x.normalize()),
            TypeDefinition::Service(x) => TypeDefinition::Service(x.normalize()),
        }
    }
}

impl ServiceDefinition {
    fn normalize(self) -> Self {
        ServiceDefinition{request: self.request.normalize(), response: self.response.normalize()}
    }
}

impl MessageDefinition {
    fn normalize(self) -> Self {
        let mut normalized_lines = Vec::new();
        for line in self.0 {
            match line.normalize() {
                Some(x) => normalized_lines.push(x),
                None => (),
            }
        }
        MessageDefinition(normalized_lines)        
    }
}

impl Line {
    fn normalize(self) -> Option<Self> {
        // 1. Remove comments.
        match self {
            Line::Empty => None,
            Line::Comment(_) => None,
            Line::Definition(def, _) => match def.normalize() {
                Some(norm_def) => Some(Line::Definition(norm_def, None)),
                None => None,},
            Line::Directive(dir, _) => Some(Line::Directive(dir, None)),
        }
    }
}

impl AttributeDefinition {
    fn normalize(self) -> Option<Self> {
        match self {
            AttributeDefinition::Field(def) => match def.normalize() {
                Some(norm_field) => Some(AttributeDefinition::Field(norm_field)),
                None => None,},
            // 2. Remove all constant definitions
            AttributeDefinition::Const(_) => None,
        }
    }
}

impl FieldDefinition {
    fn normalize(self) -> Option<Self> {
        // 3. Ensure that all cast specifiers are explicitly defined; if not, add default cast specifiers.
        let cast_mode = match self.cast_mode {
            None => Some(CastMode::Saturated),
            x => x,
        };

        // 4. For dynamic arrays, replace the max length specifier in the form [<X] to the form [<=Y]
        let array = match self.array {
            ArrayInfo::DynamicLess(Size(num)) => ArrayInfo::DynamicLeq(Size(num-1)),
            x => x,
        };

        Some(FieldDefinition{cast_mode: cast_mode, field_type: self.field_type, array: array, name: self.name})
    }
}
