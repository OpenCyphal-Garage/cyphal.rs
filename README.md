# uavcan.rs [![Build Status](https://travis-ci.org/UAVCAN/uavcan.rs.svg?branch=master)](https://travis-ci.org/UAVCAN/uavcan.rs) [![Crates.io](https://img.shields.io/crates/UAVCAN/uavcan-core.svg)](https://crates.io/crates/uavcan-core)

## This is an experiment, don't read anything into it.

Implementation of UAVCAN protocol in rust. Once I have a decent no-std implementation I'll make a PR [upstream](https://github.com/UAVCAN/uavcan.rs)

My general plan is this:
- Create a UAVCAN/CAN-only library, requiring std.
- Re-work CAN-only implementation to provide three options.
  - no-std, pure static (no allocator required)
  - no-std, with user-provided allocation scheme
  - std based, user can provide a global allocator, or just run defaults.
- Pull CAN-specific components out of the core crate and provide the CAN transport as a seperate crate
  - At this point I would probably PR against upstream and maybe start thinking about being the
    official maintainer.
  - Transports would simply be a trait at this point.
- Add rust support to [nunavut](https://github.com/UAVCAN/uavcan.rs)
  - I'm inclined to place the serialization in macros (similar to the existing
    uavcan-derive), so that all serialization and testing can be done directly
    in Rust and contained with in a single crate. This would mean nunavut would
    simply generate a crate of structs, all of them with some form of
    `#[derive(uavcan)]`. I'm not entirely opposed to leaving things in nunavut
    but Rust does provide the tools we need, so keeping things consistent would be nice.

## Design issues

- Do old transfer sessions get immediately overwritten when a new
  transfer ID comes in?

## Port to embedded

### steps

1. make lib *no_std* as default
1. added feature _std_ for std support
1. use crate [embedded_time](https://github.com/FluenTech/embedded-time/) for timestamping
1. added Clocks:
   1. StdClock for std environments
   1. TestClock for tests
2. write another session for no std use (with crate [heapless](https://github.com/japaric/heapless))

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
