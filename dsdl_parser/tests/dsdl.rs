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
    let _dsdl = DSDL::read("./tests/dsdl/").unwrap();
}

#[test]
fn verify_display() {
    test_logger::ensure_env_logger_initialized();
    let dsdl = DSDL::read("./tests/dsdl/").unwrap();
    for dsdl_file in dsdl.files() {
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
    let dsdl = DSDL::read("./tests/dsdl/").unwrap();
    
    assert_eq!(format!("{}", dsdl.get_file("uavcan.protocol.GetNodeInfo").unwrap().clone().normalize()),
               "uavcan.protocol.GetNodeInfo
---
uavcan.protocol.NodeStatus status
uavcan.protocol.SoftwareVersion software_version
uavcan.protocol.HardwareVersion hardware_version
saturated uint8[<=80] name");
}

#[test]
fn normalize_actuator_status() {
    let dsdl = DSDL::read("./tests/dsdl/").unwrap();
    
    assert_eq!(format!("{}", dsdl.get_file("uavcan.equipment.actuator.Status").unwrap().clone().normalize()),
               "uavcan.equipment.actuator.Status
saturated uint8 actuator_id
saturated float16 position
saturated float16 force
saturated float16 speed
void1
saturated uint7 power_rating_pct");
}

#[test]
fn normalize_node_status() {
    let dsdl = DSDL::read("./tests/dsdl/").unwrap();
    
    assert_eq!(format!("{}", dsdl.get_file("uavcan.protocol.NodeStatus").unwrap().clone().normalize()),
               "uavcan.protocol.NodeStatus
saturated uint32 uptime_sec
saturated uint2 health
saturated uint3 mode
saturated uint3 sub_mode
saturated uint16 vendor_specific_status_code");
}


#[test]
fn verify_dsdl_signature() {
    let dsdl = DSDL::read("./tests/dsdl/").unwrap();

    assert_eq!(dsdl.get_file("uavcan.protocol.NodeStatus").unwrap().clone().normalize().dsdl_signature(), 0x0f0868d0c1a7c6f1);
    assert_eq!(dsdl.get_file("uavcan.protocol.AccessCommandShell").unwrap().clone().normalize().dsdl_signature(), 0x59276b5921c9246e);
    assert_eq!(dsdl.get_file("uavcan.equipment.actuator.Command").unwrap().clone().normalize().dsdl_signature(), 0x8d9a6a920c1d616c);
    assert_eq!(dsdl.get_file("uavcan.equipment.actuator.Status").unwrap().clone().normalize().dsdl_signature(), 0x5e9bba44faf1ea04);

}


#[test]
fn verify_data_type_signature() {
    let dsdl = DSDL::read("./tests/dsdl/").unwrap();

    assert_eq!(dsdl.data_type_signature("uavcan.protocol.NodeStatus").unwrap(), 0x0f0868d0c1a7c6f1);
    assert_eq!(dsdl.data_type_signature("uavcan.protocol.CANIfaceStats").unwrap(), 0x13b106f0c44ca350);
    
    assert_eq!(dsdl.data_type_signature("uavcan.protocol.GetTransportStats").unwrap(), 0xbe6f76a7ec312b04);

    assert_eq!(dsdl.data_type_signature("uavcan.protocol.GetNodeInfo").unwrap(), 0xee468a8121c46a9e);

}
