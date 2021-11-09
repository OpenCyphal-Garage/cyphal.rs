pub mod bencher;
mod report;
pub mod runner;
mod stats;
mod watch;

pub use report::report;
pub use stats::Summary;

use embedded_time::{
    duration::{Generic, Nanoseconds},
    Instant,
};
pub use watch::{RunningWatch, Watch};

pub trait BenchClock = embedded_time::Clock;

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
