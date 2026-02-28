#![no_main]

use istok_core::codec::h3_frame;
use libfuzzer_sys::fuzz_target;

// Invariant: decode_frame_header must never panic on arbitrary input.
// Any byte sequence must produce Ok or Err, never a panic.
fuzz_target!(|data: &[u8]| {
    let _ = h3_frame::decode_frame_header(data);
});
