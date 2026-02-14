extern crate alloc;

use alloc::vec;

use istok_h3::engine::{EngineEvent, TimerId};
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId};

/// Placeholder engine for now
struct H3Engine {
    control_stream: Option<StreamId>,
}

impl H3Engine {
    fn new() -> Self {
        Self { control_stream: None }
    }
}

impl istok_h3::engine::Engine for H3Engine {
    fn on_event<'a>(&mut self, ev: EngineEvent<'a>, out: &mut dyn Extend<istok_h3::engine::EngineCommand<'a>>) {
        use istok_h3::engine::EngineCommand;
        use istok_transport::QuicCommand;

        match ev {
            EngineEvent::Boot => {
                // Skeleton behavior:
                // 1) open uni stream
                // 2) assume chosen StreamId=2 for now OR let runtime feed it back later.
                // For the scaffold test, we can hardcode a stream id and make mock expect it.
                let id = StreamId(2);
                self.control_stream = Some(id);

                out.extend([EngineCommand::Quic(QuicCommand::OpenUni { id_hint: Some(id) })]);

                // Write: stream type (0x00) + SETTINGS frame (0x04) + len 0
                let bytes: &'static [u8] = &[0x00, 0x04, 0x00];
                out.extend([EngineCommand::Quic(QuicCommand::StreamWrite { id, data: bytes, fin: false })]);
            }
            _ => {}
        }
    }
}

#[test]
fn boot_opens_control_and_sends_settings() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_id = StreamId(2);

    h.run_script(&[
        ScriptStep::InBoot,
        ScriptStep::Expect(ExpectCommand::QuicOpenUni),
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: control_id,
            data_prefix: alloc::vec![0x00, 0x04, 0x00],
            fin: false,
        }),
        ScriptStep::ExpectNone,
    ]);
}
