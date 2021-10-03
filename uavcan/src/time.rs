#[cfg(test)]
pub use test_clock::TestClock;

#[cfg(feature = "std")]
pub use std_clock::StdClock;

#[cfg(test)]
mod test_clock {
    use core::{
        fmt::Debug,
        hash::Hash,
        ops::{Add, AddAssign},
    };

    use embedded_time::{
        duration::Duration, fixed_point::FixedPoint, Clock, ConversionError, TimeInt,
    };

    #[derive(Debug, Clone)]
    pub struct TestClock<TickType: TimeInt + AddAssign>(core::cell::RefCell<TickType>);

    impl<TickType: TimeInt + AddAssign> TestClock<TickType>
    where
        Self: Clock,
    {
        pub fn add_duration<D: Duration + FixedPoint>(
            &mut self,
            duration: D,
        ) -> Result<(), ConversionError>
        where
            TickType: From<<D as FixedPoint>::T>,
        {
            *self.0.borrow_mut() += duration.to_generic(Self::SCALING_FACTOR)?.integer();
            Ok(())
        }

        // like alloc::borrow::ToOwned but don't want to use alloc crate by now
        pub fn to_owned(&self) -> Self {
            self.clone()
        }
    }

    impl TestClock<u32> {
        pub fn new() -> Self {
            Self(core::cell::RefCell::new(0))
        }
    }

    impl<TickType: TimeInt + Hash + Add + AddAssign> Clock for TestClock<TickType> {
        type T = TickType;

        const SCALING_FACTOR: embedded_time::rate::Fraction =
            embedded_time::rate::Fraction::new(1, 1);

        fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
            Ok(embedded_time::Instant::new(*self.0.borrow()))
        }
    }
}

#[cfg(feature = "std")]
mod std_clock {

    #[derive(Clone, Debug)]
    pub struct StdClock;

    impl embedded_time::Clock for StdClock {
        type T = u64;

        const SCALING_FACTOR: embedded_time::rate::Fraction =
            embedded_time::rate::Fraction::new(1, 1);

        fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
            let now = std::time::Instant::now();
            let time_passed = now.elapsed().as_secs();

            Ok(embedded_time::Instant::new(time_passed))
        }
    }

    impl Default for StdClock {
        fn default() -> Self {
            Self {}
        }
    }
}
