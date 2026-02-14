//! HTTP/3 constants (RFC 9114).
//! Keep protocol "magic numbers" here to avoid scattering them across the codebase.

/// Unidirectional stream types
pub const STREAM_TYPE_CONTROL: u64 = 0x00;
pub const STREAM_TYPE_PUSH: u64 = 0x01;
pub const STREAM_TYPE_QPACK_ENCODER: u64 = 0x02;
pub const STREAM_TYPE_QPACK_DECODER: u64 = 0x03;

/// Frame types
pub const FRAME_TYPE_DATA: u64 = 0x00;
pub const FRAME_TYPE_HEADERS: u64 = 0x01;
pub const FRAME_TYPE_SETTINGS: u64 = 0x04;
pub const FRAME_TYPE_GOAWAY: u64 = 0x07;
// Add more as needed (CANCEL_PUSH=0x03, MAX_PUSH_ID=0x0D, etc.)

/// Common error codes (subset; expand later)
pub const H3_NO_ERROR: u64 = 0x0100;
pub const H3_GENERAL_PROTOCOL_ERROR: u64 = 0x0101;
pub const H3_FRAME_UNEXPECTED: u64 = 0x0103;
pub const H3_FRAME_ERROR: u64 = 0x0106;
