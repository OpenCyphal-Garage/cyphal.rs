#![no_std]

#[macro_use]
extern crate uavcan_indexable_derive;

mod uavcan_frame;
mod can_frame;
mod types;
mod crc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
