extern crate lalrpop;

use std::env;

fn main() -> Result<(), Box<std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;

    lalrpop::Configuration::new()
        .set_out_dir(out_dir)
        .process()
}