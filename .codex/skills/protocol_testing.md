# Protocol testing (istok)

## Deterministic harness
- Use a mock transport with:
  - scripted incoming datagrams
  - captured outgoing datagrams
  - simulated timers

## Codec tests
- roundtrip: encode -> decode -> equals
- reject malformed frames
- boundary conditions (max varint, empty payloads)

## Fuzzing (later)
- fuzz targets for frame decoding and QPACK decoding
- crashes are bugs; never panic on input
