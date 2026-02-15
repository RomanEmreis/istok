//! HTTP/3 frame header and frame codec.
//!
//! Invariants:
//! - Frame header is encoded as `type(varint)` + `length(varint)`.
//! - `decode_frame` is allocation-free and returns a borrowed payload slice.
//! - Parsing is deterministic and only consumes bytes required by the frame.

use core::fmt;

use crate::codec::varint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub ty: u64,
    pub len: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BufferTooSmall,
    LengthExceedsInput,
    VarInt(varint::VarIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferTooSmall => write!(f, "buffer too small"),
            Error::LengthExceedsInput => write!(f, "frame payload length exceeds remaining input"),
            Error::VarInt(inner) => write!(f, "varint error: {inner}"),
        }
    }
}

impl From<varint::VarIntError> for Error {
    fn from(value: varint::VarIntError) -> Self {
        Self::VarInt(value)
    }
}

/// Decode an HTTP/3 frame header from `input`.
///
/// Returns `(header, bytes_consumed)`.
pub fn decode_frame_header(input: &[u8]) -> Result<(FrameHeader, usize), Error> {
    let (ty, ty_len) = varint::decode(input)?;
    let (len, len_len) = varint::decode(&input[ty_len..])?;

    Ok((
        FrameHeader { ty, len },
        ty_len + len_len,
    ))
}

/// Encode an HTTP/3 frame header into `out`.
///
/// Returns `bytes_written`.
pub fn encode_frame_header(h: FrameHeader, out: &mut [u8]) -> Result<usize, Error> {
    let ty_encoded_len = varint::encoded_len(h.ty)?;
    let len_encoded_len = varint::encoded_len(h.len)?;
    let total = ty_encoded_len + len_encoded_len;

    if out.len() < total {
        return Err(Error::BufferTooSmall);
    }

    let ty_written = varint::encode(h.ty, out)?;
    let len_written = varint::encode(h.len, &mut out[ty_written..])?;

    Ok(ty_written + len_written)
}

/// Decode a full HTTP/3 frame from `input`.
///
/// Returns `(header, payload, bytes_consumed)`.
pub fn decode_frame(input: &[u8]) -> Result<(FrameHeader, &[u8], usize), Error> {
    let (header, header_len) = decode_frame_header(input)?;

    let payload_len = usize::try_from(header.len).map_err(|_| Error::LengthExceedsInput)?;
    let available = input.len().saturating_sub(header_len);
    if payload_len > available {
        return Err(Error::LengthExceedsInput);
    }

    let payload_start = header_len;
    let payload_end = payload_start + payload_len;
    let payload = &input[payload_start..payload_end];

    Ok((header, payload, payload_end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::h3::consts;

    #[test]
    fn roundtrip_header() {
        let cases = [
            FrameHeader { ty: consts::FRAME_TYPE_DATA, len: 0 },
            FrameHeader { ty: consts::FRAME_TYPE_HEADERS, len: 5 },
            FrameHeader { ty: consts::FRAME_TYPE_SETTINGS, len: 16_383 },
            FrameHeader {
                ty: consts::FRAME_TYPE_GOAWAY,
                len: varint::VARINT_MAX,
            },
        ];

        let mut buf = [0u8; 16];
        for header in cases {
            buf.fill(0);
            let written = encode_frame_header(header, &mut buf).unwrap();
            let (decoded, consumed) = decode_frame_header(&buf[..written]).unwrap();
            assert_eq!(decoded, header);
            assert_eq!(consumed, written);
        }
    }

    #[test]
    fn roundtrip_full_frame_with_payload() {
        let payload = [0xde, 0xad, 0xbe, 0xef, 0x01, 0x02];
        let header = FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: payload.len() as u64,
        };

        let mut buf = [0u8; 32];
        let header_len = encode_frame_header(header, &mut buf).unwrap();
        buf[header_len..header_len + payload.len()].copy_from_slice(&payload);
        let total_len = header_len + payload.len();

        let (decoded_header, decoded_payload, consumed) = decode_frame(&buf[..total_len]).unwrap();
        assert_eq!(decoded_header, header);
        assert_eq!(decoded_payload, payload);
        assert_eq!(consumed, total_len);
    }

    #[test]
    fn malformed_truncated_varint_in_header() {
        // Type varint present, length varint indicates 8-byte encoding but bytes are missing.
        let input = [consts::FRAME_TYPE_DATA as u8, 0b11 << 6, 0x00, 0x01];
        assert_eq!(decode_frame_header(&input).unwrap_err(), Error::VarInt(varint::VarIntError::BufferTooSmall));
    }

    #[test]
    fn malformed_length_exceeds_input() {
        let mut buf = [0u8; 16];
        let header = FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: 4,
        };
        let header_len = encode_frame_header(header, &mut buf).unwrap();
        // Only provide 3 payload bytes when header claims 4.
        buf[header_len..header_len + 3].copy_from_slice(&[1, 2, 3]);
        let total = header_len + 3;

        assert_eq!(decode_frame(&buf[..total]).unwrap_err(), Error::LengthExceedsInput);
    }

    #[test]
    fn boundary_values_zero_payload() {
        let header = FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        };
        let mut buf = [0u8; 16];
        let header_len = encode_frame_header(header, &mut buf).unwrap();

        let (decoded_header, payload, consumed) = decode_frame(&buf[..header_len]).unwrap();
        assert_eq!(decoded_header, header);
        assert!(payload.is_empty());
        assert_eq!(consumed, header_len);
    }

    #[test]
    fn boundary_values_large_length_header_only() {
        let header = FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: varint::VARINT_MAX,
        };
        let mut buf = [0u8; 16];
        let written = encode_frame_header(header, &mut buf).unwrap();
        let (decoded, consumed) = decode_frame_header(&buf[..written]).unwrap();
        assert_eq!(decoded, header);
        assert_eq!(consumed, written);
    }

    #[test]
    fn encode_header_buffer_too_small() {
        let header = FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 16_384,
        };
        let mut buf = [0u8; 2];
        assert_eq!(encode_frame_header(header, &mut buf).unwrap_err(), Error::BufferTooSmall);
    }
}
