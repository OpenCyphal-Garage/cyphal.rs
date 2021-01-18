use super::*;

// TODO make this a macro or something for more relevant error messages
fn all_frame_asserts(frame: InternalRxFrame, source_id: Option<NodeId>, destination_id: Option<NodeId>, start: bool, end: bool, payload: &[u8]) {
    assert!(std::matches!(frame.priority, Priority::Nominal));
    assert_eq!(frame.source_node_id, source_id);
    assert_eq!(frame.destination_node_id, destination_id);
    assert_eq!(frame.port_id, 0);
    assert_eq!(frame.transfer_id, 0);
    assert_eq!(frame.start_of_transfer, start);
    assert_eq!(frame.end_of_transfer, end);
    assert_eq!(frame.payload, payload);
}

#[test]
fn receive_anon_frame() {
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: ArrayVec::new(),
    };

    frame.payload.extend(0..5);
    frame.payload.push(TailByte::new(true, true, true, 0));

    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing anon frame");
    let frame = result.expect("Failed to process anon frame");

    all_frame_asserts(frame, None, None, true, true, &[0, 1, 2, 3, 4, 224]);
}

#[test]
fn receive_message_frame() {
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanMessageId::new(Priority::Nominal, 0, Some(41)),
        payload: ArrayVec::new(),
    };

    frame.payload.push(TailByte::new(true, true, true, 0));
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing message frame");
    let frame = result.expect("Failed to process message frame");

    all_frame_asserts(frame, Some(41), None, true, true, &[224])
}

#[test]
fn receive_service_frame() {
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanServiceId::new(Priority::Nominal, false, 0, 42, 41),
        payload: ArrayVec::new(),
    };

    frame.payload.push(TailByte::new(true, true, true, 0));
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing service response frame");
    let internal_frame = result.expect("Failed to process valid service response frame");
    all_frame_asserts(internal_frame, Some(41), Some(42), true, true, &[224]);

    let mut frame = frame.clone();
    frame.id = CanServiceId::new(Priority::Nominal, true, 0, 42, 41);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing service request frame");
    let internal_frame = result.expect("Failed to process valid service request frame");
    all_frame_asserts(internal_frame, Some(41), Some(42), true, true, &[224]);
}

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
fn discard_anon_multi_frame() {
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: ArrayVec::new(),
    };

    // Need to fill frame to avoid tail_byte_checks() cases
    frame.payload.extend(0..7);

    frame.payload.push(TailByte::new(false, true, true, 0));
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.unwrap_err();
    assert!(std::matches!(err, RxError::AnonNotSingleFrame));

    frame.payload[7] = TailByte::new(true, false, true, 0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.unwrap_err();
    assert!(std::matches!(err, RxError::AnonNotSingleFrame));

    frame.payload[7] = TailByte::new(false, false, true, 0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.unwrap_err();
    assert!(std::matches!(err, RxError::AnonNotSingleFrame));
}

#[test]
fn discard_misguided_service_frames() {
    let mut frame = CanFrame {
        timestamp: std::time::Instant::now(),
        id: CanServiceId::new(Priority::Nominal, true, 0, 31, 41),
        payload: ArrayVec::new(),
    };

    // Request
    frame.payload.push(TailByte::new(true, true, true, 0));
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.unwrap();
    assert!(std::matches!(result, None), "Didn't discard misguided service request");

    // Request (anonymous node)
    let result = Can::rx_process_frame(&None, &frame);
    let result = result.unwrap();
    assert!(std::matches!(result, None), "Didn't discard service request to anonymous node");

    // Response
    frame.id = CanServiceId::new(Priority::Nominal, false, 0, 31, 41);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.unwrap();
    assert!(std::matches!(result, None), "Didn't discard misguided service response");

    let result = Can::rx_process_frame(&None, &frame);
    let result = result.unwrap();
    assert!(std::matches!(result, None), "Didn't discard service response to anonymous node");
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

// TODO do I need tests for valid IDs?