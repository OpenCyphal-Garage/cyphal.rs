extern crate dsdl_parser;
extern crate test_logger;

use std::io::BufReader;
use std::io::Read;

use dsdl_parser::*;

#[test]
fn parse_protocol() {
    test_logger::ensure_env_logger_initialized();
    let _dsdl = DSDL::read("./tests/dsdl/uavcan/protocol").unwrap();
}

#[test]
fn parse_dsdl() {
    test_logger::ensure_env_logger_initialized();
    let _dsdl = DSDL::read("./tests/dsdl/uavcan/").unwrap();
}

#[test]
fn verify_display() {
    test_logger::ensure_env_logger_initialized();
    let dsdl = DSDL::read("./tests/dsdl/uavcan/").unwrap();
    for dsdl_file in dsdl.files.values() {
        let mut filename = String::from("./tests/dsdl/");
        if dsdl_file.name.namespace != "" {
            filename = filename + dsdl_file.name.namespace.replace(".", "/").as_str() + "/";
        }
        if let Some(ref id) = dsdl_file.name.id {
            filename = filename + id.as_str() + ".";
        }
        if let Some(ref version) = dsdl_file.name.version {
            filename = filename + format!(".{}", version).as_str();
        }
        filename.push_str(dsdl_file.name.name.as_str());
        filename.push_str(".uavcan");
        
            
        let file = std::fs::File::open(filename.clone()).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        assert_eq!(format!("{}", dsdl_file.definition).split_whitespace().collect::<Vec<_>>(), contents.split_whitespace().collect::<Vec<_>>(), "Parsed file not equivalent to read file\n\nParsed file: \n{}\n\nRead file: \n{}", dsdl_file, contents);
        
        println!("Verified correct parsing on file: {}", filename);
    }
}

#[test]
fn normalize_get_node_info() {
    let dsdl = DSDL::read("./tests/dsdl/uavcan/").unwrap();
    
    assert_eq!(format!("{}", dsdl.files.get("uavcan.protocol.GetNodeInfo").unwrap().clone().normalize()),
               "uavcan.protocol.GetNodeInfo
---
saturated NodeStatus status
saturated SoftwareVersion software_version
saturated HardwareVersion hardware_version
saturated uint8[<=80] name");
}

#[test]
fn normalize_node_status() {
    let dsdl = DSDL::read("./tests/dsdl/uavcan/").unwrap();
    
    assert_eq!(format!("{}", dsdl.files.get("uavcan.protocol.NodeStatus").unwrap().clone().normalize()),
               "uavcan.protocol.NodeStatus
saturated uint32 uptime_sec
saturated uint2 health
saturated uint3 mode
saturated uint3 sub_mode
saturated uint16 vendor_specific_status_code");
}
