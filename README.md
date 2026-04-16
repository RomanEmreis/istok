# Istok

A deterministic HTTP/3 engine in Rust, built around a test-first state machine and explicit protocol semantics.

## Why

Most HTTP/3 implementations are hard to reason about and test due to implicit state and non-deterministic behavior.

Istok takes a different approach:
- deterministic state machine
- scriptable transport (mock-first testing)
- explicit protocol handling

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

**Pre-alpha** — active milestone: **M2 — QPACK (minimal)**

Core HTTP/3 engine is implemented and validated against a deterministic mock transport.

### Progress

- ✅ M0–M1 complete:
  - H3 codecs, state machine, control/request streams
  - deterministic test harness
  - full error-path coverage
  - response framing (HEADERS + DATA)

- 🚧 M2 in progress:
  - minimal QPACK (static table only)

## Milestones

See [`docs/milestones.md`](docs/milestones.md) for the full roadmap.

## Development
- Formatting, linting, tests:
  - `cargo fmt --all`
  - `cargo clippy --all-targets --all-features --locked -- -D warnings`
  - `cargo test --workspace --locked`

## Testing philosophy

We start with deterministic unit tests (mock QUIC streams), then move to real-network integration.

## Contributing
Architecture-impacting changes require an RFC in [`docs/rfcs/`](docs/rfcs/).
See [`.agents/AGENTS.md`](.agents/AGENTS.md) for agent roles and workflow constraints.
