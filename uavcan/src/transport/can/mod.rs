//! UAVCAN/CAN transport implementation.
//!
//! CAN will essentially be the "reference implementation", and *should* always follow
//! the best practices, so if you want to add support for a new transport, you should
//! follow the conventions here.
//!
//! Provides a unit struct to create a Node for CAN. This implements the common
//! transmit function that *must* be implemented by any transport. This can't be a
//! trait in stable unfortunately because it would require GATs, which won't be stable
//! for quite a while... :(.

use crate::crc16::Crc16;

mod bitfields;
mod fd;
mod legacy;

#[cfg(test)]
mod tests;

pub use bitfields::{CanMessageId, CanServiceId};

use bitfields::TailByte;

/// Keeps track of toggle bit and CRC during frame processing.
#[derive(Debug)]
pub struct CanMetadata {
    toggle: bool,
    crc: Crc16,
}

impl<C: embedded_time::Clock> crate::transport::SessionMetadata<C> for CanMetadata {
    fn new() -> Self {
        Self {
            // Toggle starts off true, but we compare against the opposite value.
            toggle: false,
            crc: Crc16::init(),
        }
    }

    fn update(&mut self, frame: &crate::internal::InternalRxFrame<C>) -> Option<usize> {
        // Single frame transfers don't need to be validated
        if frame.start_of_transfer && frame.end_of_transfer {
            // Still need to truncate tail byte
            return Some(frame.payload.len() - 1);
        }

        // CRC all but the tail byte
        self.crc.digest(&frame.payload[0..frame.payload.len() - 1]);
        self.toggle = !self.toggle;

        let tail = TailByte(frame.payload[frame.payload.len() - 1]);

        if tail.toggle() == self.toggle {
            if tail.end_of_transfer() {
                // Exclude CRC from data
                Some(frame.payload.len() - 3)
            } else {
                // Just truncate tail byte
                Some(frame.payload.len() - 1)
            }
        } else {
            None
        }
    }

    fn is_valid(&self, frame: &crate::internal::InternalRxFrame<C>) -> bool {
        if frame.start_of_transfer && frame.end_of_transfer {
            return true;
        }

        if self.crc.get_crc() == 0x0000u16 {
            return true;
        }

        false
    }
}
