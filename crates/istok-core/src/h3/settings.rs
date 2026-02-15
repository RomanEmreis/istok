//! SETTINGS frame payload encoding/decoding.
//!
//! RFC 9114 defines SETTINGS as a sequence of (identifier, value) varint pairs.

use core::fmt;

/// SETTINGS payload encoding errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BufferTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    // Start minimal; expand later.
    // Examples:
    // pub max_field_section_size: Option<u64>,
    // pub enable_connect_protocol: Option<u64>,
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Encodes this SETTINGS payload into `out`.
    ///
    /// Minimal M0 behavior: empty settings are valid and encode to 0 bytes.
    pub fn encode_payload(&self, out: &mut [u8]) -> Result<usize, Error> {
        let _ = self;
        let _ = out;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn empty_settings_encode_to_zero_bytes() {
        let settings = Settings::new();
        let mut out = [0u8; 0];
        let written = settings
            .encode_payload(&mut out)
            .expect("empty settings should encode");

        assert_eq!(written, 0);
    }

    #[test]
    fn empty_settings_ignore_non_empty_output_buffer() {
        let settings = Settings::new();
        let mut out = [0xAAu8; 8];
        let written = settings
            .encode_payload(&mut out)
            .expect("empty settings should encode");

        assert_eq!(written, 0);
        assert_eq!(out, [0xAAu8; 8]);
    }
}
