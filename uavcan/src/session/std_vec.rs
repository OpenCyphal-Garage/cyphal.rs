use crate::types::NodeId;
use crate::session::*;

use std::collections::{HashMap, hash_map::Entry};

/// Internal session object.
struct Session {
    // Timestamp of first frame
    pub timestamp: Option<Timestamp>,
    pub payload: Vec<u8>,
    pub transfer_id: TransferId,
    pub toggle: bool,
}

impl Session {
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
}


/// Internal subscription object. Contains hash map of sessions.
struct Subscription {
    sub: crate::Subscription,
    sessions: HashMap<NodeId, Session>,
}

impl Subscription {
    pub fn new(sub: crate::Subscription) -> Self {
        Self {
            sub,
            sessions: HashMap::new(),
        }
    }

    pub fn timestamp_expired(&self, now: Timestamp, then: Option<Timestamp>) -> bool {
        if let Some(then) = then {
            if now - then > self.sub.timeout {
                return true;
            }
        }

        return false;
    }

    /// Update subscription with incoming frame
    fn update(&mut self, frame: InternalRxFrame) -> Result<Option<Transfer>, SessionError> {
        // TODO anon transfers should be handled by the protocol.
        // although for good error handling we should handle the error here
        let source_node_id = frame.source_node_id.unwrap();
        let mut session = match self.sessions.entry(source_node_id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                // We didn't receive the start of transfer frame
                // Transfers must be sent/received in order.
                if !frame.start_of_transfer {
                    return Err(SessionError::NewSessionNoStart);
                }

                // Create a new session
                entry.insert(Session::new(frame.transfer_id))
            }
        };

        // check timeout, transfer id, then new start
        if self.timestamp_expired(frame.timestamp, session.timestamp) {
            *session = Session::new(session.transfer_id);
            return Err(SessionError::Timeout);
        }

        // TODO proper check for invalid new transfer ID
        if session.transfer_id != frame.transfer_id {
            // Create new session
            // TODO we don't necessarily want to overwrite the session immediately
            // if we get a new transfer id
            *session = Session::new(frame.transfer_id);
        }

        self.accept_frame(session, frame)
    }

    fn accept_frame(
        &mut self,
        session: &mut Session,
        frame: InternalRxFrame,
    ) -> Result<Option<Transfer>, SessionError> {
        if frame.start_of_transfer {
            session.timestamp = Some(frame.timestamp);
        }

        session.payload.extend(frame.payload);

        Ok(None)
    }
}

/// SessionManager based on full std support. Meant to be lowest
/// barrier to entry and greatest flexibility at the cost of resource usage
/// and not being no_std.
pub struct StdVecSessionManager {
    subscriptions: Vec<Subscription>,
}

impl StdVecSessionManager {
    // TODO make it update an existing subscription?
    // Idk if we want to support that.
    // maybe a seperate function.
    /// Add a subscription 
    pub fn subscribe(&mut self, subscription: crate::Subscription) -> Result<(), SubscriptionError> {
        if self.subscriptions.iter().any(|s| s.sub == subscription) {
            return Err(SubscriptionError::SubscriptionExists);
        }

        self.subscriptions.push(Subscription::new(subscription));
        Ok(())
    }

    pub fn unsubscribe(&mut self, subscription: crate::Subscription) -> Result<(), SubscriptionError> {
        match self.subscriptions.iter().position(|x| x.sub == subscription) {
            Some(pos) => {
                self.subscriptions.remove(pos);
                Ok(())
            }
            None => Err(SubscriptionError::SubscriptionDoesNotExist),
        }
    }
}

impl SessionManager for StdVecSessionManager {
    fn ingest(&mut self, frame: InternalRxFrame) -> Result<Option<Transfer>, SessionError> {
        match self.subscriptions.iter().find(|sub| {
            Self::matches_sub(&sub.sub, &frame)
        }) {
            Some(subscription) => subscription.update(frame),
            None => Err(SessionError::NoSubscription),
        }
    }

    fn update_sessions(&mut self, timestamp: Timestamp) {
        for sub in self.subscriptions {
            for session in sub.sessions.values_mut() {
                if sub.timestamp_expired(timestamp, session.timestamp) {
                    *session = Session::new(session.transfer_id);
                }
            }
        }
    }
}
