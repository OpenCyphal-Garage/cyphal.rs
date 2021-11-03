use core::ops::Add;

use embedded_time::{duration::Nanoseconds, TimeInt};

use super::{Elapsed, Watch};

pub struct Samples<'d, T: TimeInt> {
    pub name: &'static str,
    pub data: &'d [Nanoseconds<T>],
}

pub struct Runner<C: embedded_time::Clock, const ROUND_SIZE: usize> {
    timer: C,
}

impl<'a, C: embedded_time::Clock, const ROUND_SIZE: usize> Runner<C, ROUND_SIZE> {
    pub fn new(timer: C) -> Self {
        Self { timer }
    }

    pub fn run_with_watch<Target, Ret>(
        &mut self,
        target: &mut Target,
        samples: &mut [Nanoseconds<C::T>],
    ) where
        Target: FnMut(&mut Watch<C>) -> Ret,
    {
        for s in samples {
            // TODO do sth. with None return
            *s = self.run_round_with_watch(target).unwrap();
        }
    }

    fn run_round_with_watch<Target, Ret>(
        &mut self,
        target: &mut Target,
    ) -> Option<Nanoseconds<C::T>>
    where
        Target: FnMut(&mut Watch<C>) -> Ret,
    {
        let mut elapsed: Option<Nanoseconds<C::T>> = None;
        let mut watch = Watch::new(&self.timer);

        for _ in 0..ROUND_SIZE {
            core::hint::black_box(target(&mut watch));
            let now_elapsed = watch.get_elapsed().expect("watch never started");
            match elapsed.as_mut() {
                Some(elapsed) => *elapsed = elapsed.add(now_elapsed),
                None => {
                    let _ = elapsed.insert(now_elapsed);
                }
            }
            watch.reset();
        }

        elapsed.map(|i| i / C::T::from(ROUND_SIZE as u32))
    }

    pub fn run<Target, Ret>(&mut self, target: &mut Target, samples: &mut [Nanoseconds<C::T>])
    where
        Target: FnMut() -> Ret,
    {
        for s in samples {
            *s = self.run_round(target);
        }
    }

    fn run_round<Target, Ret>(&mut self, target: &mut Target) -> Nanoseconds<C::T>
    where
        Target: FnMut() -> Ret,
    {
        let start = self.timer.try_now().unwrap();

        for _ in 0..ROUND_SIZE {
            core::hint::black_box(target());
        }

        let elapsed = self.timer.elapsed_nanos(&start);

        elapsed / C::T::from(ROUND_SIZE as u32)
    }
}
