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
    delay::{Delay, SYSTDelayExt},
    fdcan::{
        config::{ClockDivider, NominalBitTiming},
        filter::{StandardFilter, StandardFilterSlot},
        frame::{self, TxFrameHeader},
        id::ExtendedId,
        id::Id,
        FdCan, NormalOperationMode,
    },
    gpio::{gpioa::PA5, GpioExt, Output, PushPull, Speed},
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

use watch::{Elapsed, TraceWatch};

static mut POOL: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::INIT;

const RECEIVE_PORT_ID: u16 = 1001;
const SEND_PORT_ID: u16 = 1000;

const PAYLOADS: [usize; 9] = [7, 12, 19, 26, 33, 40, 47, 54, 110];
const PAYLOAD_LEN: usize = PAYLOADS[7];

fn logic(
    mut can: FdCan<FDCAN1, NormalOperationMode>,
    node: Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    clock: StmClock,
    mut led: PA5<Output<PushPull>>,
    mut delay: Delay,
    mut watch: TraceWatch,
) -> ! {
    let (tx, mut rx) = node.split();
    let mut transfer_id: TransferId = 0;
    let mut last_published = clock.try_now().unwrap();
    loop {
        watch.reset();
        if clock.elapsed_millis(&last_published).0 >= 1_000 {
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

            watch.start();
            let transfer = Transfer {
                timestamp: clock.try_now().unwrap(),
                priority: Priority::Nominal,
                transfer_kind: TransferKind::Message,
                port_id: SEND_PORT_ID,
                remote_node_id: None,
                transfer_id,
                payload: &data,
            };

            let mut frame_iter = tx.transmit(&transfer).unwrap();
            while let Some(frame) = frame_iter.next() {
                delay.delay_us(100);
                transmit_fdcan(frame, &mut can);
            }

            // loop {
            //     let frame = frame_iter.next();
            //     if let Some(frame) = frame {
            //         watch.stop();
            //         transmit_fdcan(frame, &mut can);
            //         watch.start();
            //     } else {
            //         watch.stop();
            //         break;
            //     }
            // }

            let _ = block!(can.receive0(&mut |frame, payload| {
                // watch.start();
                let mut payload_ptr = payload.as_ptr() as *const u8;
                let ptr_end = unsafe { payload_ptr.add((frame.len + 1) as usize) };
                let payload_iter = core::iter::from_fn(|| {
                    if payload_ptr == ptr_end {
                        None
                    } else {
                        let byte = unsafe { *payload_ptr };
                        payload_ptr = unsafe { payload_ptr.add(1) };
                        Some(byte)
                    }
                });
                let uavcan_can_frame = CanFrame {
                    id: match frame.id {
                        Id::Extended(id) => id.as_raw(),
                        _ => panic!("support only extended can ID"),
                    },
                    payload: ArrayVec::from_iter(payload_iter),
                    timestamp: clock.try_now().unwrap(),
                };
                if let Some(transfer) = rx.try_receive_frame(uavcan_can_frame).unwrap() {
                    let _ = core::hint::black_box(transfer);
                    watch.stop();

                    info!("took {} nanos", watch.get_elapsed().unwrap().0);
                }
            }));

            transfer_id = transfer_id.wrapping_add(1);
            last_published = clock.try_now().unwrap();
        }
    }
}

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
    let can = {
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
    let watch = watch::TraceWatch::new(&measure_clock);
    // init clock
    let clock = StmClock::new(dp.TIM7, &rcc.clocks);

    let mut session_manager = HeapSessionManager::<CanMetadata, Milliseconds, StmClock>::new();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            RECEIVE_PORT_ID, // TODO check
            PAYLOAD_LEN,
            embedded_time::duration::Milliseconds(10000),
        ))
        .unwrap();

    let node = Node::<_, Can, StmClock>::new(Some(100), session_manager);

    logic(can, node, clock, led, delay_syst, watch)
}

fn transmit_fdcan(frame: &CanFrame<StmClock>, can: &mut FdCan<FDCAN1, NormalOperationMode>) {
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
