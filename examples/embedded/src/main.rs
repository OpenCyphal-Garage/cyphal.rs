#![no_main]
#![no_std]
// to use the global allocator
#![feature(alloc_error_handler)]

// Some panic handler needs to be included. This one halts the processor on panic.
extern crate panic_halt;

use core::alloc::Layout;

use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::entry;

use cortex_m as _;

use embedded_time::{duration::Milliseconds, Clock};
use hal::stm32;
use stm32g4xx_hal as hal;

use uavcan::{
    session::HeapSessionManager,
    transfer::Transfer,
    transport::can::{Can, CanMetadata},
    Node, Priority, Subscription, TransferKind,
};
use util::{turn_board_led_off, turn_board_led_on};

mod setup_hal;
mod util;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

/// A clock for the stm32
///
/// Uses the stm32 hal `HAL_GetTick`. Underlying u32 counter, will wrap after 50 days.
#[derive(Clone)]
struct StmClock;

impl StmClock {
    fn new() -> Self {
        Self {}
    }
}

impl Clock for StmClock {
    type T = u32;

    const SCALING_FACTOR: embedded_time::rate::Fraction =
        embedded_time::rate::Fraction::new(1, 1000);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        Ok(embedded_time::Instant::new(unsafe {
            stm32_hal::HAL_GetTick()
        }))
    }
}

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

    let peripherals = stm32::Peripherals::take().unwrap();
    let gpioa = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;

    let mut handles = unsafe { setup_hal::init_hal_and_config_board(gpioa, rcc) };

    let clock = StmClock::new();

    let mut session_manager = HeapSessionManager::<CanMetadata, Milliseconds, StmClock>::new();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            7509, // TODO check
            7,
            embedded_time::duration::Milliseconds(500),
        ))
        .unwrap();

    let mut node = Node::<_, Can, StmClock>::new(Some(42), session_manager);

    let mut transfer_id = 0u8;
    let mut last_published = clock.try_now().unwrap();

    loop {
        if clock.try_now().unwrap() - last_published
            > embedded_time::duration::Generic::new(1000, StmClock::SCALING_FACTOR)
        {
            // Publish string
            let hello = "Hello Python!";
            let mut str = heapless::Vec::<u8, 13>::new();
            str.extend_from_slice(hello.as_bytes()).unwrap();

            let transfer = Transfer {
                timestamp: clock.try_now().unwrap(),
                priority: Priority::Nominal,
                transfer_kind: TransferKind::Message,
                port_id: 100,
                remote_node_id: None,
                transfer_id,
                payload: &str,
            };

            // unchecked_add is unstable :(
            // unsafe { transfer_id.unchecked_add(1); }
            transfer_id += 1;

            for mut frame in node.transmit(&transfer).unwrap() {
                let mut tx_header = unsafe {
                    let mut tx_header = core::mem::zeroed::<stm32_hal::FDCAN_TxHeaderTypeDef>();
                    tx_header.Identifier = frame.id;
                    tx_header.TxFrameType = 0x0;
                    tx_header.IdType = 0x0;
                    tx_header.DataLength = 0x00050000;
                    tx_header.BitRateSwitch = 0x0;
                    tx_header.FDFormat = 0x0;
                    tx_header.TxEventFifoControl = 0x0;
                    tx_header.MessageMarker = 0x00;
                    tx_header.ErrorStateIndicator = 0x0;
                    tx_header
                };

                unsafe {
                    stm32_hal::HAL_FDCAN_AddMessageToTxFifoQ(
                        &mut handles.fdcan as *mut stm32_hal::FDCAN_HandleTypeDef,
                        &mut tx_header as *mut stm32_hal::FDCAN_TxHeaderTypeDef,
                        frame.payload.as_mut_ptr(),
                    );
                }
            }

            last_published = clock.try_now().unwrap();

            turn_board_led_on(&gpioa);
            unsafe {
                stm32_hal::HAL_Delay(1000);
            }
            turn_board_led_off(&gpioa);
        }
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
