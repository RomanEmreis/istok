extern crate alloc;

use istok_core::h3::consts;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_h3::H3Engine;
use istok_transport::{StreamId, StreamKind};

#[test]
fn early_request_buffer_over_cap_closes_connection() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let request_stream_id = StreamId(0);
    let over_cap = alloc::vec![0x42; (16 * 1024) + 16 + 1];

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: request_stream_id,
            kind: StreamKind::Bidi,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: request_stream_id,
            data: over_cap,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
