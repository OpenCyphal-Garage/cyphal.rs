#![no_std]
#![no_main]

mod board;
mod boards;
mod logging;
mod suit;

use board::{Board, Clock};
use suit::send;

use core::mem::MaybeUninit;

use cortex_m_rt::entry;

use defmt::info;
use liar::no_std::runner::Sample;

const SAMPLE_SIZE: usize = 100;
const ROUND_SIZE: usize = 1_000;

type Bencher<'s> = liar::no_std::bencher::Bencher<'s, u64>;

type UsedBoard = boards::Stm32G4;

static mut BOARD: MaybeUninit<UsedBoard> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    unsafe { BOARD.write(UsedBoard::setup()) };

    let mut data = [0u64; SAMPLE_SIZE];
    let mut bencher = Bencher::new(&mut data, time, diff, ROUND_SIZE);
    let mut out = [0u64; SAMPLE_SIZE];

    report(&bencher.bench("bench_send", &mut send::bench_send, &mut out));

    loop {}
}

pub fn report(s: &Sample) {
    let interpreter = unsafe { BOARD.assume_init_ref().get_tick_interpreter() };
    let mut total = 0f64;
    for i in 0..s.data.len() {
        total += interpreter.as_micros(s.data[i]);
    }
    let n = s.data.len() as f64;
    let avg = total / n;

    info!("[{}]\t{}", s.name, avg);
}

pub fn time() -> u64 {
    unsafe { BOARD.assume_init_ref().get_clock() }.cycles()
}

pub fn diff(start: &u64, end: &u64) -> u64 {
    end - start
}
