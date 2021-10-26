#![no_main]
#![no_std]
// to use the global allocator
#![feature(alloc_error_handler)]
#![feature(asm)]

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

mod stack_analysis;
mod to_test;

extern crate alloc;

use alloc::vec::Vec;
use arrayvec::ArrayVec;

#[cfg(not(feature = "logging"))]
use panic_halt as _;

use core::{
    alloc::Layout,
    mem::MaybeUninit,
    num::{NonZeroU16, NonZeroU8},
};

use allocator::MyAllocator;
use clock::StmClock;

use cortex_m as _;
use cortex_m_rt::{entry, pre_init};

use embedded_time::{duration::Milliseconds, Clock};
use hal::{
    delay::SYSTDelayExt,
    fdcan::{
        config::{ClockDivider, NominalBitTiming},
        filter::{StandardFilter, StandardFilterSlot},
        frame::TxFrameHeader,
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
    transport::can::{Can, CanFrame, CanMessageId, CanMetadata, FakePayloadIter, TailByte},
    types::{PortId, TransferId},
    Node, Subscription, TransferKind,
};

use util::insert_u8_array_in_u32_array;

use crate::{stack_analysis::RAM_START, to_test::publish};

#[pre_init]
unsafe fn before_main() {
    stack_analysis::fill_stack()
}

static mut POOL: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::INIT;

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

    let measure_clock = MonoTimer::new(cp.DWT, cp.DCB, &rcc.clocks);

    // init clock
    let clock = StmClock::new(dp.TIM7, &rcc.clocks);

    let port_id: PortId = 7168;

    let mut session_manager = HeapSessionManager::<CanMetadata, Milliseconds, StmClock>::new();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            port_id, // TODO check
            12 + 6 * 7,
            embedded_time::duration::Milliseconds(500),
        ))
        .unwrap();

    let mut node = Node::<_, Can, StmClock>::new(Some(100), session_manager);

    let mut transfer_id = 0u8;
    let mut last_published = clock.try_now().unwrap();

    // loop {
    let now = clock.try_now().unwrap();
    // if now - last_published
    //     > embedded_time::duration::Generic::new(1000, StmClock::SCALING_FACTOR)
    // {
    //     // Publish string
    //     let hello = "Hello!";
    //     let mut str = heapless::Vec::<u8, 6>::new();
    //     str.extend_from_slice(hello.as_bytes()).unwrap();

    //     let transfer = Transfer {
    //         timestamp: clock.try_now().unwrap(),
    //         priority: Priority::Nominal,
    //         transfer_kind: TransferKind::Message,
    //         port_id: 100,
    //         remote_node_id: None,
    //         transfer_id,
    //         payload: &str,
    //     };

    //     // unchecked_add is unstable :(
    //     // unsafe { transfer_id.unchecked_add(1); }
    //     transfer_id += 1;

    //     publish(&measure_clock, &mut node, transfer, &mut can);

    //     last_published = clock.try_now().unwrap();

    //     led.toggle().unwrap();
    //     delay_syst.delay(1000.ms());
    //     led.toggle().unwrap();
    // }

    // if now - last_published > embedded_time::duration::Generic::new(1000, StmClock::SCALING_FACTOR)
    // {
    let mut general_elapsed = 0;
    let mut success = false;
    let message_id = CanMessageId::new(uavcan::Priority::Immediate, port_id, Some(1));
    let payload_iter = FakePayloadIter::multi_frame(8, transfer_id);
    for payload in payload_iter {
        let payload = ArrayVec::from(payload);
        let frame = CanFrame {
            id: message_id,
            payload,
            timestamp: clock.try_now().unwrap(),
        };
        let start = measure_clock.now();
        if let Some(_) = node.try_receive_frame(frame).unwrap() {
            success = true;
        }
        let elapsed = start.elapsed();
        general_elapsed += elapsed;
    }

    let micros: u32 = measure_clock.frequency().duration(general_elapsed).0;
    // info!("elapsed: {} micros", micros);
    // info!("success: {}", success);

    transfer_id = transfer_id.wrapping_add(1);

    last_published = now;
    // }
    // }

    // unsafe {
    //     let mut c = 0;
    //     stack_analysis::copy_stack(RAM_START, RAM_START + 10, &mut c);
    //     stack_analysis::print_stack();
    // }

    let (used, start_stack_ptr) = unsafe { stack_analysis::count_used_ram() };
    // info!(
    //     "used stack: {}bytes\n static data: {}bytes",
    //     used,
    //     0x20_008_000 - start_stack_ptr
    // );

    let m_p = micros.to_be_bytes();
    let mut payload = used.to_be_bytes();
    payload[1] = m_p[0];
    let header = TxFrameHeader {
        bit_rate_switching: false,
        frame_format: stm32g4xx_hal::fdcan::frame::FrameFormat::Standard,
        id: Id::Extended(ExtendedId::new(0x1).unwrap()),
        len: payload.len() as u8,
        marker: None,
    };
    block!(can.transmit(header, &mut |b| {
        insert_u8_array_in_u32_array(&payload, b)
    },))
    .unwrap();

    loop {}
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
