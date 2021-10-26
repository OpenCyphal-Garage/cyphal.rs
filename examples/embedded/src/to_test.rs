use embedded_time::duration::Milliseconds;
use stm32g4xx_hal::{
    block,
    fdcan::{
        frame::TxFrameHeader,
        id::{ExtendedId, Id},
        FdCan, NormalOperationMode,
    },
    stm32::FDCAN1,
    timer::MonoTimer,
};
use uavcan::{
    session::HeapSessionManager,
    transfer::Transfer,
    transport::can::{Can, CanFrame, CanIter, CanMetadata},
    Node,
};

use crate::{clock::StmClock, util::insert_u8_array_in_u32_array};

#[no_mangle]
pub fn publish(
    clock: &MonoTimer,
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    transfer: Transfer<StmClock>,
    can: &mut FdCan<FDCAN1, NormalOperationMode>,
) {
    let start = clock.now();
    let mut iter = get_iter_from_transfer(node, &transfer);
    let frame = get_next_frame(&mut iter);
    let elapsed = start.elapsed();
    transmit_fdcan(frame, can);

    let micros: u32 = clock.frequency().duration(elapsed).0;
    // info!("elapsed: {} micros", micros);
}

#[no_mangle]
fn get_next_frame(iter: &mut CanIter<StmClock>) -> CanFrame<StmClock> {
    iter.next().unwrap()
}

#[no_mangle]
fn get_iter_from_transfer<'a>(
    node: &mut Node<HeapSessionManager<CanMetadata, Milliseconds<u32>, StmClock>, Can, StmClock>,
    transfer: &'a Transfer<StmClock>,
) -> CanIter<'a, StmClock> {
    node.transmit(transfer).unwrap()
}

#[no_mangle]
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
