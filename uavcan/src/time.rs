pub type Timestamp<C> = embedded_time::Instant<C>;
pub type Duration = embedded_time::duration::Milliseconds;

#[cfg(test)]
pub use test_clock::TestClock;

#[cfg(feature = "std")]
pub use std_clock::StdClock;

#[cfg(test)]
mod test_clock {
    use core::{
        cell::RefCell,
        fmt::Debug,
        hash::Hash,
        ops::{Add, AddAssign, Deref},
    };

    use alloc::rc::Rc;

    use embedded_time::{
        duration::{Duration, Microseconds, Milliseconds},
        fixed_point::FixedPoint,
        Clock, ConversionError, TimeInt,
    };

    /// A Clock for test cases
    ///
    /// Contains a intern tick counter and a fraction which is set to 1 / 1,000,000 (secs) = resolution of one microsecond.
    /// Clock wraps if intern type overflows! (intern type = u32 -> ~71min)
    #[derive(Debug, Clone)]
    pub struct TestClock<TickType = u32>(Rc<RefCell<TickType>>)
    where
        TickType: TimeInt;

    impl<TickType: TimeInt> TestClock<TickType>
    where
        Self: Clock,
    {
        /// Add raw ticks to the clock.
        ///
        /// In particular 1 tick = 1 microsecond.
        pub fn add_ticks(&mut self, ticks: TickType) {
            self.0.deref().borrow_mut().wrapping_add(&ticks);
        }

        /// Add a duration to the clock.
        ///
        /// # Example
        /// ```
        /// let clock = TestClock::default();
        /// clock.add_duration(&Milliseconds::new(1243u32));
        /// ```
        pub fn add_duration<D: Duration + FixedPoint>(
            &mut self,
            duration: &D,
        ) -> Result<(), ConversionError>
        where
            TickType: From<<D as FixedPoint>::T>,
        {
            let mut tick_count = self.0.deref().borrow_mut();
            *tick_count =
                tick_count.wrapping_add(&duration.to_generic(Self::SCALING_FACTOR)?.integer());
            Ok(())
        }

        /// Get a clone of the clock which points to the same tick count.
        // like alloc::borrow::ToOwned but don't want to use alloc crate by now
        pub fn to_owned(&self) -> Self {
            self.clone()
        }
    }

    impl Default for TestClock<u32> {
        fn default() -> Self {
            Self(Rc::new(RefCell::new(u32::default())))
        }
    }

    impl<TickType: TimeInt + Hash + Add + AddAssign> Clock for TestClock<TickType> {
        type T = TickType;

        const SCALING_FACTOR: embedded_time::rate::Fraction =
            embedded_time::rate::Fraction::new(1, 1_000_000);

        fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
            Ok(embedded_time::Instant::new(*self.0.borrow()))
        }
    }

    #[test]
    fn test_fraction_as_1_tick_1_micros() {
        let mut clock = TestClock::default();
        clock.add_ticks(1);

        assert_eq!(
            clock.try_now().unwrap().duration_since_epoch(),
            Microseconds::new(1u32)
                .to_generic::<u32>(TestClock::<u32>::SCALING_FACTOR)
                .unwrap(),
            "1 tick does not equal to 1 microsecond"
        )
    }

    #[test]
    fn test_add_duration() {
        let mut clock = TestClock::default();
        let millis = Milliseconds::new(1243u32);
        assert!(
            clock.add_duration(&millis).is_ok(),
            "error on incrementing clock"
        );
        assert_eq!(
            clock.try_now().unwrap().duration_since_epoch(),
            millis
                .to_generic::<u32>(TestClock::<u32>::SCALING_FACTOR)
                .unwrap(),
            "clock not at same time"
        )
    }

    #[test]
    fn test_increment_clock_with_clones() {
        let mut clock = TestClock::default();
        let clock_clone = clock.to_owned();

        let millis = Milliseconds::new(1243u32);
        assert!(
            clock.add_duration(&millis).is_ok(),
            "error on incrementing clock"
        );
        assert!(
            clock.try_now().unwrap().duration_since_epoch().eq(&millis
                .to_generic::<u32>(TestClock::<u32>::SCALING_FACTOR)
                .unwrap()),
            "clock not at same time"
        );
        assert_eq!(
            clock.try_now().unwrap(),
            clock_clone.try_now().unwrap(),
            "clocks not at same time"
        )
    }
}

#[cfg(feature = "std")]
mod std_clock {
    use core::{cell::RefCell, convert::TryInto};

    use std::rc::Rc;

    #[cfg(test)]
    use mock_instant::Instant;
    #[cfg(not(test))]
    use std::time::Instant;

    /// A clock to use in std environments.
    ///
    /// Safes the actual time instance at creation. Returns for every call `to StdClock::try_now()`
    /// the relative time to creation time.
    /// This design is used, because it should only give relative time points and not exact world
    /// time points. In addition, to set the representing type to `u64` and to give a microsecond
    /// resolution, the clock can run longer without overflow.
    ///
    /// # Example
    /// ```ignore
    /// let clock = StdClock::new();
    /// assert!(
    ///    clock.try_now().unwrap().duration_since_epoch() >
    ///    Microseconds::new(0u32)
    ///        .to_generic(StdClock::SCALING_FACTOR)
    ///        .unwrap()
    /// );
    /// ```
    #[derive(Clone, Debug)]
    pub struct StdClock(Rc<RefCell<Instant>>);

    impl StdClock {
        /// Creates a new `StdClock` with an offset of the actual `std::time::Instant`.
        ///
        /// The clock is zeroed at this time point.  
        pub fn new() -> Self {
            Self(Rc::new(RefCell::new(Instant::now())))
        }
    }

    impl embedded_time::Clock for StdClock {
        type T = u64;

        const SCALING_FACTOR: embedded_time::rate::Fraction =
            embedded_time::rate::Fraction::new(1, 1_000_000);

        fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
            let now = Instant::now() - *self.0.borrow();
            let time_passed: u64 = now
                .as_micros()
                .try_into()
                .map_err(|_| embedded_time::clock::Error::Unspecified)?;

            Ok(embedded_time::Instant::new(time_passed))
        }
    }

    #[cfg(test)]
    mod test {
        use embedded_time::{
            duration::{Duration, Microseconds},
            Clock,
        };
        use mock_instant::MockClock;

        use super::StdClock;

        #[test]
        fn test_clock() {
            // set instant to random start
            MockClock::advance(core::time::Duration::from_micros(23425));

            let clock = StdClock::new();
            let advance_micros = 1000u64;

            assert_eq!(
                clock.try_now().unwrap().duration_since_epoch(),
                Microseconds::new(0u32)
                    .to_generic(StdClock::SCALING_FACTOR)
                    .unwrap(),
                "clock is not zero at beginning"
            );

            MockClock::advance(core::time::Duration::from_micros(advance_micros));

            assert_eq!(
                clock.try_now().unwrap().duration_since_epoch(),
                Microseconds::new(advance_micros)
                    .to_generic(StdClock::SCALING_FACTOR)
                    .unwrap(),
                "clock gives not correct time"
            );
        }
    }
}
