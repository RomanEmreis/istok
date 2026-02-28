# Istok

Istok is an HTTP/3-first server engine in Rust.

## Goals
- HTTP/3 correctness with clean layering
- Minimal dependencies
- Path to `no_std` for protocol logic
- Deterministic testing via mock transport

## Workspace layout
- `crates/istok-core`: no_std(+alloc) protocol core (codecs, H3 state machines)
- `crates/istok-transport`: transport/timer trait boundaries (runtime-agnostic)
- `crates/istok-h3`: H3 engine runtime glue + deterministic mock harness
- `crates/istok-io-tokio`: tokio adapters (std-only)
- `crates/istok-server`: user-facing server API and examples
- `crates/istok-http`: temporary compatibility with `http` crate types

## Status

**pre-alpha** — active milestone: M2 — QPACK (minimal)

**Completed:**
- M0: varint codec, H3 frame codec, SETTINGS encoding
- M1.0: deterministic mock harness, engine boot, control stream
- M1.1: inbound control stream type + SETTINGS parsing
- M1.2: incremental / fragmented stream parsing
- M1.3: request stream happy-path (opaque HEADERS in → HEADERS out)
- M1.4: protocol error paths (malformed frames, unexpected stream types)
- M1.5: request/control stream hardening
- M1.6: HEADERS + DATA response framing; correct FIN placement; ordering tests

## Milestones
See `docs/milestones.md`.

## Development
- Formatting, linting, tests:
  - `cargo fmt --all`
  - `cargo clippy --all-targets --all-features --locked -- -D warnings`
  - `cargo test --workspace --locked`

## Testing philosophy
We start with deterministic unit tests (mock QUIC streams), then add real-network integration tests.

## Contributing
Architecture-impacting changes require an RFC in `docs/rfcs/`.
See `.agents/AGENTS.md` for agent roles and workflow constraints.
