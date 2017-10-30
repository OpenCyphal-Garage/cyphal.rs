extern crate getopts;
extern crate dsdl_parser;
extern crate dsdl_compiler;
#[macro_use]
extern crate quote;

use std::fs::File;
use std::io::Write;

mod opts;
use opts::InputFlags;
use opts::print_usage;

use dsdl_parser::DSDL;

use dsdl_compiler::Compile;

fn main() {
    let flags = InputFlags::read();
    
    if flags.help {
        print_usage();
        return;
    }

    let input = if let Some(path) = flags.input.clone() {
        path
    } else {
        print_usage();
        println!("\nInput needs to be specified");
        return;
    };

    let output = if let Some(path) = flags.output.clone() {
        path
    } else {
        print_usage();
        println!("\nOutput needs to be specified");
        return;
    };

    let dsdl = DSDL::read(input).unwrap();
    let items = dsdl.compile();

    let mut file = File::create(output).unwrap();

    let tokens = quote!{#(#items)*};
    
    file.write_all(tokens.as_str().as_bytes());    
    
}
