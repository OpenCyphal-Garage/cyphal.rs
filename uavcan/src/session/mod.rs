//! Session management.
//!
//! UAVCAN defines a session as an identifier of a collection of transfers
//! between a given set of agents. This module provides several
//! different implementations of a session manager, which should be chosen
//! based on the memory model of the node being implemented. A library
//! user can also implement their own session to fit their individual needs
//! using the SessionManager trait.

use crate::types::*;
use crate::transfer::Transfer;
use crate::internal::InternalRxFrame;
use crate::RxError;

mod std_vec;

pub use std_vec::StdVecSessionManager;

pub enum SessionError {
    OutOfSpace,
    NoSubscription,
    Timeout,
    NewSessionNoStart,
    InvalidTransferId,
    // TODO come up with a way to return more specific errors
    BadMetadata,
}

pub enum SubscriptionError {
    OutOfSpace,
    SubscriptionExists,
    SubscriptionDoesNotExist,
}

// TODO I need to remove my handling of CRC and tail bytes. That is
// transport specific. I need to ingest bytes, and the metadata given
// to me by the transport implementation.
//
// In the concrete CRC implementation, I need to just update CRC if
// there were no errors, after I run ingest.

// TODO use this to implement no_std static, no_std allocator, and std versions
/// Trait to declare a session manager. This is responsible for managing
/// subscriptions and ongoing sessions.
///
/// The intent here is to provide an interface to easily define
/// what management strategy you want to implement. This allows you to
/// select different models based on e.g. your memory allocation strategy,
/// or if a model provided by this crate does not suffice, you can implement
/// your own.
pub trait SessionManager {

    // These shouldn't be added because there can be inherent differences in
    // how subscriptions are managed. Subscribing in a pure-static impl
    // is different than in a vec-based approach.
    /// Add a subscription to the list. Not all implementations have to be fallible, but
    /// static implementations must be.
    //fn subscribe(&mut self, subscription: crate::Subscription) -> Result<Self::SubscriptionIdent, SubscriptionError>;
    // TODO does this need to be fallible
    /// Remove a subscription.
    //fn unsubscribe(&mut self, subscription: Self::SubscriptionIdent) -> Result<(), SubscriptionError>;


    /// Process incoming frame.
    fn ingest(&mut self, frame: InternalRxFrame) -> Result<Option<Transfer>, SessionError>;

    /// Housekeeping function called to clean up timed-out sessions.
    fn update_sessions(&mut self, timestamp: Timestamp);

    /// Helper function to match frames to the correct subscription.
    ///
    /// It's not necessary to use this in your implementation, you may have
    /// a more efficient way to check, but this is a spec-compliant function.
    fn matches_sub(subscription: &crate::Subscription, frame: &InternalRxFrame) -> bool {
        // Order is chosen to short circuit the most common inconsistencies.
        if frame.port_id != subscription.port_id {
            return false;
        }
        if frame.transfer_kind != subscription.transfer_kind {
            return false;
        }

        return true;
    }
}

// This struct needs to be moved *internal* to each SessionManager
pub struct Session<T: crate::transport::SessionMetadata> {
    // Timestamp of first frame
    pub timestamp: Option<Timestamp>,
    pub total_payload_size: usize,
    pub payload: Vec<u8>,
    pub crc: crc_any::CRCu16,
    pub transfer_id: TransferId,
    pub toggle: bool,

    pub metadata: T,
}

impl<T: crate::transport::SessionMetadata> Session<T> {
    pub fn new(transfer_id: TransferId) -> Self {
        Self {
            timestamp: None,
            total_payload_size: 0,
            payload: Vec::new(),
            // TODO uh oh this is transport-specific
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