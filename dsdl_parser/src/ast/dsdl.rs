//! Everything related to the full dsdl definition `DSDL`

use std;

use std::io::Read;

use std::fs;

use std::path::Path;

use std::str;
use std::str::FromStr;

use std::collections::HashMap;


use parser::parser;
use parser::lexer;

use crc::CRC64WE as CRC;

use ast::ty::CompositeType;
use ast::ty::Ty;
use ast::attribute_definition::AttributeDefinition;
use ast::attribute_definition::FieldDefinition;
use ast::type_definition::TypeDefinition;
use ast::type_definition::MessageDefinition;
use ast::type_definition::ServiceDefinition;
use ast::file::File;
use ast::line::Line;
use ast::file_name::FileName;

use error::ParseError;

/// The `DSDL` struct contains a number of data type definition
#[derive(Debug, PartialEq, Eq)]
pub struct DSDL {
    files: HashMap<String, File>,
}

impl DSDL {
    /// Reads `DSDL` definition recursively if path is a directory. Reads one `DSDL` definition if path is a definition.
    ///
    /// ## Example
    /// ```
    /// use dsdl_parser::DSDL;
    ///
    /// assert!(DSDL::read("tests/dsdl/").is_ok());
    ///
    /// ```
    pub fn read<P: AsRef<Path>>(path: P) -> std::io::Result<DSDL> {
        let mut dsdl = DSDL{files: HashMap::new()};
        let mut errors = Vec::new();

        if path.as_ref().is_dir() {
            for entry in fs::read_dir(path)? {
                let current_path = entry?.path();
                DSDL::read_uavcan_files(current_path.as_ref(), String::new(), &mut errors, &mut dsdl.files)?;
            }
        } else {
            DSDL::read_uavcan_files(path.as_ref(), String::new(), &mut errors, &mut dsdl.files)?;
        }

        Ok(dsdl)
    }

    fn read_uavcan_files(path: &Path, namespace: String, errors: &mut Vec<ParseError>, files: &mut HashMap<String, File>) -> std::io::Result<()> {
        let uavcan_path = if namespace.as_str() == "" {
            String::from(path.file_name().unwrap().to_str().unwrap())
        } else {
            namespace.clone() + "." + path.file_name().unwrap().to_str().unwrap()
        };
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let current_path = entry?.path();
                DSDL::read_uavcan_files(&current_path, uavcan_path.clone(), errors, files)?;
            }
        } else if let Ok(file_name) = FileName::from_str(&uavcan_path) {
            let mut file = fs::File::open(path)?;
            let mut file_content = String::new();
            file.read_to_string(&mut file_content)?;

            println!("FileName: {}", uavcan_path);

            match parser::TypeDefinitionParser::new().parse(errors, lexer::Lexer::new(&file_content)) {
                Ok(definition) => {
                    let qualified_name = if file_name.namespace.as_str() == "" {
                        file_name.name.clone()
                    } else {
                        file_name.namespace.clone() + "." + file_name.name.as_str()
                    };
                    files.insert(qualified_name, File { name: file_name, definition: definition });
                },
                Err(e) => unimplemented!("TODO: Insert error handling (Parsing failed at file: {}, with error: {:?})", uavcan_path, e),
            };
        } else {
            warn!("The file, {}, was not recognized as a DSDL file. DSDL files need to have the .uavcan extension", uavcan_path);
        }

        Ok(())
    }

    /// Return a file if there exists one, returns `None` otherwise
    ///
    /// ## Example
    /// ```
    /// use dsdl_parser::DSDL;
    ///
    /// let dsdl = DSDL::read("tests/dsdl/").unwrap();
    ///
    /// assert!(dsdl.get_file("uavcan.protocol.NodeStatus").is_some());
    ///
    /// ```
    pub fn get_file<T: AsRef<str>>(&self, name: T) -> Option<&File> {
        self.files.get(name.as_ref())
    }

    /// Returns a vector containing references to all files
    ///
    /// ## Example
    /// ```
    /// use dsdl_parser::DSDL;
    ///
    /// let dsdl = DSDL::read("tests/dsdl/").unwrap();
    ///
    /// assert!(dsdl.files().len() >= 1);
    ///
    /// ```
    pub fn files(&self) -> Vec<&File> {
        self.files.values().collect()
    }

    /// Returns the data type signature of a data type
    ///
    /// ## Example
    /// ```
    /// use dsdl_parser::DSDL;
    ///
    /// let dsdl = DSDL::read("tests/dsdl/").unwrap();
    ///
    /// assert_eq!(dsdl.data_type_signature("uavcan.protocol.GetNodeInfo").unwrap(), 0xee468a8121c46a9e);
    ///
    /// ```
    pub fn data_type_signature<T: AsRef<str>>(&self, name: T) -> Option<u64> {
        let normalized_file = match self.get_file(name) {
            Some(file) => file.clone().normalize(),
            None => return None,
        };
        let current_namespace = normalized_file.as_file().clone().name.namespace;
        let mut crc = CRC::from_value(normalized_file.dsdl_signature());

        let lines = match normalized_file.as_file().definition {
            TypeDefinition::Message(MessageDefinition(ref lines)) => lines.clone(),
            TypeDefinition::Service(ServiceDefinition{request: MessageDefinition(ref request), response: MessageDefinition(ref response)}) => {let mut lines = request.clone(); lines.append(&mut response.clone()); lines},
        };

        for line in lines {
            match line {
                Line::Definition{
                    definition: AttributeDefinition::Field(
                        FieldDefinition {
                            field_type: Ty::Composite(CompositeType{namespace: None, ref name}),
                            ..
                        },
                    ),
                    ..
                } => crc.extend(self.data_type_signature((String::from(current_namespace.clone()) + "." + name.as_ref()).as_str()).unwrap()),
                Line::Definition{
                    definition: AttributeDefinition::Field(
                        FieldDefinition {
                            field_type: Ty::Composite(CompositeType { namespace: Some(ref namespace), ref name }),
                            ..
                        }
                    ),
                    ..
                } => crc.extend(self.data_type_signature(String::from(namespace.clone()) + "." + name.as_ref()).unwrap()),
                _ => (),
            }

        }
        Some(crc.value())

    }

}