---
name: Protocol Testing
description: How to write deterministic, harness-based H3 engine tests using MockHarness — no real sockets, no timers, script-driven and byte-exact.
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

## Fuzzing

Fuzz targets live in `crates/istok-core/fuzz/fuzz_targets/` and are built with
`cargo fuzz` (nightly required). Currently covering:
- `fuzz_varint_decode` — `varint::decode`
- `fuzz_frame_decode` — `h3_frame::decode_frame_header`

To run locally (`+nightly` is required — cargo-fuzz uses `-Z` flags):
```sh
cd crates/istok-core
cargo +nightly fuzz run fuzz_varint_decode
cargo +nightly fuzz run fuzz_frame_decode
```

Runs indefinitely until `Ctrl+C`. Corpus is saved in `fuzz/corpus/` and seeds future runs.

For a time-bounded run (CI-style):
```sh
ASAN_OPTIONS="detect_odr_violation=0:quarantine_size_mb=1:malloc_context_size=0" \
cargo +nightly fuzz run fuzz_varint_decode -- -max_total_time=30 -max_len=8 -rss_limit_mb=256
```

Use `max_len=8` for varint (QUIC varint max is 8 bytes) and `max_len=16` for frame
decode (type varint + length varint = up to 16 bytes).

Rule: crashes are bugs. The engine must never panic on arbitrary input.

### Adding a new fuzz target

1. Add a `fuzz_targets/<name>.rs` file (see existing targets for the one-liner pattern).
2. Add a `[[bin]]` entry to `crates/istok-core/fuzz/Cargo.toml`.
3. **The fuzz `Cargo.toml` must have `[workspace]` at the top** — without it, Cargo
   treats the fuzz crate as part of the parent workspace and `cargo fuzz` fails.
   This is already present; do not remove it when editing the file.

## Anti-patterns

- Real `tokio::time::sleep` or `tokio::net` in unit tests — use the mock harness.
- Asserting only on the absence of errors — also assert on the actual bytes written.
- Tests that pass with wrong FIN placement — always check `fin` explicitly.
