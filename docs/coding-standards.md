# Coding Standards

- No panics/unwraps in library code.
- All public types/functions documented.
- Prefer explicit error enums.
- Default features off for deps; no_std core must not pull std implicitly.
- Unsafe forbidden unless RFC-approved.
