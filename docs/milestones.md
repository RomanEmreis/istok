# Milestones

---

## M0 — Codecs

**Status:** done

### Scope

Varint and H3 frame codecs with tests. QPACK excluded (see M2).

### Acceptance tests

- [x] varint codec + tests
- [x] H3 frame codec + tests
- [x] SETTINGS payload encoding (M0: empty) + tests
- [x] fuzz target(s) for frame/varint parsing (optional in v0 CI)

### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

### no_std / min-deps notes

- QPACK moved out of M0 (see M2).

---

## M1 — H3 state machine over mock transport

**Goal:** A deterministic, test-driven H3 engine that can complete a minimal HTTP
request/response flow over a mock QUIC transport.

---

### M1.0 — Harness + bootstrap

**Status:** done

#### Scope

Deterministic mock harness, engine boot, control stream, owned write path.

#### Acceptance tests

- [x] deterministic mock harness (script-driven)
- [x] engine boot opens local control stream + sends SETTINGS (no magic bytes)
- [x] owned write path supported in mock
- [x] StreamWriteOwned path integrated through transport + mock

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

---

### M1.1 — Inbound control stream type + SETTINGS

**Status:** done

#### Scope

Parse and accept peer-initiated control stream; parse SETTINGS frame.

#### Acceptance tests

- [x] parse peer-initiated uni stream type (varint)
- [x] accept peer control stream
- [x] parse frames on control stream (at least SETTINGS with len=0)
- [x] peer control stream sends empty SETTINGS → accepted, no close

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

---

### M1.2 — Minimal buffering / incremental parsing

**Status:** done

#### Scope

Handle fragmented input across multiple StreamReadable events.

#### Acceptance tests

- [x] handle fragmented input across multiple StreamReadable events
- [x] stream type varint split across events
- [x] frame header split across events
- [x] SETTINGS payload split across events (even if len=0, test structure)

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

---

### M1.3 — One request stream happy-path (HEADERS only, no QPACK yet)

**Status:** done

#### Scope

Accept one bidi stream as request, receive HEADERS, send minimal response HEADERS
with FIN. No QPACK.

#### Acceptance tests

- [x] request stream handling without QPACK (use placeholder header representation)
- [x] accept one bidi stream as "request"
- [x] receive HEADERS frame (payload treated as opaque bytes for now)
- [x] finish response stream (send FIN)
- [x] tolerate FIN-only readable after completion (no close)
- [x] enforce max HEADERS payload (16 KiB)
- [x] cap early buffering before control stream (MAX_EARLY_REQUEST_BUFFER)
- [x] produce a minimal response:
  - [x] send HEADERS frame only (opaque bytes), fin=true
- [x] happy-path: HEADERS in → HEADERS out

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

---

### M1.4 — Error paths

**Status:** done

#### Scope

Protocol error handling for malformed frames, unexpected stream types, and control
stream violations.

#### Acceptance tests

- [x] malformed varint / malformed frame header → close with appropriate H3 error
- [x] unexpected stream type on uni stream → close
- [x] unexpected first frame on control stream (non-SETTINGS) → close
- [x] tests for each error path

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

---

### M1.5 — Request/control hardening before QPACK

**Status:** done

#### Scope

Strict request stream error paths mirroring control stream strictness;
post-SETTINGS control stream policy.

#### Acceptance tests

- [x] request stream error paths (mirror control strictness):
  - [x] truncated request frame header with fin=true → close with H3_FRAME_ERROR
  - [x] truncated request HEADERS payload with fin=true → close with H3_FRAME_ERROR
  - [x] unexpected first frame on request stream (non-HEADERS) → close with H3_FRAME_UNEXPECTED
  - [x] malformed request frame header (decode error != BufferTooSmall) → close with H3_FRAME_ERROR
- [x] control stream post-SETTINGS policy (explicit + tested):
  - [x] after SETTINGS accepted, receiving any additional frame on control stream → close with H3_FRAME_UNEXPECTED (until M2+)
  - [x] tolerate FIN-only empty readable on control stream after SETTINGS (no close) (if your transport can surface it)
- [x] tests for each case (deterministic MockHarness scripts)

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

---

### M1.6 — Response framing semantics (multi-frame, multi-write)

**Status:** done

#### Scope

**Included:**
- Respond with HEADERS + DATA (still opaque bytes, no QPACK yet)
- HEADERS frame written with fin=false
- DATA frame written with fin=true (FIN only on last frame)

**Excluded:**
- Trailers
- Request body handling
- QPACK encoding

#### Acceptance tests

- [x] happy-path: request HEADERS in → response HEADERS out (fin=false) → response DATA out (fin=true)
- [x] response writes are ordered (HEADERS write observed before DATA write)
- [x] tolerate request-side FIN-only empty readable after completion (keep existing behavior)

#### DoD checklist

- [x] All acceptance tests green
- [x] Clippy clean
- [x] Milestones.md updated

#### no_std / min-deps notes

Keep payloads minimal and deterministic (e.g., HEADERS payload `[0x00]`, DATA payload
`[0x01]` or empty if you prefer). No trailers, no request body handling yet.

---

## M2 — QPACK (minimal)

**Status:** in progress

Design: see `docs/rfcs/0002-qpack-minimal.md`

### Scope

Static-table-only QPACK (RFC 9204). No dynamic table. No Huffman encoding.
Replaces the opaque `[0x00]` HEADERS placeholder from M1.x with real wire format.

**Included:**
- Prefix integer codec (RFC 9204 §C.1) — separate from QUIC varint
- Static table (99 entries, RFC 9204 Appendix A): lookup by index + by (name, value)
- Encoder: static-first field representations, literal fallback, no Huffman (H=0)
- Decoder: visitor pattern (`no_std + no_alloc`), static-only, rejects RIC > 0
- Engine: open QPACK encoder/decoder streams on boot; accept inbound QPACK streams
- Integration: real QPACK in response HEADERS; decode inbound request HEADERS
- Fuzz target: `fuzz_qpack_decode`

**Excluded:**
- Dynamic table (deferred indefinitely)
- Huffman encoding/decoding (deferred to M3)

### Acceptance tests

- [ ] prefix integer roundtrip, malformed-rejection, boundary tests
- [ ] static table: correct entries at known indices, lookup hits/misses
- [ ] encoder: exact static hit → Indexed Field Line; name-only hit → Literal With Ref; no hit → Literal Without Ref
- [ ] decoder: all three instruction types decoded correctly (static refs, literals)
- [ ] decoder: rejects RIC > 0 with `DynamicTableRequired`
- [ ] decoder: rejects H=1 string with `HuffmanNotSupported`
- [ ] engine: QPACK streams opened on boot (type bytes 0x02, 0x03)
- [ ] engine: inbound QPACK streams accepted without close
- [ ] integration: request HEADERS decoded; response HEADERS encoded with real fields

### DoD checklist

- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] Fuzz target added and CI fuzz job updated
- [ ] Milestones.md updated

### no_std / min-deps notes

All codec code (`prefix_int`, `qpack::*`) uses `core` only — no `alloc` required.
Visitor-pattern decoder avoids any allocation. Engine still uses `Vec` (unchanged).

---

## M3 — Tokio adapter + hello server

### Scope

- [ ] UDP transport + timer integration
- [ ] example: `h3_hello` serves a static response
- [ ] basic logging/trace hooks (optional)

---

## M4 — Interop

### Scope

- [ ] curl --http3 can fetch /hello
- [ ] document supported subset and known gaps

---

## M5 — Perf & memory

### Scope

- [ ] backpressure strategy
- [ ] memory caps / bounded buffers
- [ ] profiling checklist + baseline numbers
