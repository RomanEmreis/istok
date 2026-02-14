# Rust style (istok)

## Hard rules
- No `unwrap()`, `expect()`, `panic!()` in library code.
- Errors are explicit enums; no stringly-typed errors.
- Public items must have doc comments with invariants.

## Error handling
- Prefer `Result<T, Error>` with crate-local `Error`.
- Avoid `anyhow` in libraries; allowed in examples/bins.

## Features
- `std` feature gates std-dependent modules
- `alloc` feature gates heap usage for no_std
- Tokio adapter is `std` only
