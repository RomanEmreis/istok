---
name: Protocol Testing
applies_to: [claude, codex]
triggers: [new H3 behavior, new frame type, engine state change, any test addition]
---

# Protocol Testing

## Purpose

Defines how to write deterministic, harness-based tests for H3 engine behavior.
No real sockets, no real timers — every byte and every event is scripted.

## Deterministic harness (`MockHarness`)

Location: `crates/istok-h3/src/mock/`

The harness is script-driven:
1. Construct a `MockHarness` with scripted incoming stream events.
2. Drive the engine until it halts or produces expected writes.
3. Assert on captured outgoing writes: frame bytes, FIN flags, ordering.

```rust
// Minimal test structure
let harness = MockHarness::new(vec![
    // scripted inbound events
    StreamEvent::Data { stream_id: 0, data: vec![0x00], fin: false },
]);
let writes = harness.run_until_idle();
assert_eq!(writes[0].data, expected_headers_frame);
assert!(!writes[0].fin);
assert_eq!(writes[1].data, expected_data_frame);
assert!(writes[1].fin);
```

## Minimal, deterministic payloads

Keep test payloads small and recognizable:
- HEADERS payload: `[0x00]` (one-byte placeholder, no QPACK encoding yet)
- DATA payload: `[0x01]`
- Control stream setup bytes: minimal valid SETTINGS frame

Avoid random or generated data in unit tests — use fuzz targets for that.

## Codec tests

Every codec (varint, frame) needs three test classes:

| Class | What to test |
|---|---|
| Roundtrip | `encode(x)` → `decode(…)` == `x` |
| Malformed rejection | Known-bad byte sequences must return an error, not panic |
| Boundary conditions | Max varint value, empty payload, max frame length |

## Fuzzing (future)

Fuzz targets will live in `fuzz/` and cover:
- Frame decoding (`Frame::parse`)
- QPACK decoding (once M2 is active)

Rule: crashes are bugs. The engine must never panic on arbitrary input.

## Anti-patterns

- Real `tokio::time::sleep` or `tokio::net` in unit tests — use the mock harness.
- Asserting only on the absence of errors — also assert on the actual bytes written.
- Tests that pass with wrong FIN placement — always check `fin` explicitly.
