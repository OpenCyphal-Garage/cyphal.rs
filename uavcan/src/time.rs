use core::{
    hash::Hash,
    ops::{Add, AddAssign},
};

use embedded_time::{duration::Duration, fixed_point::FixedPoint, Clock, ConversionError, TimeInt};

#[derive(Debug)]
pub struct TestClock<TickType: TimeInt + AddAssign>(TickType);

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
        self.0 += duration.to_generic(Self::SCALING_FACTOR)?.integer();
        Ok(())
    }
}

impl TestClock<u32> {
    pub fn new() -> Self {
        Self(0)
    }
}

impl<TickType: TimeInt + Hash + Add + AddAssign> Clock for TestClock<TickType> {
    type T = TickType;

    const SCALING_FACTOR: embedded_time::rate::Fraction = embedded_time::rate::Fraction::new(1, 1);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        Ok(embedded_time::Instant::new(self.0))
    }
}
