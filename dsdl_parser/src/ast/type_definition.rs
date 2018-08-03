//! Everything related to uavcan type definitions.
//!
//! A type definition can either be a message definition or service definition.

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use ast::file_name::FileName;
use ast::line::Line;

/// A DSDL type definition.
///
/// Each DSDL definition specifies exactly one data structure that can be used for message broadcasting
/// or a pair of structures that can be used for service invocation data exchange.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDefinition {
    Message(MessageDefinition),
    Service(ServiceDefinition),
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

impl TypeDefinition {
    pub(crate) fn normalize(self, file_name: &FileName) -> Self {
        match self {
            TypeDefinition::Message(x) => TypeDefinition::Message(x.normalize(file_name)),
            TypeDefinition::Service(x) => TypeDefinition::Service(x.normalize(file_name)),
        }
    }
}

impl ServiceDefinition {
    pub(crate) fn normalize(self, file_name: &FileName) -> Self {
        ServiceDefinition{request: self.request.normalize(file_name), response: self.response.normalize(file_name)}
    }
}

impl MessageDefinition {
    pub(crate) fn normalize(self, file_name: &FileName) -> Self {
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

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            TypeDefinition::Message(ref x) => write!(f, "{}", x),
            TypeDefinition::Service(ref x) => write!(f, "{}", x),
        }
    }
}

impl Display for MessageDefinition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let request = format!("{}", self.request);
        let response = format!("{}", self.response);

        if request != "" {
            write!(f, "{}\n", request)?;
        }

        write!(f, "---")?;

        if response != "" {
            write!(f, "\n{}", response)?;
        }

        Ok(())
    }
}

