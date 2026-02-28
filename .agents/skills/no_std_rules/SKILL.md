---
name: no_std Rules
description: Compatibility rules for keeping istok-core and istok-transport free of std — allowed crates, feature gates, and forbidden patterns.
applies_to: [claude, codex]
triggers: [any change to istok-core, istok-transport, adding a dependency, new Cargo.toml]
---

# no_std Rules

## Purpose

`istok-core` and `istok-transport` must remain `no_std`-compatible so the engine
can eventually run on embedded or kernel targets. These rules prevent accidental
`std` contamination.

## Crate-level setup

```rust
// Required at the top of lib.rs for no_std crates
#![no_std]
extern crate alloc; // only if heap allocation is needed
```

## What is allowed by crate

| Crate | `core` | `alloc` | `std` |
|---|---|---|---|
| `istok-core` | yes | behind `alloc` feature | behind `std` feature only |
| `istok-transport` | yes | behind `alloc` feature | behind `std` feature only |
| `istok-h3` | yes | yes | behind `std` feature |
| `istok-io-tokio` | yes | yes | yes (std required) |

## Adding dependencies

- Always add with `default-features = false`:
  ```toml
  some-crate = { version = "x.y", default-features = false }
  ```
- Add a comment explaining why the dependency is needed:
  ```toml
  # varint encoding without std dependency
  leb128 = { version = "0.2", default-features = false }
  ```
- Prefer `core` and `alloc` primitives over external crates when the implementation is small.

## Forbidden in no_std crates

- `std::collections::HashMap` → use `alloc::collections::BTreeMap` or a feature-gated `HashMap`
- `std::io::Write` / `std::io::Read` → define crate-local traits or use `embedded-io`
- `std::string::String` → use `alloc::string::String` (behind `alloc` feature)
- `std::vec::Vec` → use `alloc::vec::Vec` (behind `alloc` feature)
- Any tokio type

## Checking compatibility

```sh
# Verify no_std compiles (no std, no alloc)
cargo build -p istok-core --no-default-features --locked

# Verify with alloc
cargo build -p istok-core --no-default-features --features alloc --locked
```

## Anti-patterns

- Using `use std::…` anywhere in `istok-core` without a `#[cfg(feature = "std")]` guard.
- Adding a dependency that transitively pulls in `std` without checking `default-features`.
- Gating only the import but not the usage (the compiler will catch this, but fix it at the import).
