//! A crate for conviniently compiling [DSDL](http://uavcan.org/Specification/3._Data_structure_description_language/) to Rust structures.
//!
//! For full description of DSDL, have a look at the [specification](http://uavcan.org/Specification/3._Data_structure_description_language/)
//! 
//! # Usage
//! This crate will look for DSDL definitions at `$(CARGO_MANIFEST_DIR)/dsdl` and make the compiled Rust definitions available inside this crate.
//!
//! If you wish to use the standard DSDL definitions add them as a git submodule inside the crate, `git submodule add https://github.com/UAVCAN/dsdl.git`.
//! 
//! ## Examples
//! The following examples assumes that the standard DSDL definition is located at `$(CARGO_MANIFEST_DIR)/dsdl`.
//! ### Basic usage
//! ```
//! extern crate dsdl;
//! extern crate uavcan;
//!
//! use uavcan::Message;
//! 
//! # fn main() {
//! #
//! assert_eq!(dsdl::uavcan::protocol::NodeStatus::TYPE_ID, Some(341));
//! # }
//! 
//! ```
#![no_std]

include!(concat!(env!("OUT_DIR"), "/dsdl.rs"));
