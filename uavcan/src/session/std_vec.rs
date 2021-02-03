//! Full std::collections based SessionManager implementation.
//!
//! This is intended to be the lowest-friction interface to get
//! started, both for library development and eventually for using the library.

use crate::session::*;
use crate::types::NodeId;

use std::collections::HashMap;

/// Internal session object.
#[derive(Clone, Debug)]
struct Session<T: crate::transport::SessionMetadata> {
    // Timestamp of first frame
    pub timestamp: Option<Timestamp>,
    pub payload: Vec<u8>,
    pub transfer_id: TransferId,

    pub md: T,
}

impl<T: crate::transport::SessionMetadata> Session<T> {
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
struct Subscription<T: crate::transport::SessionMetadata> {
    sub: crate::Subscription,
    sessions: HashMap<NodeId, Session<T>>,
}

fn timestamp_expired(
    timeout: core::time::Duration,
    now: Timestamp,
    then: Option<Timestamp>,
) -> bool {
    if let Some(then) = then {
        if now - then > timeout {
            return true;
        }
    }

    return false;
}

impl<T: crate::transport::SessionMetadata> Subscription<T> {
    pub fn new(sub: crate::Subscription) -> Self {
        Self {
            sub,
            sessions: HashMap::new(),
        }
    }

    /// Update subscription with incoming frame
    fn update(&mut self, frame: InternalRxFrame) -> Result<Option<Transfer>, SessionError> {
        // TODO anon transfers should be handled by the protocol.
        // although for good error handling we should handle the error here
        let session = frame.source_node_id.unwrap();
        // Create default session if it doesn't exist
        if !self.sessions.contains_key(&session) {
            if !frame.start_of_transfer {
                return Err(SessionError::NewSessionNoStart);
            }
            self.sessions
                .insert(session, Session::new(frame.transfer_id));
        }

        // TODO proper check for invalid new transfer ID
        if self.sessions[&session].transfer_id != frame.transfer_id {
            // Create new session
            // TODO we don't necessarily want to overwrite the session immediately
            // if we get a new transfer id
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
        frame: InternalRxFrame,
    ) -> Result<Option<Transfer>, SessionError> {
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
pub struct StdVecSessionManager<T: crate::transport::SessionMetadata> {
    subscriptions: Vec<Subscription<T>>,
}

impl<T: crate::transport::SessionMetadata> StdVecSessionManager<T> {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    // TODO make it update an existing subscription?
    // Idk if we want to support that.
    // maybe a seperate function.
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

impl<T: crate::transport::SessionMetadata> SessionManager for StdVecSessionManager<T> {
    fn ingest(&mut self, frame: InternalRxFrame) -> Result<Option<Transfer>, SessionError> {
        match self
            .subscriptions
            .iter_mut()
            .find(|sub| Self::matches_sub(&sub.sub, &frame))
        {
            Some(subscription) => subscription.update(frame),
            // TODO I don't think this should be an error
            //None => Err(SessionError::NoSubscription),
            None => Ok(None),
        }
    }

    fn update_sessions(&mut self, timestamp: Timestamp) {
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
