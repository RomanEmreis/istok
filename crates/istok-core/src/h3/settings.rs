//! SETTINGS frame payload encoding/decoding.
//!
//! RFC 9114 defines SETTINGS as a sequence of (identifier, value) varint pairs.

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
}
