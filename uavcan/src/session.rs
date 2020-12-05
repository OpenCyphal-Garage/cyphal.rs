use crate::types::*;
use crate::transfer::Transfer;
use crate::internal::InternalRxFrame;
use crate::RxError;

use crc_any::CRC;

// TODO use this to implement no_std static, no_std allocator, and std versions
trait SessionManager {}

pub struct Session {
    // Timestamp of first frame
    pub timestamp: Option<Timestamp>,
    pub total_payload_size: usize,
    pub payload: Vec<u8>,
    pub crc: crc_any::CRCu16,
    pub transfer_id: TransferId,
    pub toggle: bool,
}

impl Session {
    pub fn new(transfer_id: TransferId) -> Self {
        Self {
            timestamp: None,
            total_payload_size: 0,
            payload: Vec::new(),
            crc: crc_any::CRCu16::crc16ccitt_false(),
            transfer_id,
            toggle: false,
        }
    }

    pub fn update<'a>(
        &mut self,
        frame: InternalRxFrame<'a>,
        timeout: core::time::Duration,
        extent: usize,
    ) -> Result<Option<Transfer>, RxError> {
        // TODO check transport index

        // Check timeouts for session
        if let Some(last_time) = self.timestamp {
            if frame.timestamp - last_time > timeout {
                // Reset session instance
                *self = Self::new(self.transfer_id);

                if !frame.start_of_transfer {
                    return Err(RxError::Timeout);
                }
            }
        }

        // I have no idea why the diff check is > 1 in libcanard...
        if self.transfer_id != frame.transfer_id {
            *self = Self::new(frame.transfer_id);
            if !frame.start_of_transfer {
                return Err(RxError::InvalidTransferId);
            }
        }

        // Pull in frame
        self.accept_frame(frame, extent)
    }

    fn accept_frame<'a>(
        &mut self,
        frame: InternalRxFrame<'a>,
        extent: usize,
    ) -> Result<Option<Transfer>, RxError> {
        // Timestamp only gets updated from first frame
        if frame.start_of_transfer {
            self.timestamp = Some(frame.timestamp);
        }

        // Update CRC if it isn't a single frame
        // NOTE: payload is evaluated even if data is truncated
        let single_frame = frame.start_of_transfer && frame.end_of_transfer;
        if !single_frame {
            self.crc.digest(frame.payload);
        }

        // Read in payload, truncating
        let payload_to_copy = if self.payload.len() + frame.payload.len() > extent {
            self.payload.len() + frame.payload.len() - extent
        } else {
            frame.payload.len()
        };
        self.payload
            .extend_from_slice(&frame.payload[0..payload_to_copy]);
        self.total_payload_size += frame.payload.len();

        if frame.end_of_transfer {
            let mut out = None;
            // Single frames or when our CRC has completed it's check means we've finished the transfer
            // TODO proper check and error for invalid CRC at end of transfer
            if single_frame || self.crc.get_crc() == 0x0000u16 {
                // Don't pass CRC to the user
                let truncated_size = self.total_payload_size - self.payload.len();
                // If we have not already truncated the CRC, remove it from the output
                let real_payload = if !single_frame && truncated_size < 2 {
                    self.payload.len() - 2 - truncated_size
                } else {
                    self.payload.len()
                };
                out = Some(Transfer::from_frame(
                    frame,
                    self.timestamp.unwrap(),
                    &self.payload[0..real_payload],
                ));
            }
            // TODO maybe use a different function for this reset
            *self = Self::new(self.transfer_id);
            Ok(out)
        } else {
            self.toggle = !self.toggle;
            Ok(None)
        }
    }
}
