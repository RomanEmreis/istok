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

**Goal:** Replace the opaque `[0x00]` HEADERS placeholder from M1.x with
real QPACK-encoded header blocks. Static table only — no dynamic table, no
Huffman. Full design in `docs/rfcs/0002-qpack-minimal.md`.

---

### M2.0 — Prefix integer codec

**Status:** pending

#### Scope

New codec module `istok-core/src/codec/prefix_int.rs`. QPACK uses a different
integer encoding than QUIC varint (RFC 9204 §C.1, same scheme as HPACK RFC 7541
§5.1): N prefix bits are owned by the surrounding instruction byte; overflow
extends into subsequent bytes with a 0x80 continuation flag.

#### Acceptance tests

- [ ] roundtrip: values 0, 1, prefix_max−1, prefix_max, prefix_max+1, 2^14, 2^21, 2^28 across prefix widths 1–8
- [ ] decode: known RFC 9204 Appendix B.1 example vectors produce correct values
- [ ] decode: empty input → `BufferTooSmall`
- [ ] decode: extension chain truncated mid-byte → `BufferTooSmall`
- [ ] decode: extension chain exceeds 5 bytes → `Overflow`
- [ ] encode: `BufferTooSmall` when output slice too short
- [ ] encode: values beyond practical cap → `ValueTooLarge`

#### DoD checklist

- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] Milestones.md updated

#### no_std / min-deps notes

`core` only. No allocation. `encode` writes into caller-supplied `&mut [u8]`.

---

### M2.1 — Static table

**Status:** pending

#### Scope

`istok-core/src/qpack/static_table.rs` — the 99-entry static table from RFC
9204 Appendix A as `const`/`static` byte-slice pairs, plus two lookup functions.

```
entry(index) -> Option<(&'static [u8], &'static [u8])>
lookup(name, value) -> Option<(usize, bool)>   // (index, exact_value_match)
```

#### Acceptance tests

- [ ] `entry(0)` → `(b":authority", b"")`
- [ ] `entry(1)` → `(b":path", b"/")`
- [ ] `entry(25)` → `(b":status", b"200")`
- [ ] `entry(98)` → last valid entry per RFC 9204 Appendix A
- [ ] `entry(99)` → `None` (out of range)
- [ ] `lookup(b":method", b"GET")` → exact hit at known index
- [ ] `lookup(b":status", b"200")` → exact hit
- [ ] `lookup(b":status", b"999")` → name-only hit (index to `:status` entry, `value_matches=false`)
- [ ] `lookup(b"x-custom", b"val")` → `None`

#### DoD checklist

- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] Milestones.md updated

#### no_std / min-deps notes

Entire table is `&'static [u8]` pairs in a `const` array. No `alloc`, no `std`.

---

### M2.2 — Encoder

**Status:** pending

#### Scope

`istok-core/src/qpack/encoder.rs` — encodes a slice of `HeaderField` pairs into
a QPACK header block. Always produces RIC=0 (static-only). No Huffman (H=0).

Field line selection (static-table-first):

| Condition | Wire representation |
|---|---|
| exact `(name, value)` in static table | Indexed Field Line `0b11` + 6-bit index |
| name-only hit | Literal With Static Name Ref `0b0001` + 4-bit index + literal value |
| no hit | Literal Without Name Ref `0b001` + literal name + literal value |

Writes: Required Insert Count (0) + S=0 Delta Base (0) + field lines.

#### Acceptance tests

- [ ] encode `[(:status, "200")]` → exact Indexed Field Line at static index 25
- [ ] encode `[(:status, "999")]` → Literal With Static Name Ref at `:status` index
- [ ] encode `[("x-custom", "val")]` → Literal Without Name Ref
- [ ] encode multiple fields → all present in output, order preserved
- [ ] output always starts with two zero bytes (RIC=0, Delta Base=0 for 1-byte prefixes)
- [ ] `BufferTooSmall` when output slice cannot fit encoded block
- [ ] roundtrip: encoded output fed to decoder (M2.3) produces original fields

#### DoD checklist

- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] Milestones.md updated

#### no_std / min-deps notes

No allocation. Takes `&mut [u8]` output buffer. `HeaderField<'a>` borrows caller data.

---

### M2.3 — Decoder

**Status:** pending

#### Scope

`istok-core/src/qpack/decoder.rs` — decodes a QPACK header block using a
visitor/callback pattern. Static-table only: rejects RIC > 0 and Huffman strings.

Handled instruction types (RFC 9204 §3.2):
- `0b11xxxxxx` — Indexed Field Line, static (S=1); reject S=0 (dynamic)
- `0b0001xxxx` — Literal With Name Reference, static; reject dynamic
- `0b001xxxxx` — Literal Without Name Reference

Error cases that must close with `H3_QPACK_DECOMPRESSION_FAILED` when wired
into the engine (M2.4): `DynamicTableRequired`, `InvalidIndex`, `UnexpectedEnd`.
`HuffmanNotSupported` is also a connection error at the engine level.

Fuzz target `fuzz_qpack_decode` added after this sub-milestone is green.

#### Acceptance tests

- [ ] decode Indexed Field Line (static) → correct `(name, value)` from static table
- [ ] decode Literal With Static Name Ref → correct name from table, literal value
- [ ] decode Literal Without Name Ref → both name and value from literal bytes
- [ ] decode multiple fields in sequence → visitor called once per field, in order
- [ ] RIC ≠ 0 in prefix → `DynamicTableRequired`
- [ ] Indexed Field Line with S=0 (dynamic) → `DynamicTableRequired`
- [ ] Literal With Name Ref with dynamic flag → `DynamicTableRequired`
- [ ] index out of static table range → `InvalidIndex`
- [ ] H=1 string length prefix → `HuffmanNotSupported`
- [ ] truncated field mid-parse → `UnexpectedEnd`
- [ ] empty input (after two-byte prefix) → visitor never called, `Ok(())`
- [ ] roundtrip with M2.2 encoder output → original fields recovered

#### DoD checklist

- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] Fuzz target `fuzz_qpack_decode` added to `crates/istok-core/fuzz/`
- [ ] `fuzz_qpack_decode` added to CI fuzz job (30 s run)
- [ ] Milestones.md updated

#### no_std / min-deps notes

Visitor closure — zero allocation. Input references are borrowed; static-table
names are `&'static [u8]`. Fully `no_std + no_alloc`.

---

### M2.4 — Engine integration

**Status:** pending

#### Scope

Wire the codec work from M2.0–M2.3 into `h3_engine.rs` and the mock harness.
Replace placeholder bytes with real QPACK. Open QPACK streams on boot. Accept
inbound QPACK streams without closing the connection.

**Changes:**

1. `Boot` handler — open QPACK encoder and decoder streams (type bytes `0x02`,
   `0x03`); stream-type varint written, no further data (static-only).
2. `InboundUniState::Type` — accept stream types `0x02` and `0x03`; transition
   to a new `InboundUniState::Ignored` variant that discards all further data.
3. Response HEADERS — replace `RESPONSE_HEADERS_PAYLOAD: [u8; 1] = [0x00]` with
   `qpack::encode(&[(:status, "200"), (content-type, "text/plain")], &mut buf)`.
4. Inbound HEADERS payload — call `qpack::decode(payload, |name, value| { … })`;
   on `DynamicTableRequired`, `InvalidIndex`, `HuffmanNotSupported`, or
   `UnexpectedEnd` → `close_request_with(H3_QPACK_DECOMPRESSION_FAILED)`.
5. New constants in `consts.rs`: `H3_QPACK_DECOMPRESSION_FAILED = 0x0200`,
   `H3_QPACK_ENCODER_STREAM_ERROR = 0x0201`, `H3_QPACK_DECODER_STREAM_ERROR = 0x0202`.

#### Acceptance tests

- [ ] on `Boot`: three unidirectional stream opens emitted (control, QPACK encoder, QPACK decoder)
- [ ] QPACK encoder stream write carries type byte `0x02` only
- [ ] QPACK decoder stream write carries type byte `0x03` only
- [ ] inbound stream type `0x02` accepted, no close command emitted
- [ ] inbound stream type `0x03` accepted, no close command emitted
- [ ] data on an accepted QPACK stream after type byte → silently discarded, no close
- [ ] response HEADERS payload decodes (via M2.3) to `:status 200` and `content-type`
- [ ] inbound HEADERS with RIC > 0 → `CloseConnection(H3_QPACK_DECOMPRESSION_FAILED)`
- [ ] inbound HEADERS with out-of-range static index → `CloseConnection(H3_QPACK_DECOMPRESSION_FAILED)`
- [ ] happy-path: static-encoded request HEADERS in → real QPACK response HEADERS + DATA out

#### DoD checklist

- [ ] All acceptance tests green
- [ ] Clippy clean
- [ ] `cargo test --workspace --locked` green
- [ ] Milestones.md updated

#### no_std / min-deps notes

Engine continues to use `alloc::vec::Vec` (unchanged). Codec calls are
allocation-free; output buffers are stack-allocated `[u8; N]` slices.

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
