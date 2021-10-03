//! Session management.
//!
//! UAVCAN defines a session as an identifier of a collection of transfers
//! between a given set of agents. This module provides several
//! different implementations of a session manager, which should be chosen
//! based on the memory model of the node being implemented. A library
//! user can also implement their own session to fit their individual needs
//! using the SessionManager trait.

use crate::internal::InternalRxFrame;
use crate::transfer::Transfer;
use crate::types::*;

#[cfg(not(feature = "std"))]
mod pool_vec;
#[cfg(feature = "std")]
mod std_vec;

#[cfg(feature = "std")]
pub use std_vec::StdVecSessionManager;

use embedded_time::fixed_point::FixedPoint;

/// Session-related errors, caused by reception errors.
#[derive(Copy, Clone, Debug)]
pub enum SessionError {
    OutOfSpace,
    NoSubscription,
    Timeout,
    NewSessionNoStart,
    InvalidTransferId,
    // TODO come up with a way to return more specific errors
    BadMetadata,
}

/// Errors caused when adding a subscription.
#[derive(Copy, Clone, Debug)]
pub enum SubscriptionError {
    OutOfSpace,
    SubscriptionExists,
    SubscriptionDoesNotExist,
}

/// Trait to declare a session manager. This is responsible for managing
/// subscriptions and ongoing sessions.
///
/// The intent here is to provide an interface to easily define
/// what management strategy you want to implement. This allows you to
/// select different models based on e.g. your memory allocation strategy,
/// or if a model provided by this crate does not suffice, you can implement
/// your own.
pub trait SessionManager<C: embedded_time::Clock> {
    /// Process incoming frame.
    fn ingest(&mut self, frame: InternalRxFrame<C>) -> Result<Option<Transfer<C>>, SessionError>;

    /// Housekeeping function called to clean up timed-out sessions.
    fn update_sessions(&mut self, timestamp: Timestamp<C>);

    /// Helper function to match frames to the correct subscription.
    ///
    /// It's not necessary to use this in your implementation, you may have
    /// a more efficient way to check, but this is a spec-compliant function.
    fn matches_sub<D: embedded_time::duration::Duration + FixedPoint>(
        subscription: &crate::Subscription<D>,
        frame: &InternalRxFrame<C>,
    ) -> bool {
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

fn timestamp_expired<C: embedded_time::Clock, D>(
    timeout: D,
    now: Timestamp<C>,
    then: Option<Timestamp<C>>,
) -> bool
where
    D: embedded_time::duration::Duration + FixedPoint,
    <C as embedded_time::Clock>::T: From<<D as FixedPoint>::T>,
{
    if let Some(then) = then {
        if now - then > timeout.to_generic(C::SCALING_FACTOR).unwrap() {
            return true;
        }
    }

    return false;
}
