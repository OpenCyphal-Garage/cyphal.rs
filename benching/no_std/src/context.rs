use embedded_time::duration::Milliseconds;
use uavcan::{
    session::HeapSessionManager,
    transport::can::{Can, CanMetadata},
    Node,
};

use crate::{
    benching::bencher::RenewableContext,
    clock::MonotonicClock,
    suit::context::{NeedsClock, NeedsNode},
};

pub struct Context {
    node: Node<HeapSessionManager<CanMetadata, Milliseconds, MonotonicClock>, Can, MonotonicClock>,
    clock: MonotonicClock,
}

impl Context {
    pub fn new(mono_clock: MonotonicClock) -> Self {
        Self {
            node: Node::new(None, HeapSessionManager::new()),
            clock: mono_clock,
        }
    }
}

impl RenewableContext for Context {
    fn reset(&mut self) {
        self.node = Node::new(None, HeapSessionManager::new());
    }
}

impl NeedsNode for Context {
    type Clock = MonotonicClock;
    type SessionType = HeapSessionManager<CanMetadata, Milliseconds, MonotonicClock>;
    type TransportType = Can;

    fn node_as_ref(&self) -> &Node<Self::SessionType, Self::TransportType, Self::Clock> {
        &self.node
    }

    fn node_as_mut(&mut self) -> &mut Node<Self::SessionType, Self::TransportType, Self::Clock> {
        &mut self.node
    }
}

impl NeedsClock for Context {
    fn clock_as_ref(&self) -> &Self::Clock {
        &self.clock
    }

    fn clock_as_mut(&mut self) -> &mut Self::Clock {
        &mut self.clock
    }
}
