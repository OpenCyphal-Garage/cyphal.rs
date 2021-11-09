use embedded_time::duration::Nanoseconds;

use num_traits::Zero;

use super::{
    runner::{Runner, Samples},
    Watch,
};

pub trait RenewableContext {
    fn reset(&mut self);
}

pub struct Bencher<C: embedded_time::Clock, const SAMPLE_AMOUNT: usize, const ROUND_SIZE: usize> {
    data: [Nanoseconds<C::T>; SAMPLE_AMOUNT],
    runner: Runner<C, ROUND_SIZE>,
}

impl<C: embedded_time::Clock, const SAMPLE_AMOUNT: usize, const ROUND_SIZE: usize>
    Bencher<C, SAMPLE_AMOUNT, ROUND_SIZE>
where
    <C as embedded_time::Clock>::T: Default,
{
    pub fn new(clock: C) -> Self {
        Bencher {
            data: [Nanoseconds::new(C::T::zero()); SAMPLE_AMOUNT],
            runner: Runner::new(clock),
        }
    }
}

impl<C: embedded_time::Clock, const SAMPLE_AMOUNT: usize, const ROUND_SIZE: usize>
    Bencher<C, SAMPLE_AMOUNT, ROUND_SIZE>
{
    pub fn run<Target, Ret>(&mut self, mut target: Target)
    where
        Target: FnMut() -> Ret,
    {
        self.runner.run(&mut target, &mut self.data);
    }

    pub fn run_with_watch<Target, Ret>(&mut self, mut target: Target)
    where
        Target: FnMut(&mut Watch<C>) -> Ret,
    {
        self.runner.run_with_watch(&mut target, &mut self.data);
    }

    pub fn bench<'s, Target, Context>(
        &'s mut self,
        name: &'static str,
        target: &mut Target,
        context: &mut Context,
    ) -> Samples<'s, C::T>
    where
        Target: FnMut(&mut Bencher<C, SAMPLE_AMOUNT, ROUND_SIZE>, &mut Context),
    {
        target(self, context);

        Samples {
            name,
            data: &self.data,
        }
    }
}
