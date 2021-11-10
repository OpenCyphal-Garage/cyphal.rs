#![no_main]
#![no_std]
// to use the global allocator
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(bench_black_box)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(feature = "logging")]
mod logging;
#[cfg(feature = "logging")]
use defmt::info;

mod allocator;
mod clock;
mod util;
mod watch;

mod stack_analysis;

extern crate alloc;

use alloc::vec::Vec;
use arrayvec::ArrayVec;

#[cfg(not(feature = "logging"))]
use panic_halt as _;
use streaming_iterator::StreamingIterator;

use core::{
    alloc::Layout,
    mem::MaybeUninit,
    num::{NonZeroU16, NonZeroU8},
};

use allocator::MyAllocator;
use clock::StmClock;

use cortex_m::{self as _, peripheral::DWT};
use cortex_m_rt::{entry, pre_init};

use embedded_time::{
    duration::{Microseconds, Milliseconds, Nanoseconds},
    Clock,
};
use hal::{
    delay::SYSTDelayExt,
    fdcan::{
        config::{ClockDivider, NominalBitTiming},
        filter::{StandardFilter, StandardFilterSlot},
        frame::{self, TxFrameHeader},
        id::ExtendedId,
        id::Id,
        FdCan, NormalOperationMode,
    },
    gpio::{GpioExt, Speed},
    nb::block,
    prelude::*,
    rcc::{Config, PLLSrc, PllConfig, Rcc, RccExt, SysClockSrc},
    stm32::{Peripherals, FDCAN1},
    timer::MonoTimer,
};
use stm32g4xx_hal as hal;

use uavcan::{
    session::HeapSessionManager,
    transfer::Transfer,
    transport::can::{
        Can, CanFrame, CanIter, CanMessageId, CanMetadata, FakePayloadIter, TailByte,
    },
    types::{PortId, TransferId},
    Node, Priority, Subscription, TransferKind,
};

use util::insert_u8_array_in_u32_array;

use watch::Elapsed;

// #[pre_init]
// unsafe fn before_main() {
//     stack_analysis::fill_stack()
// }

static mut POOL: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::INIT;

#[no_mangle]
fn measure_receive(
    clock: &StmClock,
    watch: &mut watch::TraceWatch,
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    can: &mut FdCan<FDCAN1, NormalOperationMode>,
    port_id: PortId,
    transfer_id: &mut u8,
) {
    let mut frame_count = 0;
    let message_id = CanMessageId::new(uavcan::Priority::Immediate, port_id, Some(1));
    let payload_iter = FakePayloadIter::<8>::multi_frame(3, *transfer_id);
    for payload in payload_iter {
        let payload = arrayvec::ArrayVec::from_iter(payload);
        let frame = CanFrame {
            id: message_id,
            payload,
            timestamp: clock.try_now().unwrap(),
        };
        watch.start();
        if let Some(frame) = node
            .try_receive_frame(core::hint::black_box(frame))
            .unwrap()
        {
            core::hint::black_box(frame);
        }
        watch.stop();
        frame_count += 1;
    }

    let nanos = watch.get_elapsed().unwrap().0;
    info!("elapsed: {} nanos for {} frames", nanos, frame_count);
    watch.reset();
    *transfer_id = transfer_id.wrapping_add(1);
}

#[no_mangle]
fn measure_send(
    clock: &StmClock,
    watch: &mut watch::TraceWatch,
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    can: &mut FdCan<FDCAN1, NormalOperationMode>,
    port_id: PortId,
    transfer_id: &mut u8,
) {
    const PAYLOADS: [usize; 9] = [7, 12, 19, 26, 33, 40, 47, 54, 110];
    const PAYLOAD_LEN: usize = PAYLOADS[6];
    let data = heapless::Vec::<u8, PAYLOAD_LEN>::from_iter(
        core::iter::from_fn(|| {
            static mut COUNT: u8 = 0;
            unsafe {
                COUNT += 1;
                Some(COUNT)
            }
        })
        .take(PAYLOAD_LEN),
    );

    let transfer = Transfer {
        timestamp: clock.try_now().unwrap(),
        priority: Priority::Nominal,
        transfer_kind: TransferKind::Message,
        port_id: 100,
        remote_node_id: None,
        transfer_id: *transfer_id,
        payload: &data,
    };

    *transfer_id = transfer_id.wrapping_add(1);

    publish(watch, node, transfer, can);
}

fn publish(
    watch: &mut watch::TraceWatch,
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    transfer: Transfer<StmClock>,
    can: &mut FdCan<FDCAN1, NormalOperationMode>,
) {
    let mut frame_count = 0;
    watch.start();
    let mut frame_iter = node.transmit(&transfer).unwrap();
    watch.stop();
    loop {
        watch.start();
        let frame = frame_iter.next();
        watch.stop();
        if let Some(frame) = frame {
            // transmit_fdcan(frame, can);

            frame_count += 1;
            core::hint::black_box(frame);
        } else {
            break;
        }
    }

    let nanos = watch.get_elapsed().unwrap().0;
    info!("elapsed: {} nanos for {} frames", nanos, frame_count);
    watch.reset();
}

// #[no_mangle]
// fn test(clock: &StmClock) {
//     let now = clock.try_now().unwrap();
//     let start = DWT::get_cycle_count();
//     loop {
//         if clock.elapsed_millis(&now).0 >= 10 {
//             break;
//         }
//     }
//     let stop = DWT::get_cycle_count();

//     info!("took {}", (stop - start) * 1000 / 170);
// }

#[entry]
fn main() -> ! {
    // init heap
    let cursor = unsafe { POOL.as_mut_ptr() } as *mut u8;
    let size = 1024;
    unsafe { ALLOCATOR.init(cursor, size) };

    // define peripherals of the board
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");
    let rcc = dp.RCC.constrain();
    let mut rcc = config_rcc(rcc);

    let gpioa = dp.GPIOA.split(&mut rcc);

    let mut led = gpioa.pa5.into_push_pull_output();
    let mut delay_syst = cp.SYST.delay(&rcc.clocks);

    // init can
    let mut can = {
        let rx = gpioa.pa11.into_alternate().set_speed(Speed::VeryHigh);
        let tx = gpioa.pa12.into_alternate().set_speed(Speed::VeryHigh);

        let can = FdCan::new_with_clock_source(
            dp.FDCAN1,
            tx,
            rx,
            &rcc,
            hal::fdcan::FdCanClockSource::PCLK,
        );

        let mut can = can.into_config_mode();
        can.set_protocol_exception_handling(false);
        can.set_clock_divider(ClockDivider::_2);
        can.set_frame_transmit(hal::fdcan::config::FrameTransmissionConfig::AllowFdCan);

        let btr = NominalBitTiming {
            prescaler: NonZeroU16::new(5).unwrap(),
            seg1: NonZeroU8::new(14).unwrap(),
            seg2: NonZeroU8::new(2).unwrap(),
            sync_jump_width: NonZeroU8::new(1).unwrap(),
        };

        can.set_nominal_bit_timing(btr);

        can.set_standard_filter(
            StandardFilterSlot::_0,
            StandardFilter::accept_all_into_fifo0(),
        );
        // can.into_external_loopback()
        can.into_normal()
    };

    let measure_clock = watch::StmClock::new(cp.DWT, cp.DCB);
    let mut watch = watch::TraceWatch::new(&measure_clock);
    // init clock
    let clock = StmClock::new(dp.TIM7, &rcc.clocks);

    let port_id: PortId = 7168;

    let mut session_manager = HeapSessionManager::<CanMetadata, Milliseconds, StmClock>::new();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            port_id, // TODO check
            12 + 14 * 7,
            embedded_time::duration::Milliseconds(10000),
        ))
        .unwrap();

    let mut node = Node::<_, Can, StmClock>::new(Some(100), session_manager);

    let mut transfer_id = 0u8;
    let mut last_published = clock.try_now().unwrap();

    loop {
        if clock.elapsed_millis(&last_published).0 >= 1_000 {
            measure_send(
                &clock,
                &mut watch,
                &mut node,
                &mut can,
                port_id,
                &mut transfer_id,
            );
            last_published = clock.try_now().unwrap();
        }

        // if clock.elapsed_millis(&last_published).0 >= 1_000 {
        //     measure_receive(
        //         &clock,
        //         &mut watch,
        //         &mut node,
        //         &mut can,
        //         port_id,
        //         &mut transfer_id,
        //     );
        //     last_published = clock.try_now().unwrap();
        // }
    }
}

fn transmit_fdcan(frame: CanFrame<StmClock>, can: &mut FdCan<FDCAN1, NormalOperationMode>) {
    let header = TxFrameHeader {
        bit_rate_switching: false,
        frame_format: stm32g4xx_hal::fdcan::frame::FrameFormat::Standard,
        id: Id::Extended(ExtendedId::new(frame.id).unwrap()),
        len: frame.payload.len() as u8,
        marker: None,
    };
    block!(can.transmit(header, &mut |b| {
        insert_u8_array_in_u32_array(&frame.payload, b)
    },))
    .unwrap();
}

fn config_rcc(rcc: Rcc) -> Rcc {
    rcc.freeze(
        Config::new(SysClockSrc::PLL)
            .pll_cfg(PllConfig {
                mux: PLLSrc::HSI,
                m: 4,
                n: 85,
                r: 2,
                q: Some(2),
                p: Some(2),
            })
            .ahb_psc(hal::rcc::Prescaler::NotDivided)
            .apb_psc(hal::rcc::Prescaler::NotDivided),
    )
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
