use core::marker::PhantomData;

use crate::session::*;
use crate::types::NodeId;

/// Internal session object.
struct Session<T, C>
where
    T: crate::transport::SessionMetadata<C>,
    C: embedded_time::Clock,
{
    // Timestamp of first frame
    pub timestamp: Option<Timestamp<C>>,
    pub payload: heapless::pool::Node<u8>,
    pub transfer_id: TransferId,

    pub md: T,

    _clock: PhantomData<C>,
}

impl<T, C> Session<T, C>
where
    T: crate::transport::SessionMetadata<C>,
    C: embedded_time::Clock,
{
    pub fn new(transfer_id: TransferId, memory: heapless::pool::Node<u8>) -> Self {
        Self {
            timestamp: None,
            payload: memory,
            transfer_id,
            md: T::new(),

            _clock: PhantomData,
        }
    }
}

/// Internal subscription object. Contains hash map of sessions.
struct Subscription<'a, T, C, D, const N: usize>
where
    T: crate::transport::SessionMetadata<C>,
    C: embedded_time::Clock,
    D: embedded_time::duration::Duration + FixedPoint,
{
    sub: crate::Subscription<D>,
    sessions: heapless::FnvIndexMap<NodeId, heapless::pool::Node<T>, N>,

    mem_pool: &'a heapless::pool::Pool<u8>,
    _clock: PhantomData<C>,
}

impl<'a, T, C, D, const N: usize> Subscription<'a, T, C, D, N>
where
    T: crate::transport::SessionMetadata<C>,
    D: embedded_time::duration::Duration + FixedPoint,
    C: embedded_time::Clock,
    <C as embedded_time::Clock>::T: From<<D as FixedPoint>::T>,
{
    pub fn new(sub: crate::Subscription<D>, mem_pool: &'a heapless::pool::Pool<u8>) -> Self {
        Self {
            sub,
            sessions: heapless::FnvIndexMap::new(),

            mem_pool,
            _clock: PhantomData,
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
                .insert(session, Session::new(frame.transfer_id, self.mem_pool.));
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
        frame: InternalRxFrame<StdClock>,
    ) -> Result<Option<Transfer<StdClock>>, SessionError> {
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
