//! UAVCAN/CAN transport implementation.

use arrayvec::ArrayVec;
use num_traits::{ToPrimitive, FromPrimitive};

use crate::{TxError, types::*};
use crate::Priority;


use super::Transport;
use crate::session::SessionManager;
use crate::RxError;
use crate::internal::InternalRxFrame;
use crate::TransferKind;
use crate::NodeId;


mod bitfields;
mod session;

#[cfg(test)]
mod tests;

pub use bitfields::*;

pub const MTU_SIZE: usize = 8;

/// Unit struct for declaring transport type
#[derive(Copy, Clone, Debug)]
pub struct Can;

// I don't like that I have to do this.
// *Not* doing this would rely on GAT's 
impl<S: SessionManager> crate::Node<S, Can> {
    fn transmit<'a>(transfer: &'a crate::transfer::Transfer) -> Result<CanIter<'a>, TxError> {
        CanIter::new(
            transfer,
            Some(1),
        )
    }
}

impl Transport for Can {
    type Frame = CanFrame;


    fn rx_process_frame<'a>(node_id: &Option<NodeId>, frame: &'a Self::Frame) -> Result<Option<InternalRxFrame<'a>>, RxError> {
        // Frames cannot be empty. They must at least have a tail byte.
        // NOTE: libcanard specifies this as only for multi-frame transfers but uses
        // this logic.
        if frame.payload.len() == 0 {
            return Err(RxError::FrameEmpty);
        }

        // Pull tail byte from payload
        let tail_byte = TailByte(*frame.payload.last().unwrap());

        // Protocol version states SOT must have toggle set
        if tail_byte.start_of_transfer() && !tail_byte.toggle() {
            return Err(RxError::TransferStartMissingToggle);
        }
        // Non-last frames must use the MTU fully
        if !tail_byte.end_of_transfer() && frame.payload.len() < MTU_SIZE {
            return Err(RxError::NonLastUnderUtilization);
        }

        if CanServiceId(frame.id).is_svc() {
            // Handle services
            let id = CanServiceId(frame.id);

            // Ignore frames not meant for us
            if node_id.is_none() || id.destination_id() != node_id.unwrap() {
                return Ok(None);
            }

            let transfer_kind = if id.is_req() {
                TransferKind::Request
            } else {
                TransferKind::Response
            };

            return Ok(Some(InternalRxFrame::as_service(
                frame.timestamp,
                Priority::from_u8(id.priority()).unwrap(),
                transfer_kind,
                id.service_id(),
                id.source_id(),
                id.destination_id(),
                tail_byte.transfer_id(),
                tail_byte.start_of_transfer(),
                tail_byte.end_of_transfer(),
                &frame.payload,
            )));
        } else {
            // Handle messages
            let id = CanMessageId(frame.id);

            // We can ignore ID in anonymous transfers
            let source_node_id = if id.is_anon() {
                // Anonymous transfers can only be single-frame transfers
                if !(tail_byte.start_of_transfer() && tail_byte.end_of_transfer()) {
                    return Err(RxError::AnonNotSingleFrame);
                }

                None
            } else {
                Some(id.source_id())
            };

            if !id.valid() {
                return Err(RxError::InvalidCanId);
            }

            return Ok(Some(InternalRxFrame::as_message(
                frame.timestamp,
                Priority::from_u8(id.priority()).unwrap(),
                id.subject_id(),
                source_node_id,
                tail_byte.transfer_id(),
                tail_byte.start_of_transfer(),
                tail_byte.end_of_transfer(),
                &frame.payload,
            )));
        }
    }

    //fn transmit<'a>(transfer: &'t crate::transfer::Transfer) -> CanIter<'a> {
    //    CanIter::new(transfer, Some(1))
    //}
}

#[derive(Debug)]
struct CanIter<'a> {
    transfer: &'a crate::transfer::Transfer,
    frame_id: u32,
    payload_offset: usize,
    crc: crc_any::CRCu16,
    crc_left: u8,
    toggle: bool,
    is_start: bool,
}

impl<'a> CanIter<'a> {
    fn new(transfer: &'a crate::transfer::Transfer, node_id: Option<NodeId>) -> Result<Self, TxError> {
        // TODO return errors here, e.g. if anon but sending service message
        // Also another error is if you're anon but sending multi-frame transfers
        let frame_id = match transfer.transfer_kind {
            TransferKind::Message => {
                if node_id.is_none() && transfer.payload.len() > 7 {
                    return Err(TxError::AnonNotSingleFrame);
                }

                CanMessageId::new(
                    transfer.priority,
                    transfer.port_id,
                    node_id
                ).to_u32().unwrap()
            }
            TransferKind::Request => {
                // These runtime checks should be removed via proper typing further up but we'll
                // leave it as is for now.
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let destination = transfer.remote_node_id.ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    transfer.priority,
                    true,
                    transfer.port_id,
                    transfer.remote_node_id.unwrap(),
                    source
                ).to_u32().unwrap()
            }
            TransferKind::Response => {
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let destination = transfer.remote_node_id.ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    transfer.priority,
                    false,
                    transfer.port_id,
                    destination,
                    source
                ).to_u32().unwrap()
            }
        };

        Ok(Self {
            transfer,
            frame_id,
            payload_offset: 0,
            crc: crc_any::CRCu16::crc16ccitt_false(),
            crc_left: 2,
            toggle: true,
            is_start: true,
        })
    }
}

impl<'a> Iterator for CanIter<'a> {
    type Item = CanFrame;

    // I'm sure I could take an optimization pass at the logic here
    fn next(&mut self) -> Option<Self::Item> {
        let mut frame = CanFrame {
            timestamp: std::time::Instant::now(),
            id: self.frame_id,
            payload: ArrayVec::new(),
        };

        let bytes_left = self.transfer.payload.len() - self.payload_offset;
        let is_end = bytes_left <= 7;
        let copy_len = core::cmp::min(bytes_left, 7);

        if self.is_start && is_end {
            // Single frame transfer, no CRC
            frame.payload.extend(self.transfer.payload[0..copy_len].iter().copied());
            self.payload_offset += bytes_left;
            unsafe {
                frame.payload.push_unchecked(TailByte::new(
                    true,
                    true,
                    true,
                    self.transfer.transfer_id
                ).to_u8().unwrap())
            }
        } else {
            // Nothing left to transmit, we are done
            if bytes_left == 0 && self.crc_left == 0 {
                return None;
            }

            // Handle CRC
            let out_data = &self.transfer.payload[self.payload_offset..self.payload_offset + copy_len];
            self.crc.digest(out_data);
            frame.payload.extend(out_data.iter().copied());

            // Finished with our data, now we deal with crc
            // (we can't do anything if bytes_left == 7, so ignore that case)
            if bytes_left < 7 {
                let crc = &self.crc.get_crc().to_be_bytes();
                if self.crc_left == 2 {
                    if 7 - bytes_left >= 2 {
                        // Iter doesn't work. Internal type is &u8 but extend
                        // expects u8
                        frame.payload.push(crc[0]);
                        frame.payload.push(crc[1]);
                        self.crc_left = 0;
                    } else {
                        unsafe {
                            frame.payload.push_unchecked(crc[0]);
                        }
                    }
                    match frame.payload.try_extend_from_slice(crc) {
                        Ok(()) => self.crc_left = 0,
                        Err(_) => self.crc_left -= 1,
                    }
                } else if self.crc_left == 1 {
                    unsafe {
                        frame.payload.push_unchecked(crc[1]);
                    }
                }
            }

            // Advance state of iter
            self.toggle = !self.toggle;
        }

        self.is_start = false;

        Some(frame)
    }

    // TODO impl size_hint for frames remaining
}

// TODO convert to embedded-hal PR type
/// Extended CAN frame (the only one supported by UAVCAN/CAN)
#[derive(Clone, Debug)]
pub struct CanFrame {
    pub timestamp: Timestamp,
    pub id: u32,
    pub payload: ArrayVec<[u8; 8]>,
}

/// Keeps track of toggle bit and CRC during frame processing.
#[derive(Debug)]
pub struct CanMetadata {
    toggle: bool,
    crc: crc_any::CRCu16,
}

impl super::SessionMetadata for CanMetadata {
    fn new() -> Self {
        Self {
            // Toggle starts off true, but we compare against the opposite value.
            toggle: false,
            crc: crc_any::CRCu16::crc16ccitt_false(),
        }
    }

    fn update(&mut self, frame: &crate::internal::InternalRxFrame) -> Option<usize> {
        // Single frame transfers don't need to be validated
        if frame.start_of_transfer && frame.end_of_transfer {
            // TODO should I still check if toggle starts at 1?

            // Still need to truncate tail byte
            return Some(frame.payload.len() - 1);
        }

        // CRC all but the tail byte
        self.crc.digest(&frame.payload[0..frame.payload.len() - 1]);
        self.toggle = !self.toggle;

        let toggle = TailByte(frame.payload[frame.payload.len() - 1]).toggle();

        if toggle == self.toggle {
            // Truncate tail byte
            Some(frame.payload.len() - 1)
        } else {
            None
        }
    }
    
    fn is_valid(&self, frame: &crate::internal::InternalRxFrame) -> bool {
        if frame.start_of_transfer && frame.end_of_transfer {
            return true;
        }

        if self.crc.get_crc() == 0x0000u16 {
            return true;
        }

        return false;
    }
}