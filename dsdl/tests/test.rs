extern crate dsdl;
extern crate uavcan;

use uavcan::Message;
use uavcan::Response;
use uavcan::Request;

#[test]
fn test_existence() {
    assert_eq!(dsdl::uavcan::protocol::NodeStatus::TYPE_ID, Some(341));
    assert_eq!(dsdl::uavcan::protocol::GetNodeInfoRequest::TYPE_ID, Some(1));
    assert_eq!(dsdl::uavcan::protocol::GetNodeInfoResponse::TYPE_ID, Some(1));
}
