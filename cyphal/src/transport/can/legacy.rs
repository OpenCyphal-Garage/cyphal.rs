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

use super::bitfields::*;
use crate::crc16::Crc16;
use crate::internal::InternalRxFrame;
use crate::time::Timestamp;
use crate::transport::Transport;
use crate::StreamingIterator;
use crate::{NodeId, Priority, RxError, TransferKind, TxError};
use crate::transfer::{Transfer, TransferMetadata};

/// Unit struct for declaring transport type
#[derive(Copy, Clone, Debug)]
pub struct Can;

impl<C: embedded_time::Clock + 'static> Transport<C> for Can {
    type Frame = CanFrame<C>;
    type FrameIter<'a> = CanIter<'a, C>;

    const MTU_SIZE: usize = 8;

    fn rx_process_frame<'a>(
        node_id: &Option<NodeId>,
        frame: &'a Self::Frame,
    ) -> Result<Option<InternalRxFrame<'a, C>>, RxError> {
        // Frames cannot be empty. They must at least have a tail byte.
        // NOTE: libcanard specifies this as only for multi-frame transfers but uses
        // this logic.
        if frame.payload.is_empty() {
            return Err(RxError::FrameEmpty);
        }

        // Pull tail byte from payload
        let tail_byte = TailByte(*frame.payload.last().unwrap());

        // Protocol version states SOT must have toggle set
        if tail_byte.start_of_transfer() && !tail_byte.toggle() {
            return Err(RxError::TransferStartMissingToggle);
        }
        // Non-last frames must use the MTU fully
        if !tail_byte.end_of_transfer() && frame.payload.len() < <Self as Transport<C>>::MTU_SIZE {
            return Err(RxError::NonLastUnderUtilization);
        }

        if CanServiceId(frame.id.as_raw()).is_svc() {
            // Handle services
            let id = CanServiceId(frame.id.as_raw());

            // Ignore invalid frames
            if !id.valid() {
                return Err(RxError::InvalidCanId);
            }

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
            let id = CanMessageId(frame.id.as_raw());

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

    fn transmit<'a, X: Transfer<'a, C>>(
        transfer: &'a X,
    ) -> Result<Self::FrameIter<'a>, TxError> {
        CanIter::new(transfer, Some(1))
    }
}

/// Iterator type to transmit a transfer.
///
/// By splitting transmission into an iterator I can easily `.collect()` it for a handy
/// array, store it in another object, or just bulk transfer it all at once, without
/// having to commit to any proper memory model.
#[derive(Debug)]
pub struct CanIter<'a, C: embedded_time::Clock> {
    transfer_metadata: &'a TransferMetadata<C>,
    payload: &'a [u8],
    frame_id: ExtendedId,
    payload_offset: usize,
    crc: Crc16,
    crc_left: u8,
    toggle: bool,
    is_start: bool,
    can_frame: Option<CanFrame<C>>,
}

impl<'a, C: embedded_time::Clock> CanIter<'a, C> {
    pub fn new<X: Transfer<'a, C>>(
        transfer: &'a X,
        node_id: Option<NodeId>,
    ) -> Result<Self, TxError> {
        let frame_id = match transfer.metadata().transfer_kind {
            TransferKind::Message => {
                if node_id.is_none() && transfer.payload().len() > 7 {
                    return Err(TxError::AnonNotSingleFrame);
                }

                CanMessageId::new(transfer.metadata().priority, transfer.metadata().port_id, node_id)
            }
            TransferKind::Request => {
                // These runtime checks should be removed via proper typing further up but we'll
                // leave it as is for now.
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let metadata = transfer.metadata();
                let destination = metadata
                    .remote_node_id
                    .ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    metadata.priority,
                    true,
                    metadata.port_id,
                    destination,
                    source,
                )
            }
            TransferKind::Response => {
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let metadata = transfer.metadata();
                let destination = metadata
                    .remote_node_id
                    .ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    metadata.priority,
                    false,
                    metadata.port_id,
                    destination,
                    source,
                )
            }
        };

        Ok(Self {
            transfer_metadata: transfer.metadata(),
            payload: transfer.payload(),
            frame_id,
            payload_offset: 0,
            crc: Crc16::init(),
            crc_left: 2,
            toggle: true,
            is_start: true,
            can_frame: None,
        })
    }
}

// TODO can also impl regular Iterator on top of this
// or build out some core function, and use in both StreamingIterator and Iterator
impl<'a, C: Clock> StreamingIterator for CanIter<'a, C> {
    type Item = CanFrame<C>;

    fn get(&self) -> Option<&Self::Item> {
        self.can_frame.as_ref()
    }

    // I'm sure I could take an optimization pass at the logic here
    fn advance(&mut self) {
        let bytes_left = self.payload.len() - self.payload_offset;

        // Nothing left to transmit, we are done
        if bytes_left == 0 && self.crc_left == 0 {
            let _ = self.can_frame.take();
            return;
        }

        let is_end = bytes_left <= 7;
        let copy_len = core::cmp::min(bytes_left, 7);

        // TODO enough to use the transfer timestamp, or need actual timestamp
        let frame = self
            .can_frame
            .get_or_insert_with(|| CanFrame::new(self.transfer_metadata.timestamp, self.frame_id.as_raw()));

        frame.payload.clear();

        if self.is_start && is_end {
            // Single frame transfer, no CRC
            frame
                .payload
                .extend(self.payload[0..copy_len].iter().copied());
            self.payload_offset += bytes_left;
            self.crc_left = 0;
            unsafe {
                frame
                    .payload
                    .push_unchecked(TailByte::new(true, true, true, self.transfer_metadata.transfer_id).0)
            }
        } else {
            // Handle CRC
            let out_data =
                &self.payload[self.payload_offset..self.payload_offset + copy_len];
            self.crc.digest(out_data);
            frame.payload.extend(out_data.iter().copied());

            // Increment offset
            self.payload_offset += copy_len;

            // Finished with our data, now we deal with crc
            // (we can't do anything if bytes_left == 7, so ignore that case)
            if bytes_left < 7 {
                let crc = self.crc.get_crc().to_be_bytes();

                // TODO I feel like this logic could be cleaned up somehow
                if self.crc_left == 2 {
                    if 7 - bytes_left >= 2 {
                        // Iter doesn't work. Internal type is &u8 but extend
                        // expects u8
                        frame.payload.extend(crc.into_iter());
                        self.crc_left = 0;
                    } else {
                        // SAFETY: only written if we have enough space
                        unsafe {
                            frame.payload.push_unchecked(crc[0]);
                        }
                        self.crc_left = 1;
                    }
                } else if self.crc_left == 1 {
                    // SAFETY: only written if we have enough space
                    unsafe {
                        frame.payload.push_unchecked(crc[1]);
                    }
                    self.crc_left = 0;
                }
            }

            // SAFETY: should only copy at most 7 elements prior to here
            unsafe {
                frame.payload.push_unchecked(
                    TailByte::new(
                        self.is_start,
                        is_end,
                        self.toggle,
                        self.transfer_metadata.transfer_id,
                    )
                    .0,
                );
            }

            // Advance state of iter
            self.toggle = !self.toggle;
        }

        self.is_start = false;
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut bytes_left = self.payload.len() - self.payload_offset;

        // Single frame transfer
        if self.is_start && bytes_left <= 7 {
            return (1, Some(1));
        }

        // Multi-frame, so include CRC
        bytes_left += 2;
        let mut frames = bytes_left / 7;
        if bytes_left % 7 > 0 {
            frames += 1;
        }

        (frames, Some(frames))
    }
}

// TODO convert to embedded-hal PR type
/// Extended CAN frame (the only one supported by UAVCAN/CAN)
#[derive(Clone, Debug)]
pub struct CanFrame<C: embedded_time::Clock> {
    pub timestamp: Timestamp<C>,
    pub id: ExtendedId,
    pub payload: ArrayVec<[u8; 8]>,
}

impl<C: embedded_time::Clock> CanFrame<C> {
    fn new(timestamp: Timestamp<C>, id: u32) -> Self {
        Self {
            timestamp,
            // TODO get rid of this expect, it probably isn't necessary, just added quickly
            id: ExtendedId::new(id).expect("invalid ID"),
            payload: ArrayVec::<[u8; 8]>::new(),
        }
    }
}

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
