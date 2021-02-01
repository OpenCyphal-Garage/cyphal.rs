use uavcan::{Node, Subscription, TransferKind, transfer::Transfer};
use uavcan::session::StdVecSessionManager;
use uavcan::transport::can::{Can, CanMetadata, CanFrame as UavcanFrame};

use socketcan::{CANFrame, CANSocket};
use arrayvec::ArrayVec;

fn main() {
    let mut session_manager = StdVecSessionManager::<CanMetadata>::new();
    session_manager.subscribe(Subscription::new(
        TransferKind::Message,
        7509, // TODO check
        7,
        core::time::Duration::from_millis(500)
    )).unwrap();
    let mut node: Node<StdVecSessionManager<CanMetadata>, Can> = Node::new(Some(42), session_manager);


    let sock = CANSocket::open("vcan0").unwrap();


    loop {
        let socketcan_frame = sock.read_frame().unwrap();

        // Note that this exposes some things I *don't* like about the API
        // 1: we should have CanFrame::new or something
        // 2: I don't like how the payload is working
        let mut uavcan_frame = UavcanFrame {
            timestamp: std::time::Instant::now(),
            id: socketcan_frame.id(),
            payload: ArrayVec::new(),
        };
        uavcan_frame.payload.extend(socketcan_frame.data().iter().copied());

        let xfer = match node.try_receive_frame(uavcan_frame) {
            Ok(xfer) => xfer,
            Err(err) => {
                println!("try_receive_frame error: {:?}", err);
                return;
            }
        };

        // Two errors to deal with:
        // 1. payload len is 1 ???
        if let Some(xfer) = xfer {
            match xfer.transfer_kind {
                TransferKind::Message => {
                    println!("UAVCAN message received!");
                    print!("\tData: ");
                    for byte in xfer.payload {
                        print!("0x{:02x} ", byte);
                    }
                    println!("");
                }
                _ => {
                    // TODO
                }
            }

        }

    }
}
