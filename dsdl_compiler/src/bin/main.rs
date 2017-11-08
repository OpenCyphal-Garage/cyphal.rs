extern crate getopts;
#[macro_use]
extern crate log;
extern crate badlog;
extern crate dsdl_parser;
extern crate dsdl_compiler;
#[macro_use]
extern crate quote;

use std::fs::File;
use std::io::Write;

mod opts;
use opts::InputFlags;

use dsdl_parser::DSDL;

use dsdl_compiler::Compile;
use dsdl_compiler::CompileConfig;

fn main() {
    badlog::init(Some("info"));
    
    let flags = InputFlags::read();
    
    if flags.help {
        opts::print_usage();
        return;
    }

    if flags.version {
        opts::print_version();
        return;
    }

    let input = if let Some(path) = flags.input.clone() {
        path
    } else {
        opts::print_usage();
        println!("\nInput needs to be specified");
        return;
    };

    let output = if let Some(path) = flags.output.clone() {
        path
    } else {
        opts::print_usage();
        println!("\nOutput needs to be specified");
        return;
    };

    let dsdl = match DSDL::read(input) {
        Ok(dsdl) => dsdl,
        Err(error) => {
            error!("errored when reading DSDL: {}", error);
            return;
        },
    };
    
    let items = dsdl.compile(&CompileConfig::default());
    
    let mut file = match File::create(output) {
        Ok(file) => file,
        Err(error) => {
            error!("errored when creating output file: {}", error);
            return;
        },
    };


    let tokens = quote!{#(#items)*};
    
    match file.write_all(tokens.as_str().as_bytes()) {
        Ok(_) => (),
        Err(error) => error!("errored when writing to output file: {}", error),
    }
    
}
