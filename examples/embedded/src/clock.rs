use cortex_m::peripheral::{DCB, DWT};
use embedded_time::Clock;
use stm32g4xx_hal::rcc::Clocks;

/// A clock for the stm32
///
/// Uses the stm32 hal `HAL_GetTick`. Underlying u32 counter, will wrap after 50 days.
#[derive(Clone)]
pub struct StmClock;

impl StmClock
where
    Self: Clock,
{
    pub fn new(mut dwt: DWT, mut dcb: DCB, clocks: &Clocks) -> Self {
        dcb.enable_trace();
        dwt.enable_cycle_counter();

        // now the CYCCNT counter can't be stopped or reset
        drop(dwt);

        // assert_eq!(
        //     *(Self::SCALING_FACTOR.denominator()),
        //     clocks.ahb_clk.0,
        //     "clock ahb has not correct frequency"
        // );

        Self {}
    }
}

impl Clock for StmClock {
    type T = u32;

    const SCALING_FACTOR: embedded_time::rate::Fraction =
        embedded_time::rate::Fraction::new(1, 170_000_000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        Ok(embedded_time::Instant::new(DWT::get_cycle_count()))
    }
}
