use uavcan::{
    session::SessionManager,
    transport::can::{Can, CanFrame, CanMessageId, FakePayloadIter},
    types::PortId,
    Node,
};

use super::Bencher;

pub struct Context<'a, S: SessionManager<C>, C: embedded_time::Clock + 'static + Clone> {
    pub node: Node<S, Can, C>,
    pub clock: &'a C,
}

pub(crate) fn bench_receive<
    S: SessionManager<C>,
    C: embedded_time::Clock + 'static + Clone,
    CM: embedded_time::Clock,
    const N: usize,
>(
    bencher: &mut Bencher<CM>,
    context: &mut Context<S, C>,
) {
    let port_id: PortId = 7168;
    let message_id = CanMessageId::new(uavcan::Priority::Immediate, port_id, Some(1));
    let mut transfer_id = 0u8;

    bencher.run_with_watch(|watch| {
        let payload_iter = FakePayloadIter::<8>::multi_frame(N, transfer_id);
        for payload in payload_iter {
            let payload = arrayvec::ArrayVec::from_iter(payload);
            let frame = CanFrame {
                id: message_id,
                payload,
                timestamp: context.clock.try_now().unwrap(),
            };
            watch.start();
            if let Some(frame) = context.node.try_receive_frame(frame).unwrap() {
                core::hint::black_box(frame);
            }
            watch.stop();
            transfer_id = transfer_id.wrapping_add(1);
        }
    })
}
