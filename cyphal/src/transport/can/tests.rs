use alloc::vec;

use crate::time::TestClock;
use arrayvec::ArrayVec;
use embedded_hal::can::ExtendedId;
use embedded_time::Clock;

use super::bitfields::TailByte;
use super::{legacy::*, *};
use crate::internal::InternalRxFrame;
use crate::transport::Transport;
use crate::*;

// I feel I may have gone overboard with these tests, but I'm still getting to grips with
// testing well so I'm not sure where the boundary should be.

// TODO make this a macro or something for more relevant error messages
fn all_frame_asserts<C: embedded_time::Clock>(
    frame: InternalRxFrame<C>,
    source_id: Option<NodeId>,
    destination_id: Option<NodeId>,
    start: bool,
    end: bool,
    payload: &[u8],
) {
    assert!(matches!(frame.priority, Priority::Nominal));
    assert_eq!(frame.source_node_id, source_id);
    assert_eq!(frame.destination_node_id, destination_id);
    assert_eq!(frame.port_id, 0);
    assert_eq!(frame.transfer_id, 0);
    assert_eq!(frame.start_of_transfer, start);
    assert_eq!(frame.end_of_transfer, end);
    assert_eq!(frame.payload, payload);
}

/// Ensure valid anonymous frames are recieved properly.
#[test]
fn receive_anon_frame() {
    let clock = TestClock::default();
    let mut frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: arrayvec::ArrayVec::<[u8; 8]>::new(),
    };

    frame.payload.extend(0..5);
    frame.payload.push(TailByte::new(true, true, true, 0).0);

    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing anon frame");
    let frame = result.expect("Failed to process anon frame");

    all_frame_asserts(frame, None, None, true, true, &[0, 1, 2, 3, 4, 224]);
}

/// Ensure that valid message frames are recieved properly.
#[test]
fn receive_message_frame() {
    let clock = TestClock::default();
    let mut frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: CanMessageId::new(Priority::Nominal, 0, Some(41)),
        payload: arrayvec::ArrayVec::<[u8; 8]>::new(),
    };

    frame.payload.push(TailByte::new(true, true, true, 0).0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing message frame");
    let frame = result.expect("Failed to process message frame");

    all_frame_asserts(frame, Some(41), None, true, true, &[224])
}

/// Ensure that valid service frames are recieved properly.
#[test]
fn receive_service_frame() {
    let clock = TestClock::default();
    let mut frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: CanServiceId::new(Priority::Nominal, false, 0, 42, 41),
        payload: arrayvec::ArrayVec::<[u8; 8]>::new(),
    };

    frame.payload.push(TailByte::new(true, true, true, 0).0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing service response frame");
    let internal_frame = result.expect("Failed to process valid service response frame");
    all_frame_asserts(internal_frame, Some(41), Some(42), true, true, &[224]);

    let mut frame = frame;
    frame.id = CanServiceId::new(Priority::Nominal, true, 0, 42, 41);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.expect("Error processing service request frame");
    let internal_frame = result.expect("Failed to process valid service request frame");
    all_frame_asserts(internal_frame, Some(41), Some(42), true, true, &[224]);
}

/// Any transmitted frame must at minimum have a tail byte, so discard empty frames.
#[test]
fn discard_empty_frame() {
    let clock = TestClock::default();
    let frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: ExtendedId::ZERO,
        payload: ArrayVec::new(),
    };
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.expect_err("Empty frame did not error out.");
    assert!(
        matches!(err, RxError::FrameEmpty),
        "Did not catch empty frame!"
    );
}

/// Anonymous transfers must be limited to single frames.
#[test]
fn discard_anon_multi_frame() {
    let clock = TestClock::default();
    let mut frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: arrayvec::ArrayVec::<[u8; 8]>::new(),
    };

    // Need to fill frame to avoid tail_byte_checks() cases
    frame.payload.extend(0..7);

    frame.payload.push(TailByte::new(false, true, true, 0).0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.unwrap_err();
    assert!(matches!(err, RxError::AnonNotSingleFrame));

    frame.payload[7] = TailByte::new(true, false, true, 0).0;
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.unwrap_err();
    assert!(matches!(err, RxError::AnonNotSingleFrame));

    frame.payload[7] = TailByte::new(false, false, true, 0).0;
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.unwrap_err();
    assert!(matches!(err, RxError::AnonNotSingleFrame));
}

/// Service transfers to non-local nodes can safely be ignored.
#[test]
fn discard_misguided_service_frames() {
    let clock = TestClock::default();
    let mut frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: CanServiceId::new(Priority::Nominal, true, 0, 31, 41),
        payload: arrayvec::ArrayVec::<[u8; 8]>::new(),
    };

    // Request
    frame.payload.push(TailByte::new(true, true, true, 0).0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.unwrap();
    assert!(
        matches!(result, None),
        "Didn't discard misguided service request"
    );

    // Request (anonymous node)
    let result = Can::rx_process_frame(&None, &frame);
    let result = result.unwrap();
    assert!(
        matches!(result, None),
        "Didn't discard service request to anonymous node"
    );

    // Response
    frame.id = CanServiceId::new(Priority::Nominal, false, 0, 31, 41);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let result = result.unwrap();
    assert!(
        matches!(result, None),
        "Didn't discard misguided service response"
    );

    let result = Can::rx_process_frame(&None, &frame);
    let result = result.unwrap();
    assert!(
        matches!(result, None),
        "Didn't discard service response to anonymous node"
    );
}

/// Tests that several validity checks on tail bytes are properly caught.
#[test]
fn tail_byte_checks() {
    let clock = TestClock::default();
    // Start with invalid tail byte - toggle should be true to start transfer
    let mut frame = CanFrame {
        timestamp: clock.try_now().unwrap(),
        id: CanMessageId::new(Priority::Nominal, 0, None),
        payload: arrayvec::ArrayVec::<[u8; 8]>::new(),
    };

    frame.payload.push(TailByte::new(true, true, false, 0).0);
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.expect_err("Invalid toggle");
    assert!(
        matches!(err, RxError::TransferStartMissingToggle),
        "Did not catch invalid start toggle"
    );

    frame.payload[0] = TailByte::new(true, false, true, 0).0;
    let result = Can::rx_process_frame(&Some(42), &frame);
    let err = result.expect_err("Invalid toggle");
    assert!(
        matches!(err, RxError::NonLastUnderUtilization),
        "Did not catch unfilled non-end frame"
    );
}

/// Creates a transfer of message type to reduce boilerplate code in testing some
/// CanIter functionality
fn make_generic_message_transfer(payload: &[u8]) -> crate::transfer::Transfer<TestClock> {
    let clock = TestClock::default();
    crate::transfer::Transfer {
        timestamp: clock.try_now().unwrap(),
        priority: Priority::Nominal,
        transfer_kind: TransferKind::Message,
        port_id: 0,
        remote_node_id: None,
        transfer_id: 0,
        payload,
    }
}

/// Tests that creating new transfers populates the ID correctly.
#[test]
fn transfer_valid_ids() {
    let mut transfer = make_generic_message_transfer(&[1, 2, 3]);

    // User wouldn't be expected to deal with CanIter, as it's called higher up the stack,
    // but this is the most ergonomic entry point for this test.

    // Anonymous message
    let mut can_iter = CanIter::new(&transfer, None).unwrap();
    let frame: &CanFrame<TestClock<u32>> = can_iter.next().expect("Failed to create iter");
    let id = CanMessageId(frame.id.as_raw());
    assert!(id.is_message());
    assert!(id.is_anon());
    assert!(id.subject_id() == 0);
    assert!(id.priority() == Priority::Nominal as u8);
    // Source ID should be random, not sure how to handle this...

    let mut can_iter = CanIter::new(&transfer, Some(12)).unwrap();
    let frame: &CanFrame<TestClock<u32>> = can_iter.next().expect("");
    let id = CanMessageId(frame.id.as_raw());
    assert!(id.is_message());
    assert!(!id.is_anon());
    assert!(id.subject_id() == 0);
    assert!(id.priority() == Priority::Nominal as u8);

    transfer.transfer_kind = TransferKind::Request;
    let err = CanIter::new(&transfer, None).expect_err("Anonymous service transfers not allowed");
    assert!(matches!(err, TxError::ServiceNoSourceID));

    // TODO finish out these tests. Maybe split this into more tests as well?
}

/// Checks that the iterator produces the expected number of frames.
fn assert_frame_count(mut iter: CanIter<TestClock>, mut expected: usize) {
    while let Some(_frame) = iter.next() {
        assert!(expected > 0);
        expected -= 1;
    }
}

// TODO perhaps test placement of CRC as well in these functions
/// Tests that the iterator operates correctly when CRC portion is split between the last
/// two frames.
#[test]
fn iter_crc_split() {
    let buf = vec![0u8; 13];
    let transfer = make_generic_message_transfer(buf.as_slice());
    assert_frame_count(CanIter::new(&transfer, Some(0)).unwrap(), 3);
}

/// Tests that the iterator operates correctly when CRC portion is included with the last
/// data frame.
#[test]
fn iter_crc_inclusive() {
    let buf = vec![0u8; 12];
    let transfer = make_generic_message_transfer(buf.as_slice());
    assert_frame_count(CanIter::new(&transfer, Some(0)).unwrap(), 2);
}

/// Tests that the iterator operates correctly when the CRC portion is the entire contents
/// of the last frame.
#[test]
fn iter_crc_exclusive() {
    let buf = vec![0u8; 14];
    let transfer = make_generic_message_transfer(buf.as_slice());
    assert_frame_count(CanIter::new(&transfer, Some(0)).unwrap(), 3);
}
