extern crate dsdl_parser;

use dsdl_parser::DSDL;

#[test]
fn read_dsdl() {
    let _dsdl = DSDL::open("./tests/dsdl/").unwrap();
}

