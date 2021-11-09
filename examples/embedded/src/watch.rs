use core::ops::Add;

use cortex_m::peripheral::{DCB, DWT};
use embedded_time::{
    duration::{Generic, Nanoseconds},
    Instant,
};

pub type TraceWatch<'a> = Watch<'a, StmClock>;

pub struct StmClock;

impl StmClock {
    pub fn new(mut dwt: DWT, mut dcb: DCB) -> Self {
        init_timer(dwt, dcb);
        Self {}
    }
}

impl embedded_time::Clock for StmClock {
    type T = u32;

    const SCALING_FACTOR: embedded_time::rate::Fraction =
        embedded_time::rate::Fraction::new(1, 170_000_000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        Ok(embedded_time::Instant::new(DWT::get_cycle_count()))
    }
}

fn init_timer(mut dwt: DWT, mut dcb: DCB) {
    dcb.enable_trace();
    dwt.enable_cycle_counter();

    // now the CYCCNT counter can't be stopped or reset
    drop(dwt);
}

pub trait Elapsed<C: embedded_time::Clock> {
    fn elapsed(&self, since: &Instant<C>) -> Generic<C::T>;
    fn elapsed_nanos(&self, since: &Instant<C>) -> Nanoseconds<C::T>;
}

impl<C> Elapsed<C> for C
where
    C: embedded_time::Clock,
{
    fn elapsed(&self, since: &Instant<C>) -> Generic<C::T> {
        let now = self.try_now().unwrap();
        match now.checked_duration_until(since) {
            Some(elapsed) => elapsed,
            None => now.checked_duration_since(since).unwrap(),
        }
    }

    fn elapsed_nanos(&self, since: &Instant<C>) -> Nanoseconds<C::T> {
        Nanoseconds::try_from(self.elapsed(since)).unwrap()
    }
}

pub struct RunningWatch<'a, 'b, C: embedded_time::Clock> {
    watch: &'a mut Watch<'b, C>,
    aborted: bool,
}

impl<'a, 'b, C: embedded_time::Clock> RunningWatch<'a, 'b, C> {
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

impl<'b, C: embedded_time::Clock> Drop for RunningWatch<'_, 'b, C> {
    fn drop(&mut self) {
        if !self.aborted {
            self.watch.stop()
        }
    }
}

pub struct Watch<'a, C: embedded_time::Clock> {
    elapsed: Option<Nanoseconds<C::T>>,
    last_start: Option<Instant<C>>,
    clock: &'a C,
}

impl<'a, C: embedded_time::Clock> Watch<'a, C> {
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
    pub fn get_elapsed(&mut self) -> Option<Nanoseconds<C::T>> {
        if self.last_start.is_some() {
            self.stop();
        }
        self.elapsed
    }
}
