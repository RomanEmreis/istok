# Testing Strategy

## Deterministic tests first
Use mock transport to simulate datagrams and timers without relying on real sockets.

## Codec tests
- roundtrip encode/decode
- malformed inputs
- property tests (where feasible)

## Integration tests
- tokio adapter brings real network tests
- interop targets: curl --http3
