# DSDL compiler

> A compiler for the DSDL (Data structure description language) used in [uavcan](http://uavcan.org)

## DSDL
DSDL defines the data types transfered with uavcan. For full description of DSDL, have a look at the [specification](http://uavcan.org/Specification/3._Data_structure_description_language/)

## Binary

### Installation
dsdlc` can be installed by running `cargo install dsdl_compiler`

###  Usage
To find documentation on usage. run `dsdlc -h` after installation

## Library

### Examples
#### Compile DSDL directory

```
use dsdl_compiler::DSDL;
use dsdl_compiler::Compile;

let dsdl = DSDL::read("tests/dsdl/").unwrap();
let items = dsdl.compile();

assert!(items.len() >= 1);

```

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

