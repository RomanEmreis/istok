extern crate alloc;

use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn request_headers_header_truncated_with_fin_closes_connection() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_stream_id = StreamId(3);
    let request_stream_id = StreamId(0);

    let control_settings = alloc::vec![
        consts::STREAM_TYPE_CONTROL as u8,
        consts::FRAME_TYPE_SETTINGS as u8,
        0x00,
    ];

    // First byte uses the 8-byte varint tag, but only one byte is present.
    let truncated_header = alloc::vec![0b11 << 6];

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: control_stream_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: control_stream_id,
            data: control_settings,
            fin: false,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicOpen {
            id: request_stream_id,
            kind: StreamKind::Bidi,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: request_stream_id,
            data: truncated_header,
            fin: true,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
