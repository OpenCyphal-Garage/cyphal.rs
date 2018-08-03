//! Everything related to the uavcan filename `FileName`

use std::str::FromStr;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// Uniquely defines a DSDL file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileName {
    pub id: Option<u64>,
    pub namespace: String,
    pub name: String,
    pub version: Option<Version>,
}

impl FileName {
    /// Split a namespace into parts
    ///
    /// # Examples
    /// ```
    /// use dsdl_parser::FileName;
    ///
    /// let name = FileName {
    ///                     id: Some(341),
    ///                     namespace: String::from("uavcan.protocol"),
    ///                     name: String::from("NodeStatus"),
    ///                     version: None,
    /// };
    ///
    /// assert_eq!(name.split_namespace(), vec!["uavcan", "protocol"]);
    ///
    /// ```
    pub fn split_namespace(&self) -> Vec<String> {
        if self.namespace == String::new() {
            Vec::new()
        } else {
            self.namespace.split('.').map(|x| String::from(x)).collect()
        }
    }

    /// Split a namespace into parts
    ///
    /// # Examples
    /// ```
    /// use dsdl_parser::FileName;
    ///
    /// let name = FileName {
    ///                     id: Some(341),
    ///                     namespace: String::from("uavcan.protocol"),
    ///                     name: String::from("NodeStatus"),
    ///                     version: None,
    /// };
    ///
    /// assert_eq!(name.rsplit_namespace(), vec!["protocol", "uavcan"]);
    ///
    /// ```
    pub fn rsplit_namespace(&self) -> Vec<String> {
        if self.namespace == String::new() {
            Vec::new()
        } else {
            self.namespace.rsplit('.').map(|x| String::from(x)).collect()
        }
    }

}

/// A dsdl file version
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseFileNameError {
    MissingExtension,
    Format,
    VersionFormat,
}

impl FromStr for FileName {
    type Err = ParseFileNameError;

    fn from_str(s: &str) -> Result<FileName, Self::Err> {
        let mut split = s.rsplit('.').peekable();

        if let Some("uavcan") = split.next() {
        } else {
            return Err(ParseFileNameError::MissingExtension);
        }

        let version = match u32::from_str(split.peek().ok_or(ParseFileNameError::Format)?) {
            Ok(minor_version) => {
                split.next().unwrap(); // remove minor version
                let major_version = u32::from_str(split.next().ok_or(ParseFileNameError::Format)?).map_err(|_| ParseFileNameError::VersionFormat)?;
                Some(Version{major: major_version, minor: minor_version})
            },
            Err(_) => None,
        };

        let name = String::from(split.next().unwrap());

        let id = if let Ok(id) = u64::from_str(split.peek().unwrap_or(&"")) {
            split.next().unwrap();
            Some(id)
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

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.namespace.as_str() != "" {
            write!(f, "{}.", self.namespace)?;
        }
        write!(f, "{}", self.name)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn parse_file_name() {
        assert_eq!(FileName::from_str("NodeStatus.uavcan"), Ok(FileName { id: None, namespace: String::from(""), name: String::from("NodeStatus"), version: None }));
        assert_eq!(FileName::from_str("protocol.NodeStatus.uavcan"), Ok(FileName { id: None, namespace: String::from("protocol"), name: String::from("NodeStatus"), version: None }));
        assert_eq!(FileName::from_str("uavcan.protocol.NodeStatus.uavcan"), Ok(FileName { id: None, namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: None }));
        assert_eq!(FileName::from_str("341.NodeStatus.uavcan"), Ok(FileName { id: Some(341), namespace: String::from(""), name: String::from("NodeStatus"), version: None }));
        assert_eq!(FileName::from_str("uavcan.protocol.341.NodeStatus.uavcan"), Ok(FileName { id: Some(341), namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: None }));
        assert_eq!(FileName::from_str("NodeStatus.0.1.uavcan"), Ok(FileName { id: None, namespace: String::from(""), name: String::from("NodeStatus"), version: Some(Version{major: 0, minor: 1}) }));
        assert_eq!(FileName::from_str("uavcan.protocol.NodeStatus.0.1.uavcan"), Ok(FileName { id: None, namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: Some(Version{major: 0, minor: 1}) }));
        assert_eq!(FileName::from_str("uavcan.protocol.341.NodeStatus.0.1.uavcan"), Ok(FileName { id: Some(341), namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: Some(Version{major: 0, minor: 1}) }));
    }
}