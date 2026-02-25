# Milestones

## M0 — Codecs
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
- [x] request stream handling without QPACK (use placeholder header representation)
- [x] accept one bidi stream as “request”
- [x] receive HEADERS frame (payload treated as opaque bytes for now)
- [x] finish response stream (send FIN)
- [x] tolerate FIN-only readable after completion (no close)
- [x] enforce max HEADERS payload (16 KiB)
- [x] cap early buffering before control stream (MAX_EARLY_REQUEST_BUFFER)
- [x] produce a minimal response:
  - [x] send HEADERS frame only (opaque bytes), fin=true
- [x] tests:
  - [x] happy-path: HEADERS in → HEADERS out

### M1.4 — Error paths
- [x] malformed varint / malformed frame header → close with appropriate H3 error
- [x] unexpected stream type on uni stream → close
- [x] unexpected first frame on control stream (non-SETTINGS) → close
- [x] tests for each error path

### M1.5 — Request/control hardening before QPACK
- [ ] request stream error paths (mirror control strictness):
  - [ ] truncated request frame header with fin=true → close with H3_FRAME_ERROR
  - [ ] truncated request HEADERS payload with fin=true → close with H3_FRAME_ERROR
  - [ ] unexpected first frame on request stream (non-HEADERS) → close with H3_FRAME_UNEXPECTED
  - [ ] malformed request frame header (decode error != BufferTooSmall) → close with H3_FRAME_ERROR
- [ ] control stream post-SETTINGS policy (explicit + tested):
  - [ ] after SETTINGS accepted, receiving any additional frame on control stream → close with H3_FRAME_UNEXPECTED (until M2+)
  - [ ] tolerate FIN-only empty readable on control stream after SETTINGS (no close) (if your transport can surface it)
- [ ] tests for each case (deterministic MockHarness scripts)

### M1.6 — Response framing semantics (multi-frame, multi-write)
Scope:
- [ ] respond with HEADERS + DATA (still opaque bytes, no QPACK yet)
  - [ ] write HEADERS frame first with fin=false
  - [ ] write DATA frame second with fin=true (FIN only on last frame)
- [ ] deterministic tests:
  - [ ] happy-path: request HEADERS in → response HEADERS out (fin=false) → response DATA out (fin=true)
  - [ ] response writes are ordered (HEADERS write observed before DATA write)
  - [ ] tolerate request-side FIN-only empty readable after completion (keep existing behavior)

Notes:
- Keep payloads minimal and deterministic (e.g., HEADERS payload [0x00], DATA payload [0x01] or empty if you prefer).
- No trailers, no request body handling yet.

## M2 — QPACK (minimal)
- [ ] minimal QPACK decoder/encoder for small header sets
- [ ] integrate into M1.3 HEADERS handling
- [ ] tests for small header sets

## M3 — Tokio adapter + hello server
- [ ] UDP transport + timer integration
- [ ] example: `h3_hello` serves a static response
- [ ] basic logging/trace hooks (optional)

## M4 — Interop
- [ ] curl --http3 can fetch /hello
- [ ] document supported subset and known gaps

## M5 — Perf & memory
- [ ] backpressure strategy
- [ ] memory caps / bounded buffers
- [ ] profiling checklist + baseline numbers
