use crate::engine::{CommandSink, Engine, EngineCommand, EngineEvent};
use istok_transport::{QuicCommand, StreamId};

/// Minimal H3 engine skeleton.
/// Codex will implement real behavior using istok-core codecs and settings encoder.
pub struct H3Engine {
    control_stream: Option<StreamId>,
}

impl H3Engine {
    pub fn new() -> Self {
        Self { control_stream: None }
    }
}

impl Default for H3Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for H3Engine {
    fn on_event<'a>(&mut self, ev: EngineEvent<'a>, out: &mut dyn CommandSink<'a>) {
        match ev {
            EngineEvent::Boot => {
                // Placeholder: open uni control stream and write SETTINGS.
                // Real impl should:
                // - pick/control stream id via runtime/quic
                // - write stream type (control) then SETTINGS frame (encoded via istok-core)
                let id = StreamId(2);
                self.control_stream = Some(id);

                out.push(EngineCommand::Quic(QuicCommand::OpenUni { id_hint: Some(id) }));

                // Temporary hardcoded bytes; must be replaced by istok-core encoder.
                let bytes: &'static [u8] = &[0x00, 0x04, 0x00];
                out.push(EngineCommand::Quic(QuicCommand::StreamWrite { id, data: bytes, fin: false }));
            }
            _ => {}
        }
    }
}
