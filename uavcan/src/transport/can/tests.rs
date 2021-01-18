use super::*;

#[test]
fn discard_empty_frame() {
    let frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: 0,
        payload: ArrayVec::new(),
    };
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.expect_err("Empty frame did not error out.");
    assert!(std::matches!(err, RxError::FrameEmpty), "Did not catch empty frame!");
}

#[test]
fn tail_byte_checks() {
    // Start with invalid tail byte - toggle should be true to start transfer
    let tail_byte = TailByte::new(true, true, false, 0);
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: ArrayVec::new(),
    };

    frame.payload.push(tail_byte.to_u8().unwrap());
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.expect_err("Invalid toggle");
    assert!(std::matches!(err, RxError::TransferStartMissingToggle), "Did not catch invalid start toggle");

    let tail_byte = TailByte::new(true, false, true, 0);
    frame.payload[0] = tail_byte.to_u8().unwrap();
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.expect_err("Invalid toggle");
    println!("{:?}", err);
    assert!(std::matches!(err, RxError::NonLastUnderUtilization), "Did not catch unfilled non-end frame");
}

#[test]
fn receive_anon_frame() {
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: ArrayVec::new(),
    };

    // TODO there must be a way to do it something like this way
    //frame.payload.extend([1, 2, 3, 4, 5, TailByte::new(true, true, true, 0).to_u8().unwrap()].iter());
    for i in 0..4 {
        frame.payload.push(i);
    }
    frame.payload.push(TailByte::new(true, true, true, 0));

    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing anon frame");
    let frame = result.expect("Failed to process anon frame");

    assert!(std::matches!(frame.priority, Priority::Nominal));
    assert!(std::matches!(frame.source_node_id, None));
    assert!(std::matches!(frame.destination_node_id, None));
    assert_eq!(frame.port_id, 0);
    assert_eq!(frame.transfer_id, 0);
    assert_eq!(frame.start_of_transfer, true);
    assert_eq!(frame.end_of_transfer, true);
    // Note that tail byte is supposed to be preserved here for CanMetadata to deal with.
    assert_eq!(frame.payload, [0, 1, 2, 3, 224]);
}

#[test]
fn discard_anon_multi_frame() {
}
