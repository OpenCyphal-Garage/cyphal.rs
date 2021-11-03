use defmt::{info, Format};
use embedded_time::fixed_point::FixedPoint;
use streaming_iterator::StreamingIterator;
use uavcan::{
    session::SessionManager, transfer::Transfer, transport::can::Can, Node, Priority, TransferKind,
};

use super::Bencher;

pub struct Context<'a, S: SessionManager<C>, C: embedded_time::Clock + 'static + Clone> {
    pub node: Node<S, Can, C>,
    pub clock: &'a C,
}

fn get_test_payload<const N: usize>() -> heapless::Vec<u8, N> {
    heapless::Vec::<u8, N>::from_iter(
        core::iter::from_fn(|| {
            static mut COUNT: u8 = 0;
            unsafe {
                COUNT += 1;
                Some(COUNT)
            }
        })
        .take(N),
    )
}

pub(crate) fn bench_send<
    S: SessionManager<C>,
    C: embedded_time::Clock + 'static + Clone,
    CM: embedded_time::Clock,
    const N: usize,
>(
    bencher: &mut Bencher<CM>,
    context: &mut Context<S, C>,
) {
    let data = get_test_payload::<N>();
    let mut transfer_id = 0;

    bencher.run_with_watch(|watch| {
        let transfer = Transfer {
            timestamp: context.clock.try_now().unwrap(),
            priority: Priority::Nominal,
            transfer_kind: TransferKind::Message,
            port_id: 100,
            remote_node_id: None,
            transfer_id,
            payload: &data,
        };
        {
            watch.start();
            let mut frame_iter = context.node.transmit(&transfer).unwrap();
            while let Some(frame) = frame_iter.next() {
                core::hint::black_box(frame);
            }
            watch.stop();
        }

        transfer_id += 1;
    });
}
