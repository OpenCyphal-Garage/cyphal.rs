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

use arrayvec::ArrayVec;
use embedded_hal::can::ExtendedId;
use embedded_time::Clock;
use num_traits::FromPrimitive;
use streaming_iterator::StreamingIterator;

use crate::time::Timestamp;
use crate::Priority;
use crate::TxError;

use super::Transport;
use crate::crc16::Crc16;
use crate::internal::InternalRxFrame;
use crate::NodeId;
use crate::RxError;
use crate::TransferKind;

mod legacy;
mod bitfields;

#[cfg(test)]
mod tests;

pub use legacy::*;
pub use bitfields::{CanMessageId, CanServiceId};

