#![no_main]

use istok_core::codec::varint;
use libfuzzer_sys::fuzz_target;

// Invariant: decode must never panic on arbitrary input.
// Any byte sequence must produce Ok or Err, never a panic.
fuzz_target!(|data: &[u8]| {
    let _ = varint::decode(data);
});
