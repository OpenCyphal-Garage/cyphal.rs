# Uavcan.rs design goals

## Design Goals
- High robustness and safety
- Feature-completeness, suitability for complex applications (like Libuavcan, unlike Libcanard). This includes:
    - Network discovery
    - Time synchronization
    - Dynamic ID allocation server and client
- Ease of use
- Assume as little as possible about the application
- Possible to extend for CAN-FD (after Uavcan supports this)
