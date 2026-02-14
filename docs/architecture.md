# Architecture

## Key principles
1) Core logic is IO-agnostic and (eventually) no_std.
2) Transport is a trait boundary.
3) Tests start with deterministic mock transport.

## Data flow (high level)
UDP datagrams -> QUIC layer (external or internal) -> H3 layer -> request stream -> handler -> response stream.

## Crates
- istok-core: parsing/encoding and pure state machines
- istok-transport: traits for datagrams/timers/crypto
- istok-h3: connection runtime over transport
- istok-io-tokio: tokio adapters
- istok-server: user-facing API

## Compatibility
Temporarily we expose Request/Response via `http` crate types (istok-http).
Later we may introduce Istok-native primitives and keep adapters.
