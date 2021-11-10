use core::ops::Add;

use cortex_m::peripheral::{DCB, DWT};
use embedded_time::{
    duration::{Generic, Milliseconds, Nanoseconds},
    Instant,
};

pub type TraceWatch<'a> = Watch<'a, StmClock>;

pub struct StmClock;

impl StmClock {
    pub fn new(dwt: DWT, dcb: DCB) -> Self {
        init_timer(dwt, dcb);
        Self {}
    }
}

impl embedded_time::Clock for StmClock {
    type T = u32;

    const SCALING_FACTOR: embedded_time::rate::Fraction =
        embedded_time::rate::Fraction::new(1, 170_000_000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        let ticks = DWT::get_cycle_count();
        Ok(embedded_time::Instant::new(ticks))
    }
}

fn init_timer(mut dwt: DWT, mut dcb: DCB) {
    dcb.enable_trace();
    unsafe { dwt.cyccnt.write(0) };
    dwt.enable_cycle_counter();

    // now the CYCCNT counter can't be stopped or reset
    drop(dwt);
}

pub trait Elapsed<C: embedded_time::Clock> {
    fn elapsed(&self, since: &Instant<C>) -> Generic<C::T>;
    fn elapsed_nanos(&self, since: &Instant<C>) -> Nanoseconds<u64>;
    fn elapsed_millis(&self, since: &Instant<C>) -> Milliseconds<u64>;
}

impl<C> Elapsed<C> for C
where
    C: embedded_time::Clock,
    <C as embedded_time::Clock>::T: Into<u64>,
{
    fn elapsed(&self, since: &Instant<C>) -> Generic<C::T> {
        let now = self.try_now().unwrap();
        match now.checked_duration_until(since) {
            Some(elapsed) => elapsed,
            None => now.checked_duration_since(since).unwrap(),
        }
    }

    fn elapsed_nanos(&self, since: &Instant<C>) -> Nanoseconds<u64> {
        let elapsed = self.elapsed(since);
        Nanoseconds::try_from(elapsed).unwrap()
    }

    fn elapsed_millis(&self, since: &Instant<C>) -> Milliseconds<u64> {
        let elapsed = self.elapsed(since);
        Milliseconds::try_from(elapsed).unwrap()
    }
}

pub struct RunningWatch<'a, 'b, C: embedded_time::Clock>
where
    <C as embedded_time::Clock>::T: Into<u64>,
{
    watch: &'a mut Watch<'b, C>,
    aborted: bool,
}

impl<'a, 'b, C: embedded_time::Clock> RunningWatch<'a, 'b, C>
where
    <C as embedded_time::Clock>::T: Into<u64>,
{
    fn new(watch: &'a mut Watch<'b, C>) -> Self {
        Self {
            watch,
            aborted: false,
        }
    }

    pub fn abort(&mut self) {
        self.aborted = true;
    }
}

impl<'b, C: embedded_time::Clock> Drop for RunningWatch<'_, 'b, C>
where
    <C as embedded_time::Clock>::T: Into<u64>,
{
    fn drop(&mut self) {
        if !self.aborted {
            self.watch.stop()
        }
    }
}

pub struct Watch<'a, C: embedded_time::Clock>
where
    <C as embedded_time::Clock>::T: Into<u64>,
{
    elapsed: Option<Nanoseconds<u64>>,
    last_start: Option<Instant<C>>,
    clock: &'a C,
}

impl<'a, C: embedded_time::Clock> Watch<'a, C>
where
    <C as embedded_time::Clock>::T: Into<u64>,
{
    pub(crate) fn new(clock: &'a C) -> Self {
        Self {
            elapsed: None,
            last_start: None,
            clock,
        }
    }

    pub fn start(&mut self) {
        if self.last_start.is_some() {
            return;
        }

        let _ = self.last_start.insert(self.clock.try_now().unwrap());
    }

    pub fn start_scoped(&mut self) -> Option<RunningWatch<'_, 'a, C>> {
        if self.last_start.is_some() {
            return None;
        }

        let _ = self.last_start.insert(self.clock.try_now().unwrap());

        Some(RunningWatch::new(self))
    }

    pub fn stop(&mut self) {
        if self.last_start.is_none() {
            return;
        }
        let elapsed = self.clock.elapsed_nanos(&self.last_start.take().unwrap());
        match self.elapsed.as_mut() {
            Some(n) => *n = n.add(elapsed),
            None => {
                let _ = self.elapsed.insert(elapsed);
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        self.elapsed = None;
        self.last_start = None;
    }

    /// returns None if watch never started to run
    pub fn get_elapsed(&mut self) -> Option<Nanoseconds<u64>> {
        if self.last_start.is_some() {
            self.stop();
        }
        self.elapsed
    }
}
