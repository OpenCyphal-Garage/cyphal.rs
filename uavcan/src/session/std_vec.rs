//! Full std::collections based SessionManager implementation.
//!
//! This is intended to be the lowest-friction interface to get
//! started, both for library development and eventually for using the library.

use crate::session::*;
use crate::types::NodeId;

use std::collections::HashMap;
use std::vec::Vec;

/// Internal session object.
#[derive(Clone, Debug)]
struct Session<T: crate::transport::SessionMetadata<C>, C> {
    // Timestamp of first frame
    pub timestamp: Option<Timestamp<C>>,
    pub payload: Vec<u8>,
    pub transfer_id: TransferId,

    pub md: T,
}

impl<T: crate::transport::SessionMetadata<C>, C> Session<T, C> {
    pub fn new(transfer_id: TransferId) -> Self {
        Self {
            timestamp: None,
            payload: Vec::new(),
            transfer_id,
            md: T::new(),
        }
    }
}

/// Internal subscription object. Contains hash map of sessions.
struct Subscription<T: crate::transport::SessionMetadata<C>, C> {
    sub: crate::Subscription,
    sessions: HashMap<NodeId, Session<T>>,
}

fn timestamp_expired<C>(
    timeout: core::time::Duration,
    now: Timestamp<C>,
    then: Option<Timestamp<C>>,
) -> bool {
    if let Some(then) = then {
        if now - then > timeout {
            return true;
        }
    }

    return false;
}

impl<T: crate::transport::SessionMetadata<C>, C> Subscription<T> {
    pub fn new(sub: crate::Subscription) -> Self {
        Self {
            sub,
            sessions: HashMap::new(),
        }
    }

    /// Update subscription with incoming frame
    fn update(&mut self, frame: InternalRxFrame<C>) -> Result<Option<Transfer<C>>, SessionError> {
        // TODO maybe some of the logic here can be skipped with anon transfers.
        let session = frame.source_node_id.unwrap();
        // Create default session if it doesn't exist
        if !self.sessions.contains_key(&session) {
            if !frame.start_of_transfer {
                return Err(SessionError::NewSessionNoStart);
            }
            self.sessions
                .insert(session, Session::new(frame.transfer_id));
        }

        if self.sessions[&session].transfer_id != frame.transfer_id {
            // Create new session
            self.sessions.entry(session).and_modify(|s| {
                *s = Session::new(frame.transfer_id);
            });
        } else {
            // Check for session expiration
            if timestamp_expired(
                self.sub.timeout,
                frame.timestamp,
                self.sessions[&session].timestamp,
            ) {
                let transfer_id = self.sessions[&session].transfer_id;
                self.sessions.entry(session).and_modify(|s| {
                    *s = Session::new(transfer_id);
                });
                return Err(SessionError::Timeout);
            }
        }

        self.accept_frame(session, frame)
    }

    fn accept_frame(
        &mut self,
        session: NodeId,
        frame: InternalRxFrame<C>,
    ) -> Result<Option<Transfer<C>>, SessionError> {
        let mut session = self.sessions.get_mut(&session).unwrap();

        if frame.start_of_transfer {
            session.timestamp = Some(frame.timestamp);
        }

        if let Some(len) = session.md.update(&frame) {
            // Truncate payload if subscription extent is less than the incoming data
            let payload_to_copy = if session.payload.len() + len > self.sub.extent {
                session.payload.len() + len - self.sub.extent
            } else {
                len
            };
            session.payload.extend(&frame.payload[0..payload_to_copy]);

            if frame.end_of_transfer {
                if session.md.is_valid(&frame) {
                    Ok(Some(Transfer::from_frame(
                        frame,
                        session.timestamp.unwrap(),
                        &session.payload,
                    )))
                } else {
                    Err(SessionError::BadMetadata)
                }
            } else {
                Ok(None)
            }
        } else {
            Err(SessionError::BadMetadata)
        }
    }
}

/// SessionManager based on full std support. Meant to be lowest
/// barrier to entry and greatest flexibility at the cost of resource usage
/// and not being no_std.
pub struct StdVecSessionManager<T: crate::transport::SessionMetadata<C>, C> {
    subscriptions: Vec<Subscription<T>>,
}

impl<T: crate::transport::SessionMetadata<C>, C> StdVecSessionManager<T, C> {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    /// Add a subscription
    pub fn subscribe(
        &mut self,
        subscription: crate::Subscription,
    ) -> Result<(), SubscriptionError> {
        if self.subscriptions.iter().any(|s| s.sub == subscription) {
            return Err(SubscriptionError::SubscriptionExists);
        }

        self.subscriptions.push(Subscription::new(subscription));
        Ok(())
    }

    /// Modify subscription in place, creating a new one if not found.
    pub fn edit_subscription(
        &mut self,
        subscription: crate::Subscription,
    ) -> Result<(), SubscriptionError> {
        match self
            .subscriptions
            .iter()
            .position(|s| s.sub == subscription)
        {
            Some(pos) => {
                self.subscriptions[pos] = Subscription::new(subscription);
                Ok(())
            }
            None => Err(SubscriptionError::SubscriptionDoesNotExist),
        }
    }

    /// Removes a subscription from the list.
    pub fn unsubscribe(
        &mut self,
        subscription: crate::Subscription,
    ) -> Result<(), SubscriptionError> {
        match self
            .subscriptions
            .iter()
            .position(|x| x.sub == subscription)
        {
            Some(pos) => {
                self.subscriptions.remove(pos);
                Ok(())
            }
            None => Err(SubscriptionError::SubscriptionDoesNotExist),
        }
    }
}

impl<T: crate::transport::SessionMetadata<C>, C> SessionManager<C> for StdVecSessionManager<T, C> {
    fn ingest(&mut self, frame: InternalRxFrame<C>) -> Result<Option<Transfer<C>>, SessionError> {
        match self
            .subscriptions
            .iter_mut()
            .find(|sub| Self::matches_sub(&sub.sub, &frame))
        {
            Some(subscription) => subscription.update(frame),
            None => Ok(None),
        }
    }

    fn update_sessions(&mut self, timestamp: Timestamp<C>) {
        for sub in &mut self.subscriptions {
            for session in sub.sessions.values_mut() {
                if timestamp_expired(sub.sub.timeout, timestamp, session.timestamp) {
                    let transfer_id = session.transfer_id;
                    *session = Session::new(transfer_id);
                }
            }
        }
    }
}
