use std::env;
use getopts::Options;

#[derive(Debug)]
pub(crate) struct InputFlags {
    pub input: Option<String>,
    pub output: Option<String>,
    pub data_type_signature: bool,
    pub help: bool,
    pub version: bool,
}

fn options() -> Options {
    let mut opts = Options::new();
    opts.optopt("o", "output", "set output file name", "NAME");
    opts.optopt("i", "input", "set input dir/file name", "NAME");
    
    opts.optflag("", "data-type-signature", "inserts data type signatures");
    
    opts.optflag("", "version", "print the version of this software");
    opts.optflag("h", "help", "print this help menu");
    opts
}

pub(crate) fn print_usage() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let opts = options();
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

pub(crate) fn print_version() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    println!("{} {}", program, env!("CARGO_PKG_VERSION"));
}

impl InputFlags {
    pub fn read() -> Self {
        let args: Vec<String> = env::args().collect();
        
        let opts = options();
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(f) => { panic!(f.to_string()) }
        };

        InputFlags{
            input: matches.opt_str("i"),
            output: matches.opt_str("o"),
            data_type_signature: matches.opt_present("data-type-signature"),
            help: matches.opt_present("h"),
            version: matches.opt_present("version"),
        }            
    }
}
