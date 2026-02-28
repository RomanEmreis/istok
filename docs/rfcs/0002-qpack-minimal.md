# RFC 0002: QPACK — Minimal Static-Table Implementation (M2)

## Summary

Replace the opaque `[0x00]` HEADERS placeholder used throughout M1.x with real
QPACK-encoded header blocks. Scope is deliberately narrow: **static table only**,
no dynamic table, no Huffman encoding/decoding. This is sufficient to produce and
consume well-formed HTTP/3 HEADERS frames and to unblock interop testing (M4).

---

## Motivation

M1.x treats HEADERS payloads as opaque bytes. The engine can open control streams,
handle framing, and sequence writes correctly, but it speaks no actual HTTP
semantics. M2 fixes that at the minimum viable level:

- A real peer (curl, browsers) sends QPACK-encoded request headers.
- Our response HEADERS frame must carry valid QPACK, not `[0x00]`.
- Without this, M3 (Tokio adapter) cannot produce a response any client can use.

Static-table-only QPACK is a well-defined, stable subset of RFC 9204: Required
Insert Count = 0, S bit = 0 for all field representations referencing the static
table. No dynamic table instructions are ever sent or expected. This keeps M2
tractable while delivering real interoperability.

---

## Design

### 1. New codec: prefix integer (RFC 9204 §C.1)

QPACK uses its own integer encoding — **not** the QUIC varint from M0. A prefix
integer has N prefix bits belonging to the surrounding instruction byte, followed
by up to 5 extension bytes. This is the same scheme as HPACK (RFC 7541 §5.1).

New module: `istok-core/src/codec/prefix_int.rs`

```rust
/// Decode a prefix integer from `input`.
/// `prefix_bits` is in 1..=8.
/// Returns (value, bytes_consumed).
pub fn decode(input: &[u8], prefix_bits: u8) -> Result<(u64, usize), PrefixIntError>;

/// Encode `value` into the low `prefix_bits` of `first_byte`, appending
/// extension bytes to `out` as needed.
/// Returns bytes_written (including the first byte, which caller pre-fills
/// with the instruction bits).
pub fn encode(value: u64, prefix_bits: u8, out: &mut [u8]) -> Result<usize, PrefixIntError>;

pub enum PrefixIntError {
    BufferTooSmall,
    ValueTooLarge,   // > 2^30 - 1 (practical cap; real limit is 2^62)
    Overflow,        // extension chain too long
}
```

This is the only truly new codec M2 introduces. Everything else builds on it.

### 2. Static table (`istok-core/src/qpack/static_table.rs`)

The 99-entry static table from RFC 9204 Appendix A, stored as `&'static [u8]`
pairs, indexed 0–98. Two lookup functions:

```rust
/// Return (name, value) for a static table entry.
pub fn entry(index: usize) -> Option<(&'static [u8], &'static [u8])>;

/// Find the best static table match for (name, value).
/// Returns Some((index, value_matches)) — value_matches = true means exact hit.
pub fn lookup(name: &[u8], value: &[u8]) -> Option<(usize, bool)>;
```

No allocation. Entire table is `const`/`static` data.

### 3. Encoder (`istok-core/src/qpack/encoder.rs`)

Writes a complete QPACK header block (Required Insert Count prefix + Delta Base
prefix + field representations) into a caller-supplied `&mut [u8]`.

Strategy per field line (static-table-first, no Huffman):

| Condition | Wire format |
|---|---|
| exact `(name, value)` hit in static table | Indexed Field Line — `0b11` + 6-bit index |
| name-only hit in static table | Literal With Static Name Ref — `0b0001` + 4-bit index + literal value |
| no hit | Literal Without Name Ref — `0b001` + literal name + literal value |

Literal strings use H=0 (no Huffman), length prefix, then raw bytes.

```rust
pub struct HeaderField<'a> {
    pub name:  &'a [u8],
    pub value: &'a [u8],
}

/// Encode `fields` into a QPACK header block.
/// Always produces RIC=0 (static-only). No allocation.
pub fn encode(fields: &[HeaderField<'_>], out: &mut [u8]) -> Result<usize, EncodeError>;

pub enum EncodeError {
    BufferTooSmall,
    ValueTooLarge,
}
```

### 4. Decoder (`istok-core/src/qpack/decoder.rs`)

Visitor pattern — `no_std + no_alloc` compatible. Caller supplies a closure
that receives decoded `(name, value)` byte slices. Slices reference the input
buffer for static-indexed names; literal strings are returned as-is from input.

```rust
/// Decode a QPACK header block, calling `visitor` for each decoded field.
/// Rejects any block with RIC > 0 (dynamic table not supported).
pub fn decode<F>(input: &[u8], visitor: F) -> Result<(), DecodeError>
where
    F: FnMut(&[u8], &[u8]);

pub enum DecodeError {
    UnexpectedEnd,
    InvalidInstruction,
    InvalidIndex,           // index outside static table range
    DynamicTableRequired,   // RIC > 0 — not supported in M2
    HuffmanNotSupported,    // H=1 string — not supported in M2
    StringTooLong,          // length prefix exceeds remaining input
}
```

Handled instruction types:
- `0b1_xxxxxxx` — Indexed Field Line; static if S=1 (0b11), reject if S=0 (0b10)
- `0b0001_xxxx` — Literal With Name Reference; static if N=0 (0b0001)
- `0b001_xxxxx` — Literal Without Name Reference

Unrecognised instruction types → `InvalidInstruction`.

### 5. QPACK streams in the engine

RFC 9114 §6.2.1 requires both endpoints to open a QPACK encoder stream
(type `0x02`) and QPACK decoder stream (type `0x03`). For static-only QPACK,
these streams carry **no data after the stream-type byte** — but they must exist.

Changes to `h3_engine.rs` `Boot` handler:

```
OpenUni  stream_type=0x02   (QPACK encoder stream)
OpenUni  stream_type=0x03   (QPACK decoder stream)
```

Inbound QPACK streams from the peer (`0x02`, `0x03`) are currently rejected with
`H3_GENERAL_PROTOCOL_ERROR`. M2 changes `InboundUniState::Type` to accept and
silently discard these stream types (static-only — no instructions to process).

### 6. Integration in the request/response path

| Location | Before M2 | After M2 |
|---|---|---|
| `RESPONSE_HEADERS_PAYLOAD` | `[0x00]` (1 opaque byte) | `qpack::encode(&[(:status, "200"), (content-type, "text/plain")], &mut buf)` |
| Inbound HEADERS payload | ignored (opaque) | `qpack::decode(payload, \|name, value\| { … })` |
| QPACK decode error | n/a | `close_request_with(H3_QPACK_DECOMPRESSION_FAILED)` |

New error codes in `consts.rs` (RFC 9204 §8.1):

```rust
pub const H3_QPACK_DECOMPRESSION_FAILED: u64 = 0x0200;
pub const H3_QPACK_ENCODER_STREAM_ERROR: u64 = 0x0201;
pub const H3_QPACK_DECODER_STREAM_ERROR: u64 = 0x0202;
```

### 7. Feature flag impact

| Concern | Decision |
|---|---|
| `no_std` | All of `qpack::*` uses `core` only — no `std` imports |
| `no_alloc` | Encoder takes `&mut [u8]`; decoder uses visitor — zero allocation |
| `alloc` feature | Not required by M2 codec code; engine still uses `Vec` (unchanged) |

### 8. Fuzz target

Add `fuzz_targets/fuzz_qpack_decode.rs` after M2.2 is stable:

```rust
fuzz_target!(|data: &[u8]| {
    let _ = qpack::decode(data, |_, _| {});
});
```

---

## Alternatives

**Full dynamic table (RFC 9204 §3.2.4):** Requires per-stream acknowledgment
protocol, encoder/decoder instruction streams carrying real data, and a blocking
hazard on large table sizes. Correct but out of scope for M2; deferred
indefinitely until interop testing proves necessity.

**Huffman decoding in M2:** Straightforward to implement (256-entry decoding
tree, ~500 LoC). Excluded because: (a) real peers may or may not send Huffman
strings in practice, and (b) static-table-only can reach full interop without it
since most common request headers are exact static-table hits (`:method GET`,
`:path /`, `:scheme https`, etc.). Added to M3's "nice to have" list.

**Third-party QPACK crate (`h3`/`h3i`):** Violates the min-deps goal. Both
crates pull in large dependency trees incompatible with `no_std`.

**Fixed-size output array instead of visitor:** Simpler API but arbitrary limit
— a 32-header cap would be invisible to the caller until it silently truncates.
Visitor is strictly safer and has the same `no_alloc` footprint.

---

## Risks

| Risk | Mitigation |
|---|---|
| Huffman-encoded strings from real peers → `HuffmanNotSupported` decode error | Document as known gap; M3 prerequisite for full curl interop |
| Prefix-integer overflow attack (very long extension chain) | Hard cap: reject after 5 extension bytes (max value ~2^35; well within u64) |
| Wrong prefix bit count producing incorrect field representations | Codec tests with RFC 9204 Appendix B example vectors |
| DynamicTableRequired closes connection unexpectedly against liberal peers | Expected: RFC 9204 §2.1.3 requires static-only peers to reject RIC > 0 |

---

## Rollout Plan

| Sub-milestone | Deliverables |
|---|---|
| **M2.0** | `codec/prefix_int.rs` — encode/decode with roundtrip + malformed tests |
| **M2.1** | `qpack/static_table.rs` — 99 entries, `entry()` + `lookup()`, tested |
| **M2.2** | `qpack/encoder.rs` — static-first encoding, no Huffman, tested |
| **M2.3** | `qpack/decoder.rs` — visitor decode, static-only, tested; fuzz target |
| **M2.4** | Integration — replace `[0x00]` placeholder; open QPACK streams on boot; accept inbound QPACK streams; `milestones.md` updated |
