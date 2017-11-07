# DSDL

> A convinient way to compile [DSDL](http://uavcan.org/Specification/3._Data_structure_description_language/)

## DSDL
DSDL defines the data types transfered with uavcan. For full description of DSDL, have a look at the [specification](http://uavcan.org/Specification/3._Data_structure_description_language/)

## Usage
This crate will look for DSDL definitions at `$(CARGO_MANIFEST_DIR)/dsdl` and make the compiled Rust definitions available inside this crate.

If you wish to use the standard DSDL definitions add them as a git submodule inside the crate, `git submodule add https://github.com/UAVCAN/dsdl.git`.

## Examples
The following examples assumes that the standard DSDL definition is located at `$(CARGO_MANIFEST_DIR)/dsdl`.
### Basic usage
```
extern crate dsdl;
extern crate uavcan;

use uavcan::Message;

# fn main() {
#
assert_eq!(dsdl::uavcan::protocol::NodeStatus::TYPE_ID, Some(341));
# }

```

## Alternatives
A stand alone dsdl compiler can be installed by running `cargo install dsdl_compiler`. Run `dsdlc -h` for usage documentation.


# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

