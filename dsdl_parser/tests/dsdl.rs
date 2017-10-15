extern crate dsdl_parser;

use dsdl_parser::*;

#[test]
fn parse_protocol() {
    let _dsdl = DSDL::open("./tests/dsdl/uavcan/protocol").unwrap();
}

#[test]
fn parse_dsdl() {
    let _dsdl = DSDL::open("./tests/dsdl/uavcan/").unwrap();
}
