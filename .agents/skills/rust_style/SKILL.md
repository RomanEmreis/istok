---
name: Rust Style
applies_to: [claude, codex]
triggers: [any code edit, new module, new crate, PR review]
---

# Rust Style

## Purpose

Enforces Istok's Rust coding conventions across all crates to keep the codebase
consistent, `no_std`-compatible where required, and free of hidden panics.

## Hard rules

- No `unwrap()`, `expect()`, or `panic!()` in library code. Tests and examples are the only exception.
- No `anyhow` or `thiserror` in library crates. Use explicit error enums.
- Errors are crate-local enums — no stringly-typed errors, no `Box<dyn Error>` in public API.
- All public types and functions must have doc comments that state invariants.

## Error handling

```rust
// Good — explicit enum, returned from fn
pub enum FrameError {
    UnexpectedFrame,
    VarintOverflow,
}

// Bad — opaque, loses information at boundaries
fn parse(buf: &[u8]) -> Result<Frame, Box<dyn std::error::Error>> { … }
```

## Feature gates

- `std` feature gates any module that uses `std::`.
- `alloc` feature gates heap usage so `no_std` targets can opt in.
- Tokio types are confined to `istok-io-tokio` — never bleed into `istok-core` or `istok-transport`.

## Style conventions

- Prefer `match` over chains of `if let` for exhaustiveness checking.
- Derive `Debug` on all public types; derive `PartialEq` when it's meaningful for tests.
- Keep `impl` blocks ordered: `pub fn` → `pub(crate) fn` → `fn`.

## Anti-patterns

- `unwrap()` as "this can't fail" — it can, under adversarial input.
- Generic `Error` variants that swallow context (`Io(std::io::Error)` is fine; `Other(String)` is not).
- Mixing concern layers: codec logic in transport, transport logic in H3 state machine.
