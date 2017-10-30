extern crate getopts;

mod opts;
use opts::InputFlags;
use opts::print_usage;

fn main() {
    let flags = InputFlags::read();
    if flags.help {
        print_usage();
        return;
    }

        
    println!("Hello world! {:?}", flags);
}
