use std::env;
use getopts::Options;

#[derive(Debug)]
pub(crate) struct InputFlags {
    pub input: Option<String>,
    pub output: Option<String>,
    pub help: bool,
}

fn options() -> Options {
    let mut opts = Options::new();
    opts.optopt("o", "output", "set output file name", "NAME");
    opts.optopt("i", "input", "set input dir/file name", "NAME");
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

impl InputFlags {
    pub fn read() -> Self {
        let args: Vec<String> = env::args().collect();
        
        let opts = options();
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(f) => { panic!(f.to_string()) }
        };
        let help = matches.opt_present("h");
        let output = matches.opt_str("o");
        let input = matches.opt_str("i");

        InputFlags{
            input: input,
            output: output,
            help: help,
        }            
    }
}
