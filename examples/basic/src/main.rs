use embedded_time::duration::Milliseconds;
use embedded_time::Clock;
use uavcan::session::StdVecSessionManager;
use uavcan::time::StdClock;
use uavcan::transport::can::{Can, CanFrame as UavcanFrame, CanMetadata};
use uavcan::{transfer::Transfer, types::TransferId, Node, Priority, Subscription, TransferKind};

use arrayvec::ArrayVec;
use socketcan::{CANFrame, CANSocket};

fn main() {
    let clock = StdClock::default();
    let mut session_manager = StdVecSessionManager::<CanMetadata, Milliseconds>::new();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            7509, // TODO check
            7,
            embedded_time::duration::Milliseconds(500),
        ))
        .unwrap();
    session_manager
        .subscribe(Subscription::new(
            TransferKind::Message,
            100,
            200,
            embedded_time::duration::Milliseconds(500),
        ))
        .unwrap();
    let mut node: Node<
        StdVecSessionManager<CanMetadata, embedded_time::duration::Milliseconds>,
        Can,
        _,
    > = Node::new(Some(42), session_manager);

    let sock = CANSocket::open("vcan0").unwrap();

    let mut last_publish = clock.try_now().unwrap();
    let mut transfer_id: TransferId = 30;

    sock.set_read_timeout(std::time::Duration::from_millis(100))
        .unwrap();

    loop {
        let socketcan_frame = sock.read_frame().ok();

        if let Some(socketcan_frame) = socketcan_frame {
            // Note that this exposes some things I *don't* like about the API
            // 1: we should have CanFrame::new or something
            // 2: I don't like how the payload is working
            let mut uavcan_frame = UavcanFrame {
                timestamp: clock.try_now().unwrap(),
                id: socketcan_frame.id(),
                payload: ArrayVec::new(),
            };
            uavcan_frame
                .payload
                .extend(socketcan_frame.data().iter().copied());

            let xfer = match node.try_receive_frame(uavcan_frame) {
                Ok(xfer) => xfer,
                Err(err) => {
                    println!("try_receive_frame error: {:?}", err);
                    return;
                }
            };

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
                    TransferKind::Request => {
                        println!("Request Received!");
                    }
                    TransferKind::Response => {
                        println!("Response Received!");
                    }
                }
            }
        }

        if clock.try_now().unwrap() - last_publish
            > embedded_time::duration::Generic::new(500, StdClock::SCALING_FACTOR)
        {
            // Publish string
            let hello = "Hello Python!";
            let mut str = Vec::from([hello.len() as u8, 0]);
            str.extend_from_slice(hello.as_bytes());

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
            transfer_id = (std::num::Wrapping(transfer_id) + std::num::Wrapping(1)).0;

            for frame in node.transmit(&transfer).unwrap() {
                sock.write_frame(&CANFrame::new(frame.id, &frame.payload, false, false).unwrap())
                    .unwrap();

                //print!("Can frame {}: ", i);
                //for byte in &frame.payload {
                //    print!("0x{:02x} ", byte);
                //}
                //println!("");

                //if let Some(in_xfer) = node.try_receive_frame(frame).unwrap() {
                //    println!("Received xfer!");
                //}
            }

            last_publish = clock.try_now().unwrap();
        }
    }
}
