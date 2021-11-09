use embedded_time::Clock;
use streaming_iterator::StreamingIterator;
use uavcan::{
    session::SessionManager, transfer::Transfer, transport::Transport, Node, Priority, TransferKind,
};

use super::{context::NeedsClock, Bencher};

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

#[optimize(speed)]
fn publish<S: SessionManager<C>, T: Transport<C>, C: embedded_time::Clock + 'static + Clone>(
    node: &mut Node<S, T, C>,
    transfer: Transfer<C>,
) {
    let mut frame_iter = node.transmit(&transfer).unwrap();
    while let Some(frame) = frame_iter.next() {
        core::hint::black_box(frame);
    }
}

pub(crate) fn bench_send<CM: embedded_time::Clock, Context, const N: usize>(
    bencher: &mut Bencher<CM>,
    context: &mut Context,
) where
    Context: NeedsClock,
{
    let data = core::hint::black_box(get_test_payload::<N>());
    let mut transfer_id = 0;

    bencher.run_with_watch(|watch| {
        let transfer = Transfer {
            timestamp: context.clock_as_mut().try_now().unwrap(),
            priority: Priority::Nominal,
            transfer_kind: TransferKind::Message,
            port_id: 100,
            remote_node_id: None,
            transfer_id,
            payload: &data,
        };
        {
            watch.start();
            publish(context.node_as_mut(), core::hint::black_box(transfer));
            watch.stop();
        }

        transfer_id += 1;
    });
}
