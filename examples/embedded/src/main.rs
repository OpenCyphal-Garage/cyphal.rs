#![no_main]
#![no_std]
// to use the global allocator
#![feature(alloc_error_handler)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(feature = "logging")]
mod logging;

mod allocator;
mod clock;
mod util;

mod to_test;

use arrayvec::ArrayVec;
use defmt::info;
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
use cortex_m_rt::entry;

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
    transfer::Transfer,
    transport::can::{Can, CanFrame, CanMetadata, TailByte},
    Node, Priority, Subscription, TransferKind,
};

use util::insert_u8_array_in_u32_array;

use crate::to_test::publish;

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

    let mut session_manager = HeapSessionManager::<CanMetadata, Milliseconds, StmClock>::new();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            7168, // TODO check
            12,
            embedded_time::duration::Milliseconds(500),
        ))
        .unwrap();

    let mut node = Node::<_, Can, StmClock>::new(Some(42), session_manager);

    let mut transfer_id = 0u8;
    let mut last_published = clock.try_now().unwrap();

    loop {
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

        if now - last_published
            > embedded_time::duration::Generic::new(1000, StmClock::SCALING_FACTOR)
        {
            let (id, (p1, p2)) = fake_can_frame_12_byte(&mut transfer_id, &clock);
            let frame = CanFrame {
                id,
                payload: p1,
                timestamp: clock.try_now().unwrap(),
            };
            let start = measure_clock.now();
            let _ = node.try_receive_frame(frame).unwrap();
            let frame = CanFrame {
                id,
                payload: p2,
                timestamp: clock.try_now().unwrap(),
            };
            if let Some(frame) = node.try_receive_frame(frame).unwrap() {
                match frame.transfer_kind {
                    TransferKind::Message => (),
                    _ => (),
                }
            }
            let elapsed = start.elapsed();
            let micros: u32 = measure_clock.frequency().duration(elapsed).0;
            info!("elapsed: {} micros", micros);

            last_published = now;
        }
    }
}

// 1 frame in normal can mode
fn fake_can_frame_7_byte(transfer_id: &mut u8, clock: &StmClock) -> CanFrame<StmClock> {
    let mut payload = ArrayVec::from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x0]);
    let tail = TailByte::new(true, true, true, *transfer_id);
    *transfer_id = (*transfer_id).wrapping_add(1);
    payload[7] = tail;
    CanFrame {
        id: 0x107c000d,
        payload,
        timestamp: clock.try_now().unwrap(),
    }
}

// 2 frame in normal can mode
fn fake_can_frame_12_byte(
    transfer_id: &mut u8,
    clock: &StmClock,
) -> (u32, (ArrayVec<[u8; 8]>, ArrayVec<[u8; 8]>)) {
    let mut payload = [
        ArrayVec::from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x0]),
        ArrayVec::from([0x8, 0x9, 0xA, 0xB, 0xC, 0x0, 0x0, 0x0]),
    ];
    let mut crc = crc_any::CRCu16::crc16ccitt_false();
    let tail = TailByte::new(true, false, true, *transfer_id);
    payload[0][7] = tail;
    let tail = TailByte::new(false, true, false, *transfer_id);
    payload[1][7] = tail;

    crc.digest(&payload[0][..7]);
    crc.digest(&payload[1][..5]);

    let crc = crc.get_crc().to_be_bytes();
    payload[1][5] = crc[0];
    payload[1][6] = crc[1];

    *transfer_id = (*transfer_id).wrapping_add(1);
    (0x107c000d, (payload[0].clone(), payload[1].clone()))
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
