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
    fn get_tick_interpreter(&self) -> TickInterpreter {
        TickInterpreter::new(self.get_clock_frequency())
    }
    fn get_monotonic_clock(&mut self) -> MonotonicClock;
}

pub struct TickInterpreter {
    frequency: embedded_time::rate::Hertz,
}

impl TickInterpreter {
    fn new(frequency: embedded_time::rate::Hertz) -> Self {
        Self { frequency }
    }

    pub fn as_nanos(&self, cycles: u64) -> f64 {
        cycles.saturating_mul(1_000_000_000u64) as f64 / self.frequency.0 as f64
    }

    pub fn as_micros(&self, cycles: u64) -> f64 {
        cycles.saturating_mul(1_000_000u64) as f64 / self.frequency.0 as f64
    }

    pub fn as_millis(&self, cycles: u64) -> f64 {
        cycles.saturating_mul(1_000) as f64 / self.frequency.0 as f64
    }

    pub fn as_seconds(&self, cycles: u64) -> f64 {
        cycles as f64 / self.frequency.0 as f64
    }
}
