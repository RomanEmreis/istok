# Istok — Overview

Istok is an HTTP/3-first server engine in Rust.

## Goals
- Correct HTTP/3 behavior with cleanly separated layers
- Minimal dependencies
- Path to `no_std` for protocol logic (`istok-core`)
- Deterministic testing via mock transport

## Layering
- Protocol core (no_std): frames, QPACK, H3 state machine
- Transport abstraction: datagrams, timers, crypto glue
- Runtime glue + adapters: tokio transport, server API, examples
