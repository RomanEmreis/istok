# AGENTS.md — Istok

Primary agent guidance for the Istok project. Read this first.
For multi-agent role division and skills, see `.agents/`.

---

## Mission

Istok is a lightweight HTTP/3-first server engine in Rust.
Primary goals: correctness, clean layering, minimal dependencies, and a path to `no_std`.

## Non-goals (for now)

- HTTP/1.1 and HTTP/2 support — H3-first.
- Production-grade QUIC congestion control/recovery — not until a milestone tracks it.
- Full QPACK optimization — correctness only until M2 is explicitly active.
- Custom `http` primitives — we use `http` crate types temporarily via `istok-http`.

---

## Architecture constraints (hard rules)

1. `istok-core` must be `no_std`-friendly: prefer `core` + `alloc`; `std` only behind `feature = "std"`.
2. IO, timers, and crypto are behind traits (`istok-transport`). No tokio types in core.
3. No `unwrap()` / `panic!()` in library code. Panics allowed only in tests and examples.
4. Unsafe code is forbidden unless justified by an RFC in `docs/rfcs/`.
5. Every codec/parser must have unit tests and be fuzz-safe (never panic on arbitrary input).

---

## Project layout

| Crate | Purpose |
|---|---|
| `crates/istok-core` | `no_std`(+alloc) protocol core: codecs, H3 state machines |
| `crates/istok-transport` | transport/timer/crypto trait boundaries (runtime-agnostic) |
| `crates/istok-h3` | H3 engine runtime glue + deterministic mock harness |
| `crates/istok-io-tokio` | tokio adapters (std-only) |
| `crates/istok-server` | user-facing server API and examples |
| `crates/istok-http` | compatibility wrapper around `http` crate types (temporary) |

---

## Commands

Always pass `--locked`. Never omit it.

```sh
cargo test --workspace --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --all
```

---

## Workflow

- Prefer small, vertical PRs: one milestone task or one invariant per PR.
- Architecture-affecting changes require an RFC in `docs/rfcs/` before implementation.
- Scaffold files are intentional — do not delete or rename them without an RFC or explicit instruction.
- Do not modify workspace `members` to fix build issues.
- Do not update `Cargo.lock` unless explicitly asked.
- New dependencies require a short justification comment in the relevant `Cargo.toml`.

---

## Module checklist

Every new module or significant addition must have:
- Top-level doc comment with invariants (`//! ...`)
- Doc comments on all public types and functions
- Explicit error enum (no stringly-typed errors)
- At least one unit test
- No `unwrap`/`expect`/`panic` outside tests

---

## Active milestone

See `docs/milestones.md` for the canonical task list and DoD checklists.
Do not start work on M(n+1) until M(n) DoD is fully checked off.
