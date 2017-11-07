# DSDL parser

> A parser for the DSDL (Data structure description language) used in [uavcan](http://uavcan.org)

## DSDL
DSDL defines the data types transfered with uavcan. For full description of DSDL, have a look at the [specification](http://uavcan.org/Specification/3._Data_structure_description_language/)

## Examples
### Parse DSDL directory

```rust
use dsdl_parser::DSDL;

assert!(DSDL::read("tests/dsdl/uavcan").is_ok());

```

### Parse single file

```rust
use dsdl_parser::DSDL;

assert!(DSDL::read("tests/dsdl/uavcan/protocol/341.NodeStatus.uavcan").is_ok());

```

### Display a file

```rust
use dsdl_parser::DSDL;

let dsdl = DSDL::read("./tests/dsdl/uavcan/").unwrap();

println!("{}", dsdl.get_file("uavcan.protocol.GetNodeInfo").unwrap());

```

### Calculate data type signature

```rust
use dsdl_parser::DSDL;

let dsdl = DSDL::read("./tests/dsdl/uavcan/").unwrap();

assert_eq!(dsdl.data_type_signature("uavcan.protocol.GetNodeInfo").unwrap(), 0xee468a8121c46a9e);
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

