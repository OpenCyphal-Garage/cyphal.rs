extern crate dsdl_compiler;

use dsdl_compiler::DSDL;

#[test]
fn read_dsdl() {
    let _dsdl = DSDL::open("./tests/dsdl/").unwrap();
}

