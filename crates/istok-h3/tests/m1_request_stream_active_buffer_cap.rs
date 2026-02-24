extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_h3::H3Engine;
use istok_transport::{StreamId, StreamKind};

#[test]
fn active_request_buffer_over_cap_closes_connection() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_stream_id = StreamId(3);
    let request_stream_id = StreamId(0);

    let mut control_buf = [0u8; 16];
    let control_type_len =
        varint::encode(consts::STREAM_TYPE_CONTROL, &mut control_buf).expect("control type encodes");
    let control_frame_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        },
        &mut control_buf[control_type_len..],
    )
    .expect("control settings frame encodes");
    let control_total = control_type_len + control_frame_len;

    let over_cap = alloc::vec![0x24; (16 * 1024) + 16 + 1];

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: control_stream_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: control_stream_id,
            data: alloc::vec::Vec::from(&control_buf[..control_total]),
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
            data: over_cap,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
