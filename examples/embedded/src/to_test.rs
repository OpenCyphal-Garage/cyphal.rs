use arrayvec::ArrayVec;
use defmt::info;
use embedded_time::{duration::Milliseconds, Clock};
use stm32g4xx_hal::{
    block,
    fdcan::{
        frame::TxFrameHeader,
        id::{ExtendedId, Id},
        FdCan, NormalOperationMode,
    },
    stm32::FDCAN1,
    timer::MonoTimer,
};
use uavcan::{
    session::HeapSessionManager,
    transfer::Transfer,
    transport::can::{Can, CanFrame, CanMessageId, CanMetadata, CanServiceId, TailByte},
    types::NodeId,
    Node, TransferKind, TxError,
};

use num_traits::cast::ToPrimitive;

use crate::{clock::StmClock, util::insert_u8_array_in_u32_array};

#[no_mangle]
pub fn publish(
    clock: &MonoTimer,
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    transfer: Transfer<StmClock>,
    can: &mut FdCan<FDCAN1, NormalOperationMode>,
) {
    let mut iter = get_iter_from_transfer(node, &transfer, clock);
    let start = clock.now();
    let frame = get_next_frame(&mut iter);
    let elapsed = start.elapsed();
    transmit_fdcan(frame, can);

    let micros: u32 = clock.frequency().duration(elapsed).0;
    // info!("elapsed: {} micros", micros);
}

#[no_mangle]
fn get_next_frame(iter: &mut CanIter<StmClock>) -> CanFrame<StmClock> {
    iter.next().unwrap()
}

#[no_mangle]
fn get_iter_from_transfer<'a>(
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    transfer: &'a Transfer<StmClock>,
    clock: &MonoTimer,
) -> CanIter<'a, StmClock> {
    // let start = clock.now();
    let iter = CanIter::new(transfer, Some(2334), clock).unwrap();
    // let elapsed = start.elapsed();
    // let micros: u32 = clock.frequency().duration(elapsed).0;
    // info!("elapsed: {} micros", micros);

    iter
}

#[no_mangle]
fn transmit_fdcan(frame: CanFrame<StmClock>, can: &mut FdCan<FDCAN1, NormalOperationMode>) {
    let header = TxFrameHeader {
        bit_rate_switching: false,
        frame_format: stm32g4xx_hal::fdcan::frame::FrameFormat::Standard,
        id: Id::Extended(ExtendedId::new(frame.id).unwrap()),
        len: frame.payload.len() as u8,
        marker: None,
    };
    block!(can.transmit(header, &mut |b| {
        insert_u8_array_in_u32_array(&frame.payload, b)
    },))
    .unwrap();
}

#[derive(Debug)]
pub struct CanIter<'a, C: embedded_time::Clock> {
    transfer: &'a Transfer<'a, C>,
    frame_id: u32,
    payload_offset: usize,
    crc: crc_any::CRCu16,
    crc_left: u8,
    toggle: bool,
    is_start: bool,
}

impl<'a, C: embedded_time::Clock> CanIter<'a, C> {
    pub fn new(
        transfer: &'a Transfer<C>,
        node_id: Option<NodeId>,
        clock: &MonoTimer,
    ) -> Result<Self, TxError> {
        let frame_id = match transfer.transfer_kind {
            TransferKind::Message => {
                if node_id.is_none() && transfer.payload.len() > 7 {
                    return Err(TxError::AnonNotSingleFrame);
                }

                CanMessageId::new(transfer.priority, transfer.port_id, node_id)
                    .to_u32()
                    .unwrap()
            }
            TransferKind::Request => {
                // These runtime checks should be removed via proper typing further up but we'll
                // leave it as is for now.
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let destination = transfer
                    .remote_node_id
                    .ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    transfer.priority,
                    true,
                    transfer.port_id,
                    destination,
                    source,
                )
                .to_u32()
                .unwrap()
            }
            TransferKind::Response => {
                let source = node_id.ok_or(TxError::ServiceNoSourceID)?;
                let destination = transfer
                    .remote_node_id
                    .ok_or(TxError::ServiceNoDestinationID)?;
                CanServiceId::new(
                    transfer.priority,
                    false,
                    transfer.port_id,
                    destination,
                    source,
                )
                .to_u32()
                .unwrap()
            }
        };

        let crc = crc_any::CRCu16::crc16ccitt_false();

        let start = clock.now();
        let can_iter = Ok(Self {
            transfer,
            frame_id,
            payload_offset: 0,
            crc,
            crc_left: 2,
            toggle: true,
            is_start: true,
        });
        let elapsed = start.elapsed();

        let micros: u32 = clock.frequency().duration(elapsed).0;
        info!("elapsed: {} micros", micros);

        can_iter
    }
}

impl<'a, C: Clock> Iterator for CanIter<'a, C> {
    type Item = CanFrame<C>;

    // I'm sure I could take an optimization pass at the logic here
    fn next(&mut self) -> Option<Self::Item> {
        let mut frame = CanFrame {
            // TODO enough to use the transfer timestamp, or need actual timestamp
            timestamp: self.transfer.timestamp,
            id: self.frame_id,
            payload: ArrayVec::new(),
        };

        let bytes_left = self.transfer.payload.len() - self.payload_offset;
        let is_end = bytes_left <= 7;
        let copy_len = core::cmp::min(bytes_left, 7);

        if self.is_start && is_end {
            // Single frame transfer, no CRC
            frame
                .payload
                .extend(self.transfer.payload[0..copy_len].iter().copied());
            self.payload_offset += bytes_left;
            unsafe {
                frame.payload.push_unchecked(
                    TailByte::new(true, true, true, self.transfer.transfer_id)
                        .to_u8()
                        .unwrap(),
                )
            }
        } else {
            // Nothing left to transmit, we are done
            if bytes_left == 0 && self.crc_left == 0 {
                return None;
            }

            // Handle CRC
            let out_data =
                &self.transfer.payload[self.payload_offset..self.payload_offset + copy_len];
            self.crc.digest(out_data);
            frame.payload.extend(out_data.iter().copied());

            // Increment offset
            self.payload_offset += copy_len;

            // Finished with our data, now we deal with crc
            // (we can't do anything if bytes_left == 7, so ignore that case)
            if bytes_left < 7 {
                let crc = &self.crc.get_crc().to_be_bytes();

                // TODO I feel like this logic could be cleaned up somehow
                if self.crc_left == 2 {
                    if 7 - bytes_left >= 2 {
                        // Iter doesn't work. Internal type is &u8 but extend
                        // expects u8
                        frame.payload.push(crc[0]);
                        frame.payload.push(crc[1]);
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
                frame.payload.push_unchecked(TailByte::new(
                    self.is_start,
                    is_end,
                    self.toggle,
                    self.transfer.transfer_id,
                ));
            }

            // Advance state of iter
            self.toggle = !self.toggle;
        }

        self.is_start = false;

        Some(frame)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut bytes_left = self.transfer.payload.len() - self.payload_offset;

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
