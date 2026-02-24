# Milestones

## M0 — Codecs
Definition of Done:
- [x] varint codec + tests
- [x] H3 frame codec + tests
- [x] SETTINGS payload encoding (M0: empty) + tests
- [ ] fuzz target(s) for frame/varint parsing (optional in v0 CI)

Notes:
- QPACK moved out of M0 (see M2).

## M1 — H3 state machine over mock transport
Goal:
A deterministic, test-driven H3 engine that can complete a minimal HTTP request/response flow
over a mock QUIC transport.

### M1.0 — Harness + bootstrap (done)
- [x] deterministic mock harness (script-driven)
- [x] engine boot opens local control stream + sends SETTINGS (no magic bytes)
- [x] owned write path supported in mock
- [x] StreamWriteOwned path integrated through transport + mock

### M1.1 — Inbound control stream type + SETTINGS (next)
- [x] parse peer-initiated uni stream type (varint)
- [x] accept peer control stream
- [x] parse frames on control stream (at least SETTINGS with len=0)
- [x] tests:
  - [x] peer control stream sends empty SETTINGS → accepted, no close

### M1.2 — Minimal buffering / incremental parsing
- [x] handle fragmented input across multiple StreamReadable events
- [x] tests:
  - [x] stream type varint split across events
  - [x] frame header split across events
  - [x] SETTINGS payload split across events (even if len=0, test structure)

### M1.3 — One request stream happy-path (HEADERS only, no QPACK yet)
Scope:
- [x] request stream handling without QPACK (use placeholder header representation)
- [x] accept one bidi stream as “request”
- [x] receive HEADERS frame (payload treated as opaque bytes for now)
- [x] produce a minimal response:
  - [x] send HEADERS frame (opaque bytes) and optional DATA frame
- [x] tests:
  - [x] happy-path: HEADERS in → HEADERS out

### M1.4 — Error paths
- [ ] malformed varint / malformed frame header → close with appropriate H3 error
- [ ] unexpected stream type on uni stream → close
- [ ] unexpected first frame on control stream (non-SETTINGS) → close
- [ ] tests for each error path

## M2 — QPACK (minimal)
- [ ] minimal QPACK decoder/encoder for small header sets
- [ ] integrate into M1.3 HEADERS handling
- [ ] tests for small header sets

## M3 — Tokio adapter + hello server
DoD:
- [ ] UDP transport + timer integration
- [ ] example: `h3_hello` serves a static response
- [ ] basic logging/trace hooks (optional)

## M4 — Interop
DoD:
- [ ] curl --http3 can fetch /hello
- [ ] document supported subset and known gaps

## M5 — Perf & memory
DoD:
- [ ] backpressure strategy
- [ ] memory caps / bounded buffers
- [ ] profiling checklist + baseline numbers
