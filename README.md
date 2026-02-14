# Istok

Istok is an HTTP/3-first server engine in Rust.

## Goals
- HTTP/3 correctness with clean layering
- Minimal dependencies
- Path to `no_std` for protocol logic
- Deterministic testing via mock transport

## Workspace layout
- `crates/istok-core`: no_std(+alloc) protocol core (codecs, QPACK, H3 state machines)
- `crates/istok-transport`: transport/timer trait boundaries (runtime-agnostic)
- `crates/istok-h3`: H3 engine runtime glue + deterministic mock harness
- `crates/istok-io-tokio`: tokio adapters (std-only)
- `crates/istok-server`: user-facing server API and examples
- `crates/istok-http`: temporary compatibility with `http` crate types

## Status / Milestones
See `docs/milestones.md`.

## Development
- Formatting, linting, tests:
  - `cargo fmt --all`
  - `cargo clippy --all-targets --all-features -D warnings`
  - `cargo test --workspace`

## Testing philosophy
We start with deterministic unit tests (mock QUIC streams), then add real-network integration tests.

## Contributing
Architecture-impacting changes require an RFC in `docs/rfcs/`.
See `AGENTS.md` for workflow and constraints.
