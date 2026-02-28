# CLAUDE.md — Istok

Istok is an HTTP/3-first server engine in Rust.
Goals: correctness, clean layering, minimal dependencies, path to `no_std`.

---

## Active milestone

**M1.6 — Response framing semantics (multi-frame, multi-write)**

Scope (see `docs/milestones.md#m16`):
- Respond with HEADERS + DATA (opaque bytes, no QPACK yet)
- HEADERS frame written with `fin=false`, DATA frame with `fin=true`
- Deterministic MockHarness tests for ordering and FIN behavior

Do not start M2 work unless M1.6 is fully checked off.

---

## Build and test commands

Always pass `--locked`. Never omit it.

```sh
# Run all tests
cargo test --workspace --locked

# Run tests for a specific crate
cargo test -p istok-core --locked
cargo test -p istok-h3 --locked

# Lint (must be clean — warnings are errors)
cargo clippy --all-targets --all-features --locked -- -D warnings

# Format check
cargo fmt --all -- --check

# Format (apply)
cargo fmt --all
```

---

## Hard rules — never violate these

- **No `unwrap()`, `expect()`, `panic!()` in library code.** Tests and examples are the only exception.
- **No `anyhow` or `thiserror` in library crates.** Use explicit enums. `anyhow` is allowed in examples/bins only.
- **No `std::` in `istok-core` or `istok-transport`** unless behind `#[cfg(feature = "std")]`.
- **No tokio types in `istok-core` or `istok-transport`.** Tokio lives only in `istok-io-tokio`.
- **Do not modify `Cargo.lock`** unless explicitly asked.
- **Do not add dependencies** without a short justification comment in the relevant `Cargo.toml`.
- **Do not delete or rename scaffold files** (empty `lib.rs`, placeholder modules). Leave them with a comment if needed.
- **Do not change workspace `members`** to fix build issues.

---

## Crate boundaries

| Crate | Allowed | Forbidden |
|---|---|---|
| `istok-core` | `core`, `alloc` (behind feature) | `std`, tokio, any heavy dep |
| `istok-transport` | `core`, `alloc` | tokio, `std` without feature gate |
| `istok-h3` | `istok-core`, `istok-transport` | tokio, `std` without feature gate |
| `istok-io-tokio` | tokio, `std` | `no_std` assumptions |
| `istok-server` | all crates | — |
| `istok-http` | `http` crate | heavy deps |

When adding code, verify it lands in the correct crate before writing.

---

## Where things live

```
crates/istok-core/src/
  codec/        — varint, frame encoding/decoding
  h3/           — H3 state machine, stream types
  error.rs      — core error enums

crates/istok-h3/src/
  engine.rs     — Engine, EngineCommand, EngineEvent, TimerId
  h3_engine.rs  — H3Engine (connection runtime)
  mock/         — MockHarness, script-driven deterministic tests

docs/
  milestones.md — canonical milestone checklist (source of truth)
  rfcs/         — architecture decisions; create RFC before large refactors
```

---

## Checklist for every new module

Before finishing any new module or significant addition:

- [ ] Top-level doc comment with invariants (`//! ...`)
- [ ] All public types and functions have doc comments
- [ ] Explicit error enum (no stringly-typed errors)
- [ ] At least one unit test
- [ ] No `unwrap`/`expect`/`panic` outside tests
- [ ] `no_std` compatibility preserved in `istok-core` / `istok-transport`

---

## Writing tests (M1.x pattern)

Tests for H3 engine behavior use `MockHarness` in `crates/istok-h3/src/mock`.
The harness is script-driven and deterministic — no real sockets, no timers.

Typical test structure:
1. Construct a `MockHarness` with scripted incoming stream events
2. Drive the engine until it halts or produces expected writes
3. Assert on captured outgoing writes (frame bytes, FIN flags, ordering)

Keep test payloads minimal and deterministic (e.g., HEADERS payload `[0x00]`, DATA payload `[0x01]`).

---

## Error model

- H3 error codes live in `istok-core` (e.g., `H3_FRAME_ERROR`, `H3_FRAME_UNEXPECTED`).
- Transport-level close reasons map from H3 codes — do not invent new close reasons ad-hoc.
- When in doubt about which error code to use, consult RFC 9114 §8.

---

## Before submitting changes

1. `cargo fmt --all` — no diff
2. `cargo clippy --all-targets --all-features --locked -- -D warnings` — zero warnings
3. `cargo test --workspace --locked` — all green
4. Milestone checklist in `docs/milestones.md` updated if a task is completed

---

## What NOT to do (common agent mistakes)

- Do not add `tracing`, `log`, `anyhow`, `thiserror` to library crates without being asked.
- Do not collapse or merge placeholder crates/modules to "clean up".
- Do not use `cargo +<toolchain>` or set `RUSTUP_TOOLCHAIN`.
- Do not propose HTTP/1.1 or HTTP/2 work — out of scope for now.
- Do not start QPACK optimization — correctness only until M2 is explicitly active.
- Do not suggest extracting shared logic into a new crate without an RFC.