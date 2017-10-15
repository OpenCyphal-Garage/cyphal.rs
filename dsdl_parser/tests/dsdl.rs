extern crate dsdl_parser;

use dsdl_parser::*;

#[test]
fn read_dsdl() {
    let _dsdl = DSDL::open("./tests/dsdl/uavcan/").unwrap();
}
