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
mod logging;
mod stats;
mod suit;

use alloc::format;
use allocator::MyAllocator;
use board::Board;
use embedded_time::{duration::Milliseconds, Clock, TimeInt};
use suit::{receive, send, Bencher};
use uavcan::{
    session::HeapSessionManager,
    transport::can::{Can, CanMetadata},
    Node,
};

use benching::{runner::Samples, Elapsed};

use core::{alloc::Layout, fmt::Debug, mem::MaybeUninit};

use cortex_m_rt::entry;

use defmt::{info, Format};

use crate::clock::MonotonicClock;

type UsedBoard = boards::Stm32G4;

static mut POOL: MaybeUninit<[u8; 10240]> = MaybeUninit::uninit();
#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::INIT;

#[entry]
fn main() -> ! {
    // init heap
    let cursor = unsafe { POOL.as_mut_ptr() } as *mut u8;
    let size = 10240;
    unsafe { ALLOCATOR.init(cursor, size) };

    let mut board = UsedBoard::setup();

    let measure_clock = board.get_clock();

    let mut bencher = Bencher::new(measure_clock);

    // init clock
    let clock = board.get_monotonic_clock();

    let session_manager = HeapSessionManager::<CanMetadata, Milliseconds, MonotonicClock>::new();
    let node = Node::<_, Can, MonotonicClock>::new(Some(100), session_manager);

    info!("running benches ...\n");

    // let needs = suit::send::run_single(node, clock, measure_clock);
    // info!("needs {}", needs.0);

    let mut context = suit::send::Context {
        node,
        clock: &clock,
    };

    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 7>,
        &mut context,
    ));

    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 12>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 19>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 26>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 33>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 40>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 47>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_send",
        &mut send::bench_send::<_, _, _, 54>,
        &mut context,
    ));

    let session_manager = HeapSessionManager::<CanMetadata, Milliseconds, MonotonicClock>::new();
    let node = Node::<_, Can, MonotonicClock>::new(Some(100), session_manager);

    let mut context = suit::receive::Context {
        node,
        clock: &clock,
    };

    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 1>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 2>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 3>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 4>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 5>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 6>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 7>,
        &mut context,
    ));
    report(&bencher.bench(
        "bench_receive",
        &mut receive::bench_receive::<_, _, _, 8>,
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

pub fn report<T: TimeInt + Into<u32>>(s: &Samples<T>) {
    // let mut total = 0f64;
    // for i in 0..s.data.len() {
    //     total += interpreter.as_micros(s.data[i]);
    // }
    // let n = s.data.len() as f64;
    // let avg = total / n;
    let data = s
        .data
        .iter()
        .map(|d| {
            let d: u32 = d.0.into();
            d as f64
        })
        .collect::<heapless::Vec<f64, 100>>();

    // info!("{:?}", data.as_slice());

    let summary = stats::Summary::new::<100>(&data.into_array().unwrap());

    let median = (summary.median) as usize;
    let deviation = (summary.max - summary.min) as usize;

    info!(
        "function: {}    {} ns/iter (+/- {})",
        s.name,
        fmt_thousands_sep(median, '_').as_str(),
        fmt_thousands_sep(deviation, '_').as_str()
    );
}

// Format a number with thousands separators
fn fmt_thousands_sep(mut n: usize, sep: char) -> alloc::string::String {
    let mut output = alloc::string::String::new();
    let mut trailing = false;
    for &pow in &[9, 6, 3, 0] {
        let base = 10_usize.pow(pow);
        if pow == 0 || trailing || n / base != 0 {
            if !trailing {
                output.extend(format!("{}", n / base).as_str().chars());
            } else {
                output.extend(format!("{:03}", n / base).as_str().chars());
            }
            if pow != 0 {
                output.push(sep);
            }
            trailing = true;
        }
        n %= base;
    }

    output
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    cortex_m::asm::udf();
}
