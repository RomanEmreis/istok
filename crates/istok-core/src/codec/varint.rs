use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarIntError {
    /// Not enough bytes in input/output buffer.
    BufferTooSmall,
    /// Value exceeds QUIC varint max (2^62 - 1).
    ValueTooLarge,
    /// Reserved/invalid encoding (shouldn't happen if we parse by spec, but keep it explicit).
    InvalidEncoding,
}

impl fmt::Display for VarIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarIntError::BufferTooSmall => write!(f, "buffer too small"),
            VarIntError::ValueTooLarge => write!(f, "value too large for QUIC varint"),
            VarIntError::InvalidEncoding => write!(f, "invalid varint encoding"),
        }
    }
}

/// QUIC varint maximum value: 2^62 - 1
pub const VARINT_MAX: u64 = (1u64 << 62) - 1;

/// Returns the number of bytes needed to encode `value` as QUIC varint.
pub const fn encoded_len(value: u64) -> Result<usize, VarIntError> {
    if value > VARINT_MAX {
        return Err(VarIntError::ValueTooLarge);
    }
    // 1 byte: 0..=63
    // 2 bytes: 64..=16383
    // 4 bytes: 16384..=1073741823
    // 8 bytes: 1073741824..=4611686018427387903
    if value <= 63 {
        Ok(1)
    } else if value <= 16_383 {
        Ok(2)
    } else if value <= 1_073_741_823 {
        Ok(4)
    } else {
        Ok(8)
    }
}

/// Decodes a QUIC varint from the start of `input`.
/// Returns (value, bytes_consumed).
pub fn decode(input: &[u8]) -> Result<(u64, usize), VarIntError> {
    if input.is_empty() {
        return Err(VarIntError::BufferTooSmall);
    }

    let first = input[0];
    let tag = first >> 6; // top 2 bits
    let len = match tag {
        0b00 => 1,
        0b01 => 2,
        0b10 => 4,
        0b11 => 8,
        _ => return Err(VarIntError::InvalidEncoding),
    };

    if input.len() < len {
        return Err(VarIntError::BufferTooSmall);
    }

    let mut v: u64 = 0;

    match len {
        1 => {
            v = (first & 0b0011_1111) as u64;
        }
        2 => {
            v = ((first as u64 & 0b0011_1111) << 8) | input[1] as u64;
        }
        4 => {
            v = ((first as u64 & 0b0011_1111) << 24)
                | (input[1] as u64) << 16
                | (input[2] as u64) << 8
                | (input[3] as u64);
        }
        8 => {
            v = ((first as u64 & 0b0011_1111) << 56)
                | (input[1] as u64) << 48
                | (input[2] as u64) << 40
                | (input[3] as u64) << 32
                | (input[4] as u64) << 24
                | (input[5] as u64) << 16
                | (input[6] as u64) << 8
                | (input[7] as u64);
        }
        _ => return Err(VarIntError::InvalidEncoding),
    }

    // v is by construction <= 2^62-1, but keep the guard for safety.
    if v > VARINT_MAX {
        return Err(VarIntError::InvalidEncoding);
    }

    Ok((v, len))
}

/// Encodes `value` as QUIC varint into the beginning of `out`.
/// Returns bytes_written.
pub fn encode(value: u64, out: &mut [u8]) -> Result<usize, VarIntError> {
    let len = encoded_len(value)?;
    if out.len() < len {
        return Err(VarIntError::BufferTooSmall);
    }

    match len {
        1 => {
            out[0] = (value as u8) & 0b0011_1111; // tag 00
        }
        2 => {
            out[0] = 0b01 << 6 | (((value >> 8) as u8) & 0b0011_1111);
            out[1] = (value & 0xFF) as u8;
        }
        4 => {
            out[0] = 0b10 << 6 | (((value >> 24) as u8) & 0b0011_1111);
            out[1] = ((value >> 16) & 0xFF) as u8;
            out[2] = ((value >> 8) & 0xFF) as u8;
            out[3] = (value & 0xFF) as u8;
        }
        8 => {
            out[0] = 0b11 << 6 | (((value >> 56) as u8) & 0b0011_1111);
            out[1] = ((value >> 48) & 0xFF) as u8;
            out[2] = ((value >> 40) & 0xFF) as u8;
            out[3] = ((value >> 32) & 0xFF) as u8;
            out[4] = ((value >> 24) & 0xFF) as u8;
            out[5] = ((value >> 16) & 0xFF) as u8;
            out[6] = ((value >> 8) & 0xFF) as u8;
            out[7] = (value & 0xFF) as u8;
        }
        _ => return Err(VarIntError::InvalidEncoding),
    }

    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_len_boundaries() {
        assert_eq!(encoded_len(0).unwrap(), 1);
        assert_eq!(encoded_len(63).unwrap(), 1);
        assert_eq!(encoded_len(64).unwrap(), 2);
        assert_eq!(encoded_len(16_383).unwrap(), 2);
        assert_eq!(encoded_len(16_384).unwrap(), 4);
        assert_eq!(encoded_len(1_073_741_823).unwrap(), 4);
        assert_eq!(encoded_len(1_073_741_824).unwrap(), 8);
        assert_eq!(encoded_len(VARINT_MAX).unwrap(), 8);
        assert_eq!(encoded_len(VARINT_MAX + 1).unwrap_err(), VarIntError::ValueTooLarge);
    }

    #[test]
    fn roundtrip_samples() {
        let samples = [
            0u64,
            1,
            7,
            63,
            64,
            152,
            16_383,
            16_384,
            1_000_000,
            1_073_741_823,
            1_073_741_824,
            VARINT_MAX,
        ];

        let mut buf = [0u8; 8];

        for &v in &samples {
            buf.fill(0);
            let n = encode(v, &mut buf).unwrap();
            let (decoded, used) = decode(&buf[..n]).unwrap();
            assert_eq!(decoded, v);
            assert_eq!(used, n);
        }
    }

    #[test]
    fn decode_buffer_too_small() {
        assert_eq!(decode(&[]).unwrap_err(), VarIntError::BufferTooSmall);

        // tag=11 => len=8, but only 1 byte present
        let one = [0b11 << 6];
        assert_eq!(decode(&one).unwrap_err(), VarIntError::BufferTooSmall);
    }

    #[test]
    fn encode_buffer_too_small() {
        let mut buf = [0u8; 1];
        assert_eq!(encode(64, &mut buf).unwrap_err(), VarIntError::BufferTooSmall);
    }

    #[test]
    fn decode_known_vectors_basic() {
        // 0 in 1 byte => 00xxxxxx
        let (v, n) = decode(&[0b0000_0000]).unwrap();
        assert_eq!(v, 0);
        assert_eq!(n, 1);

        // 63 in 1 byte
        let (v, n) = decode(&[0b0011_1111]).unwrap();
        assert_eq!(v, 63);
        assert_eq!(n, 1);

        // 64 in 2 bytes:
        // value=0x0040 => first: 01 + top 6 bits of value>>8 (0), second: 0x40
        let (v, n) = decode(&[0b0100_0000, 0x40]).unwrap();
        assert_eq!(v, 64);
        assert_eq!(n, 2);
    }
}
