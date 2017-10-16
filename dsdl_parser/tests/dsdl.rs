extern crate dsdl_parser;
extern crate test_logger;

use dsdl_parser::*;

#[test]
fn parse_protocol() {
    test_logger::ensure_env_logger_initialized();
    let _dsdl = DSDL::open("./tests/dsdl/uavcan/protocol").unwrap();
}

#[test]
fn parse_dsdl() {
    test_logger::ensure_env_logger_initialized();
    let _dsdl = DSDL::open("./tests/dsdl/uavcan/").unwrap();
}
