#![no_std]
#![no_main]
#![feature(total_cmp)]
#![feature(alloc_error_handler)]
#![feature(extend_one)]
#![feature(slice_pattern)]
#![feature(bench_black_box)]
#![feature(trait_alias)]
#![feature(optimize_attribute)]
#![allow(warnings)]

extern crate alloc;

mod allocator;
mod benching;
mod board;
mod boards;
mod clock;
mod context;
mod logging;
mod suit;

use allocator::MyAllocator;
use benching::report;
use board::Board;
use core::{alloc::Layout, mem::MaybeUninit};
use cortex_m_rt::entry;
use defmt::info;
use suit::{receive, send, Bencher};

type UsedBoard = boards::Stm32G4;

static mut POOL: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();
#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::INIT;

#[entry]
fn main() -> ! {
    // init heap
    let cursor = unsafe { POOL.as_mut_ptr() } as *mut u8;
    let size = 1024;
    unsafe { ALLOCATOR.init(cursor, size) };

    let mut board = UsedBoard::setup();

    let measure_clock = board.get_clock();

    let mut bencher = Bencher::new(measure_clock);

    // init clock
    let clock = board.get_monotonic_clock();

    info!("running benches ...\n");

    let mut context = context::Context::new(clock);

    report(&bencher.bench("bench_send", &mut send::bench_send::<_, _, 7>, &mut context));

    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 12>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 19>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 26>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 33>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 40>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 47>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, 54>,
        &mut context,
    ));

    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 1>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 2>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 3>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 4>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 5>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 6>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 7>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, 8>,
        &mut context,
    ));

    #[cfg(feature = "test")]
    {
        let mut context = suit::test::Context;
        report(&bencher.bench("test", &mut suit::test::bench_test::<_, 3>, &mut context));
        report(&bencher.bench("test", &mut suit::test::bench_test::<_, 4>, &mut context));
    }

    info!("finished");

    logging::exit();
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    cortex_m::asm::udf();
}
