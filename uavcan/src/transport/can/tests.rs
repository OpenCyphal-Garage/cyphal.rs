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
fn transport_tail_byte_checks() {
    
}
