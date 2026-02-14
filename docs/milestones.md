# Milestones

## M0 — Codecs + minimal QPACK
Definition of Done:
- varint codec + tests
- H3 frame codec + tests
- minimal QPACK encoder/decoder for small header sets
- fuzz target(s) for frame parsing (optional in v0 CI)

## M1 — H3 state machine over mock transport
DoD:
- deterministic mock transport harness
- control streams + one request stream happy-path
- error paths tested (malformed frames, unexpected stream types)

## M2 — Tokio adapter + hello server
DoD:
- UDP transport + timer integration
- example: `h3_hello` serves a static response
- basic logging/trace hooks (optional)

## M3 — Interop
DoD:
- curl --http3 can fetch /hello
- document supported subset and known gaps

## M4 — Perf & memory
DoD:
- backpressure strategy
- memory caps / bounded buffers
- profiling checklist + baseline numbers
