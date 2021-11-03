use core::{
    cell::RefCell,
    ops::DerefMut,
    sync::atomic::{AtomicU32, Ordering},
};
use cortex_m::interrupt::Mutex;
use embedded_time::Clock;
use stm32g4xx_hal::{
    prelude::*,
    rcc::Clocks,
    stm32::TIM7,
    stm32::{interrupt, Interrupt},
    timer::{CountDownTimer, Event, Timer},
};

static mut CLOCK_COUNTER: AtomicU32 = AtomicU32::new(0);
static TIMER_TIM7: Mutex<RefCell<Option<CountDownTimer<TIM7>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM7() {
    unsafe {
        CLOCK_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    // clear interrupt flag
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut tim) = TIMER_TIM7.borrow(cs).borrow_mut().deref_mut() {
            tim.clear_interrupt(Event::TimeOut);
        }
    })
}

/// A clock for the stm32g4 with timer tim7.
///
/// Resolution of one millisecond.
#[derive(Clone)]
pub struct MonotonicClock;

impl MonotonicClock
where
    Self: Clock,
{
    pub fn new(tim7: TIM7, clocks: &Clocks) -> Self {
        // config tim7 with a frequency of 1000 Hz.
        let timer = Timer::new(tim7, clocks);
        let mut timer = timer.start_count_down(1000.hz());
        timer.listen(Event::TimeOut);

        cortex_m::interrupt::free(|cs| TIMER_TIM7.borrow(cs).replace(Some(timer)));

        // enable interrupt for tim7
        unsafe { cortex_m::peripheral::NVIC::unmask(Interrupt::TIM7) };

        Self {}
    }
}

impl Clock for MonotonicClock {
    type T = u32;

    /// Scaling factor of the clock for 1000 Hz.
    const SCALING_FACTOR: embedded_time::rate::Fraction =
        embedded_time::rate::Fraction::new(1, 1000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        Ok(embedded_time::Instant::new(unsafe {
            CLOCK_COUNTER.load(Ordering::Relaxed)
        }))
    }
}
