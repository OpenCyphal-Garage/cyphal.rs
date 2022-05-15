//! UAVCAN/CAN-FD transport implementation WIP

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
use crate::transfer::Transfer;

/// Unit struct for declaring transport type
#[derive(Copy, Clone, Debug)]
pub struct FdCan;

impl<C: embedded_time::Clock + 'static> Transport<C> for FdCan {
    type Frame = FdCanFrame<C>;
    type FrameIter<'a, X: Transfer<'a, C>> = FdCanIter<'a, C, X> where X: 'a;

    const MTU_SIZE: usize = 64;

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
        transfer: &X,
    ) -> Result<Self::FrameIter<'a>, TxError> {
        FdCanIter::new(transfer, Some(1))
    }
}

/// Iterator type to transmit a transfer.
///
/// By splitting transmission into an iterator I can easily `.collect()` it for a handy
/// array, store it in another object, or just bulk transfer it all at once, without
/// having to commit to any proper memory model.
#[derive(Debug)]
pub struct FdCanIter<'a, C: embedded_time::Clock, X: Transfer<'a, C>> {
    transfer: &'a X,
    frame_id: ExtendedId,
    payload_offset: usize,
    crc: Crc16,
    crc_left: u8,
    toggle: bool,
    is_start: bool,
    can_frame: Option<FdCanFrame<C>>,
}

// TODO there must be a way to link the MTU sizes into here? I tried with const generics but that got complicated and unstable
impl<'a, C: embedded_time::Clock, X: Transfer<'a, C>> FdCanIter<'a, C, X> {
    pub fn new(
        transfer: &'a X,
        node_id: Option<NodeId>,
    ) -> Result<Self, TxError> {
        let frame_id = match transfer.transfer_kind() {
            TransferKind::Message => {
                if node_id.is_none() && transfer.payload().len() > 63 {
                    return Err(TxError::AnonNotSingleFrame);
                }

                CanMessageId::new(transfer.priority(), transfer.port_id(), node_id)
            }
            TransferKind::Request => {
                // These runtime checks should be removed via proper typing further up but we'll
                // leave it as is for now.
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let destination = transfer
                    .remote_node_id()
                    .ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    transfer.priority(),
                    true,
                    transfer.port_id(),
                    destination,
                    source,
                )
            }
            TransferKind::Response => {
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let destination = transfer
                    .remote_node_id()
                    .ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    transfer.priority(),
                    false,
                    transfer.port_id(),
                    destination,
                    source,
                )
            }
        };

        Ok(Self {
            transfer,
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

impl<'a, C: Clock, X: Transfer<'a, C>> StreamingIterator for FdCanIter<'a, C, X> {
    type Item = FdCanFrame<C>;

    fn get(&self) -> Option<&Self::Item> {
        self.can_frame.as_ref()
    }

    // I'm sure I could take an optimization pass at the logic here
    fn advance(&mut self) {
        let bytes_left = self.transfer.payload().len() - self.payload_offset;

        if bytes_left == 0 && self.crc_left == 0 {
            let _ = self.can_frame.take();
            return;
        }

        let is_end = bytes_left <= 63;
        let mut copy_len = core::cmp::min(bytes_left, 63);

        let frame = self.can_frame.get_or_insert_with(|| {
            FdCanFrame::new(self.transfer.timestamp(), self.frame_id.as_raw())
        });

        if self.is_start && is_end {
            // Single frame transfer, no CRC
            frame
                .payload
                .extend(self.transfer.payload()[0..copy_len].iter().copied());
            self.payload_offset += bytes_left;
            unsafe {
                frame
                    .payload
                    .push_unchecked(TailByte::new(true, true, true, self.transfer.transfer_id()).0)
            }
        } else {
            // Handle CRC
            let out_data =
                &self.transfer.payload()[self.payload_offset..self.payload_offset + copy_len];
            self.crc.digest(out_data);
            frame.payload.extend(out_data.iter().copied());

            // Increment offset
            self.payload_offset += copy_len;

            // Finished with our data, now we deal with crc
            // (we can't do anything if bytes_left == 7, so ignore that case)
            if bytes_left < 63 {
                let crc = &self.crc.get_crc().to_be_bytes();

                // TODO I feel like this logic could be cleaned up somehow
                if self.crc_left == 2 {
                    if 63 - bytes_left >= 2 {
                        // Iter doesn't work. Internal type is &u8 but extend
                        // expects u8
                        frame.payload.push(crc[0]);
                        frame.payload.push(crc[1]);
                        self.crc_left = 0;
                        copy_len += 2;
                    } else {
                        // SAFETY: only written if we have enough space
                        unsafe {
                            frame.payload.push_unchecked(crc[0]);
                        }
                        self.crc_left = 1;
                        copy_len += 1;
                    }
                } else if self.crc_left == 1 {
                    // SAFETY: only written if we have enough space
                    unsafe {
                        frame.payload.push_unchecked(crc[1]);
                    }
                    self.crc_left = 0;
                    copy_len += 1;
                }
            }

            // SAFETY: should only copy at most 7 elements prior to here
            unsafe {
                frame.payload.push_unchecked(
                    TailByte::new(
                        self.is_start,
                        is_end,
                        self.toggle,
                        self.transfer.transfer_id(),
                    )
                    .0,
                );
            }
            copy_len += 1;

            // Advance state of iter
            self.toggle = !self.toggle;
        }

        self.is_start = false;

        // Set DLC and 0-pad remaining bytes in DLC length
        // TODO find a better solution for this
        let zeroes = [0u8; 16];
        frame.dlc = match copy_len {
            0..=8 => copy_len as u8,
            9..=12 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..12 - copy_len])
                    .unwrap();
                9
            }
            13..=16 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..16 - copy_len])
                    .unwrap();
                10
            }
            17..=20 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..20 - copy_len])
                    .unwrap();
                11
            }
            21..=24 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..24 - copy_len])
                    .unwrap();
                12
            }
            25..=32 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..32 - copy_len])
                    .unwrap();
                13
            }
            33..=48 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..48 - copy_len])
                    .unwrap();
                14
            }
            49..=64 => {
                frame
                    .payload
                    .try_extend_from_slice(&zeroes[0..64 - copy_len])
                    .unwrap();
                15
            }
            _ => panic!("Copied data should never exceed 64 bytes!"),
        };
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut bytes_left = self.transfer.payload().len() - self.payload_offset;

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
/// Extended CAN FD frame (the only one supported by UAVCAN/CAN)
#[derive(Clone, Debug)]
pub struct FdCanFrame<C: embedded_time::Clock> {
    pub timestamp: Timestamp<C>,
    pub id: ExtendedId,
    // Seperate here because there are extra semantics around CAN-FD DLC
    pub dlc: u8,
    pub payload: ArrayVec<[u8; 64]>,
}

impl<C: embedded_time::Clock> FdCanFrame<C> {
    fn new(timestamp: Timestamp<C>, id: u32) -> Self {
        Self {
            timestamp,
            // TODO get rid of this expect, it probably isn't necessary, just added quickly
            id: ExtendedId::new(id).expect("invalid ID"),
            dlc: 0,
            payload: ArrayVec::<[u8; 64]>::new(),
        }
    }
}
