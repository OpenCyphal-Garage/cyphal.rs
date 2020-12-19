use num_traits::FromPrimitive;
use std::collections::hash_map::Entry;
use core::marker::PhantomData;

use crate::types::*;
use crate::RxError;
use crate::internal::InternalRxFrame;
use crate::Subscription;
use crate::transfer::Transfer;
use crate::transport::Transport;
use crate::{TransferKind, Priority};
use crate::session::SessionManager;

pub struct Node<S: SessionManager, T: Transport> {
    // TODO this is transport level type
    id: Option<NodeId>,

    pub sessions: S,

    /// Transport type
    transport: PhantomData<T>,
}

impl<S: SessionManager, T: Transport> Node<S, T> {
    // TODO needs to accept a SessionManager
    pub fn new(id: Option<NodeId>, session_manager: S) -> Self {
        Self {
            id,
            sessions: session_manager,
            transport: PhantomData,
        }
    }

    // Convenience function to access session manager inside of a closure.
    // I was going to use this because I was thinking I needed a closure
    // to access the session manager safely, but that isn't really the case.
    //
    // It still has potential to be useful (i.e. if you're using this with
    // an unsafe storage mechanism, the below form will prevent you from
    // taking references of the session manager), but idk if it actually is.
    //fn with_session_manager<R>(&mut self, f: fn(&mut T) -> R) -> R {
    //    f(&mut self.sessions)
    //}


    /// Attempts to receive frame. Returns error when frame is invalid, Some(Transfer) at the end of
    /// a transfer, and None if we haven't finished the transfer.
    pub fn try_receive_frame<'a>(
        &mut self,
        frame: &'a T::Frame,
    ) -> Result<Option<Transfer>, RxError> {
        // TODO check extended ID mask etc.
        let frame = self.transport.rx_process_frame(frame)?;

        if let Some(frame) = frame {
            match self.sessions.ingest(frame) {
                Ok(frame) => Ok(frame),
                Err(err) => Err(RxError::SessionError(err)),
            }
        } else {
            Ok(None)
        }
    }

    /// Create a series of frames to transmit.
    /// I think there could be 3 versions of this:
    /// 1. Returns a collection of frames to transmit.
    /// 2. Pushes frame onto queue, similar to libcanard.
    /// 3. Returns an iterator into a series of frames.
    ///
    /// 1 and 3 provide the user with more options but also make it harder
    /// to implement for the user.
    pub fn transmit(&self, transfer: &Transfer) -> Vec<T::Frame> {
        let mut frames = Vec::new();
        // TODO maybe a from_transfer fn
        let id = match transfer.transfer_kind {
            TransferKind::Message => {
                CanMessageId::new(
                    transfer.priority,
                    transfer.port_id,
                    self.id
                )
            }
            TransferKind::Request => {
                CanServiceId::new(
                    transfer.priority,
                    true,
                    transfer.port_id,
                    transfer.remote_node_id.unwrap(),
                    // TODO error handling
                    self.id.unwrap(),
                )
            }
            TransferKind::Response => {
                CanServiceId::new(
                    transfer.priority,
                    false,
                    transfer.port_id,
                    transfer.remote_node_id.unwrap(),
                    // TODO error handling
                    self.id.unwrap(),
                )
            }
        };

        if transfer.payload.len() <= 7 {
            // We can send as a single frame, so don't bother CRC
            // and extra loop semantics.
            let mut payload = Vec::from(transfer.payload.as_slice());
            payload.push(TailByte::new(
                true,
                true,
                true,
                transfer.transfer_id
            ));
            frames.push(CanFrame {
                timestamp: std::time::Instant::now(),
                id: id,
                payload: payload,
            })
        } else {
            let mut offset: usize = 0;
            let mut toggle = false;
            let mut crc = crc_any::CRCu16::crc16ccitt_false();
            // TODO probably split this into another function
            let mut payload = Vec::from(&transfer.payload[0..7]);
            payload.push(TailByte::new(
                true,
                false,
                true,
                transfer.transfer_id
            ));
            frames.push(CanFrame {
                timestamp: std::time::Instant::now(),
                id: id,
                payload: payload,
            });

            loop {
                // Amount of data to push into frame
                let data_len = core::cmp::max(transfer.payload.len() - offset, 7);
                let frame_data = &transfer.payload[offset..offset + data_len];
                let mut payload = Vec::from(frame_data);
                // I could do this as a first step as well
                crc.digest(frame_data);
                offset += data_len;

                let mut is_end = false;
                let mut extra_crc_frame = false;
                if data_len < 7 {
                    // We've hit the last frame

                    // Append CRC
                    let crc = &crc.get_crc().to_be_bytes();
                    // TODO I'm sure there's a way to reduce this
                    if 7 - data_len < 2 {
                        // CRC is split into a second frame, need to generate two
                        payload.push(crc[0]);
                        extra_crc_frame = true
                    } else {
                        payload.extend(crc);
                        is_end = true;
                    }
                }

                payload.push(TailByte::new(
                    false,
                    is_end,
                    toggle,
                    transfer.transfer_id
                ));
                toggle = !toggle;

                frames.push(CanFrame {
                    timestamp: std::time::Instant::now(),
                    id: id,
                    payload: payload,
                });

                // Place extra frame with last CRC byte at the end
                if extra_crc_frame {
                    let mut payload = Vec::new();
                    payload.push(crc.get_crc().to_be_bytes()[1]);
                    payload.push(TailByte::new(
                        false,
                        true,
                        toggle,
                        transfer.transfer_id
                    ));
                    frames.push(CanFrame {
                        timestamp: std::time::Instant::now(),
                        id: id,
                        payload: payload,
                    });
                }

                if is_end || extra_crc_frame {
                    break;
                }
            }
        }

        // TODO represent CAN frame with slice into payload *and*
        // transfer byte instead of copying everything into the payload.
        frames
    }
}