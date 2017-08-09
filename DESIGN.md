# Uavcan.rs design goals

## Design Goals
- High robustness and safety
- Feature-completeness, suitability for complex applications (like Libuavcan, unlike Libcanard). This includes:
    - rpc like features on top of service frames both from a caller and responder perspective. The caller should be able to choose between synchronous and asynchronous alternatives.
- Provide "high level" functionality. To make the implementation as maintainable as possible, these should require as little knowledge of the internals as possibly. They will more often than not exist in their own crate depending on the other uavcan crates. Some of the feature that will be supported is listed here:
    - Network discovery
    - Time synchronization
    - Dynamic ID allocation server and client
- Ease of use
- Assume as little as possible about the application
- Possible to extend for CAN-FD (after Uavcan supports this)
