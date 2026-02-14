# no_std rules (istok)

- `istok-core` should compile with `#![no_std]` + `extern crate alloc` when needed.
- Dependencies must be added with `default-features = false`.
- No `std::` in core unless behind `cfg(feature="std")`.
- Prefer `core` and `alloc` collections; avoid heavy crates unless justified.
