//! QPACK/HPACK prefix integer codec (RFC 7541 §5.1).
//!
//! Invariants:
//! - `prefix_bits` must be in 1..=8 (caller's responsibility; debug-asserted).
//! - `decode` never modifies the input slice.
//! - `encode` writes only to `out[0..bytes_written]`; caller pre-fills `out[0]`
//!   with instruction bits.
//! - No allocation; `core` only.

use core::fmt;

const MAX_EXTENSION_BYTES: usize = 5;
const MAX_PRACTICAL_VALUE: u64 = (1u64 << 35) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixIntError {
    /// Not enough bytes in the input or output buffer.
    BufferTooSmall,
    /// Extension chain exceeded 5 bytes — malformed or adversarial input.
    Overflow,
    /// Value is too large to encode (exceeds the 5-extension-byte cap).
    ValueTooLarge,
}

impl fmt::Display for PrefixIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefixIntError::BufferTooSmall => write!(f, "buffer too small"),
            PrefixIntError::Overflow => write!(f, "prefix integer overflow"),
            PrefixIntError::ValueTooLarge => write!(f, "value too large for prefix integer"),
        }
    }
}

#[inline]
fn prefix_mask(prefix_bits: u8) -> u64 {
    debug_assert!((1..=8).contains(&prefix_bits));
    (1u64 << prefix_bits) - 1
}

/// Decode a prefix integer from `input`.
///
/// `prefix_bits` is the number of low bits in `input[0]` that belong to this
/// integer (1–8). The high `8 - prefix_bits` bits are the instruction and are
/// masked off automatically.
///
/// Returns `(value, bytes_consumed)`.
pub fn decode(input: &[u8], prefix_bits: u8) -> Result<(u64, usize), PrefixIntError> {
    debug_assert!((1..=8).contains(&prefix_bits));

    if input.is_empty() {
        return Err(PrefixIntError::BufferTooSmall);
    }

    let mask = prefix_mask(prefix_bits);
    let raw = (input[0] & mask as u8) as u64;
    if raw < mask {
        return Ok((raw, 1));
    }

    let mut value = raw;
    for i in 0..MAX_EXTENSION_BYTES {
        let idx = 1 + i;
        if idx >= input.len() {
            return Err(PrefixIntError::BufferTooSmall);
        }
        let b = input[idx];
        value += ((b & 0x7f) as u64) << (7 * i);
        if (b & 0x80) == 0 {
            return Ok((value, idx + 1));
        }
    }

    Err(PrefixIntError::Overflow)
}

/// Encode `value` into `out`, beginning at `out[0]`.
///
/// `out[0]` must be pre-filled with the instruction bits (high
/// `8 - prefix_bits` bits). This function OR-s `value` into the low
/// `prefix_bits` bits of `out[0]` and appends extension bytes as needed.
///
/// Returns `bytes_written` (always ≥ 1; includes `out[0]`).
pub fn encode(value: u64, prefix_bits: u8, out: &mut [u8]) -> Result<usize, PrefixIntError> {
    debug_assert!((1..=8).contains(&prefix_bits));

    if value > MAX_PRACTICAL_VALUE {
        return Err(PrefixIntError::ValueTooLarge);
    }
    if out.is_empty() {
        return Err(PrefixIntError::BufferTooSmall);
    }

    let mask = prefix_mask(prefix_bits);
    if value < mask {
        out[0] |= value as u8;
        return Ok(1);
    }

    out[0] |= mask as u8;
    let mut remaining = value - mask;
    let mut written = 1usize;

    loop {
        if written >= out.len() {
            return Err(PrefixIntError::BufferTooSmall);
        }

        if remaining >= 128 {
            out[written] = ((remaining as u8) & 0x7f) | 0x80;
            remaining >>= 7;
            written += 1;
        } else {
            out[written] = remaining as u8;
            written += 1;
            break;
        }
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(prefix_bits: u8, value: u64) -> (usize, usize, u64) {
        let mut out = [0u8; 16];
        let written = encode(value, prefix_bits, &mut out).expect("encode succeeds");
        let (decoded, consumed) = decode(&out[..written], prefix_bits).expect("decode succeeds");
        (written, consumed, decoded)
    }

    #[test]
    fn roundtrip_fits_in_prefix() {
        for prefix_bits in [1u8, 4, 8] {
            let mask = (1u64 << prefix_bits) - 1;
            for value in [0u64, 1, mask - 1] {
                let (written, consumed, decoded) = roundtrip(prefix_bits, value);
                if value < mask {
                    assert_eq!(written, 1);
                    assert_eq!(consumed, 1);
                }
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn roundtrip_overflow_one_extension() {
        let prefix_bits = 3u8;
        let mask = (1u64 << prefix_bits) - 1;

        let (written_a, consumed_a, decoded_a) = roundtrip(prefix_bits, mask);
        assert_eq!(written_a, 2);
        assert_eq!(consumed_a, 2);
        assert_eq!(decoded_a, mask);

        let v = mask + 127;
        let (written_b, consumed_b, decoded_b) = roundtrip(prefix_bits, v);
        assert_eq!(written_b, 2);
        assert_eq!(consumed_b, 2);
        assert_eq!(decoded_b, v);
    }

    #[test]
    fn roundtrip_multi_extension() {
        let prefix_bits = 8u8;

        let (written_a, consumed_a, decoded_a) = roundtrip(prefix_bits, 1u64 << 14);
        assert_eq!(written_a, 3);
        assert_eq!(consumed_a, 3);
        assert_eq!(decoded_a, 1u64 << 14);

        let (written_b, consumed_b, decoded_b) = roundtrip(prefix_bits, 1u64 << 21);
        assert_eq!(written_b, 4);
        assert_eq!(consumed_b, 4);
        assert_eq!(decoded_b, 1u64 << 21);

        let (written_c, consumed_c, decoded_c) = roundtrip(prefix_bits, 1u64 << 28);
        assert_eq!(written_c, 5);
        assert_eq!(consumed_c, 5);
        assert_eq!(decoded_c, 1u64 << 28);
    }

    #[test]
    fn roundtrip_max_practical() {
        let value = (1u64 << 35) - 1;
        let (written, consumed, decoded) = roundtrip(1, value);
        assert_eq!(written, 6);
        assert_eq!(consumed, 6);
        assert_eq!(decoded, value);
    }

    #[test]
    fn roundtrip_all_prefix_widths() {
        for prefix_bits in 1u8..=8 {
            let mask = (1u64 << prefix_bits) - 1;
            for value in [0u64, mask - 1, mask + 1] {
                let (_written, _consumed, decoded) = roundtrip(prefix_bits, value);
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn decode_rfc_example_fits_in_prefix() {
        assert_eq!(decode(&[0b000_01010], 5), Ok((10, 1)));
    }

    #[test]
    fn decode_rfc_example_overflow() {
        assert_eq!(decode(&[0x1f, 0x9a, 0x0a], 5), Ok((1337, 3)));
    }

    #[test]
    fn decode_empty_input() {
        assert_eq!(decode(&[], 5), Err(PrefixIntError::BufferTooSmall));
    }

    #[test]
    fn decode_truncated_extension() {
        let mut out = [0u8; 16];
        let written = encode((1u64 << 5) - 1 + 128, 5, &mut out).expect("encode succeeds");
        assert_eq!(written, 3);
        assert_eq!(
            decode(&out[..written - 1], 5),
            Err(PrefixIntError::BufferTooSmall)
        );
    }

    #[test]
    fn decode_overflow_too_many_extension_bytes() {
        let input = [0x1f, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        assert_eq!(decode(&input, 5), Err(PrefixIntError::Overflow));
    }

    #[test]
    fn encode_value_too_large() {
        let mut out = [0u8; 16];
        assert_eq!(
            encode(1u64 << 35, 8, &mut out),
            Err(PrefixIntError::ValueTooLarge)
        );
    }

    #[test]
    fn encode_buffer_too_small() {
        let mut out = [0u8; 2];
        assert_eq!(
            encode((1u64 << 5) - 1 + 128, 5, &mut out),
            Err(PrefixIntError::BufferTooSmall)
        );
    }

    #[test]
    fn encode_preserves_instruction_bits() {
        let mut out = [0u8; 16];
        out[0] = 0b1010_0000;
        let written = encode(3, 5, &mut out).expect("encode succeeds");
        assert_eq!(written, 1);
        assert_eq!(out[0], 0b1010_0011);
    }
}
