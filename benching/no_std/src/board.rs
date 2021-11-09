use num_traits::WrappingSub;

use crate::clock::MonotonicClock;

pub trait Clock {
    type Precise: WrappingSub + Default;
    fn now(&self) -> Self::Precise;
    fn elapsed(&self, last: Self::Precise) -> Self::Precise;
}

pub trait Board {
    type Clock: Clock;
    fn setup() -> Self;
    fn get_clock(&self) -> Self::Clock;
    fn get_clock_frequency(&self) -> embedded_time::rate::Hertz;
    fn get_monotonic_clock(&mut self) -> MonotonicClock;
}
