# AGENTS.md — Istok

## Mission
Istok is a lightweight HTTP/3-first server engine in Rust.
Primary goals: correctness, clean layering, minimal dependencies, and a path to no_std.

## Non-goals (for now)
- HTTP/1.1 and HTTP/2 support (H3-first).
- Production-grade QUIC congestion control/recovery in v0 unless tracked by a milestone.
- Full QPACK optimization and tuning (start with correctness).
- Custom `http` primitives (we use `http` crate types temporarily via `istok-http`).

## Architecture rules (hard constraints)
1. `istok-core` must be `no_std`-friendly (prefer `core` + `alloc`). `std` only behind `feature = "std"`.
2. IO, timers, and crypto are behind traits (`istok-transport`). No tokio types in core.
3. No `unwrap()` / panics in library code. Use structured errors. Panics allowed only in tests/examples.
4. Unsafe code is forbidden unless justified by an RFC in `docs/rfcs/`.
5. Every codec/parser must have unit tests + property tests (where feasible) and be fuzzable.

## Project layout
- crates/istok-core: H3 + QPACK state machines and codecs (no_std + alloc)
- crates/istok-transport: transport/timer/crypto traits (no runtime binding)
- crates/istok-h3: runtime glue for H3 over transport traits
- crates/istok-io-tokio: tokio adapters (UDP + timers)
- crates/istok-server: public server engine API, examples, integration tests
- crates/istok-http: compatibility wrapper around `http` crate types (temporary)

## Milestones (vertical slices)
M0: Codecs (varint, frames) + minimal QPACK encode/decode + fuzz targets
M1: H3 connection state machine over mock transport (deterministic tests)
M2: Tokio UDP transport + "one request -> one response" demo server
M3: Basic interop with curl --http3 and a small h3spec subset
M4: Backpressure, memory discipline, performance profiling & metrics hooks

## Workflow
- Prefer small PRs: one milestone task or one invariant.
- Every new module includes:
  - top-level doc comment with invariants
  - at least one unit test
  - error model (no panics)
- CI runs: fmt, clippy, tests. Optional: fuzz, miri (later).

## Commands
- `cargo test -p istok-core`
- `cargo test -p istok-h3`
- `cargo run -p istok-server --example h3_hello`

## Decision log
Architecture-affecting decisions must be captured in `docs/rfcs/` before large refactors.

## Scaffolding is intentional
- Do NOT delete or collapse placeholder modules/files (even if currently unused).
- Keep the planned crate/module structure stable.
- If something is unused, prefer leaving it in place with a short comment, or gate it behind a feature.
- Any removal/renaming of scaffold files requires an RFC or explicit instruction.

## Reproducible builds & network
- Network access is available, but keep builds reproducible.
- Never set `RUSTUP_TOOLCHAIN` and never use `cargo +<toolchain>`.
- Do not modify workspace members to “fix” build issues.
- Always run Cargo commands with `--locked` (tests, clippy, build).
- Do not update `Cargo.lock` unless explicitly requested.
- New dependencies require a short justification and must be minimal.
- `istok-core` must remain `no_std`-friendly: avoid `std`-only deps there; prefer `default-features = false`.