extern crate dsdl_compiler;
#[macro_use]
extern crate quote;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use dsdl_compiler::DSDL;
use dsdl_compiler::Compile;
use dsdl_compiler::CompileConfig;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dsdl_path = Path::new(&cargo_dir).join("dsdl");
    let out_path = Path::new(&out_dir).join("dsdl.rs");

    let dsdl = DSDL::read(dsdl_path.clone()).unwrap();

    let mut compile_config = CompileConfig::default();
    compile_config.data_type_signature = true;
    
    let items = dsdl.compile(&compile_config);

    let mut file = File::create(&out_path).unwrap();

    let tokens = quote!{#(#items)*};
    
    file.write_all(tokens.as_str().as_bytes()).unwrap();

    // only recompile DSDL if it has changed
    println!("cargo:rerun-if-changed={}", dsdl_path.to_str().unwrap());
}
