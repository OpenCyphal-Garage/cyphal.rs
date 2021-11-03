pub mod receive;
pub mod send;
pub mod test;

const SAMPLE_SIZE: usize = 100;
const ROUND_SIZE: usize = 10;

pub type Bencher<C> = crate::benching::bencher::Bencher<C, SAMPLE_SIZE, ROUND_SIZE>;
