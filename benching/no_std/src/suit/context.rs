use uavcan::{session::SessionManager, transport::Transport, Node};

pub trait NeedsNode {
    type Clock: embedded_time::Clock + 'static + Clone;
    type SessionType: SessionManager<Self::Clock>;
    type TransportType: Transport<Self::Clock>;
    fn node_as_ref(&self) -> &Node<Self::SessionType, Self::TransportType, Self::Clock>;
    fn node_as_mut(&mut self) -> &mut Node<Self::SessionType, Self::TransportType, Self::Clock>;
}

pub trait NeedsClock: NeedsNode {
    fn clock_as_ref(&self) -> &Self::Clock;
    fn clock_as_mut(&mut self) -> &mut Self::Clock;
}
