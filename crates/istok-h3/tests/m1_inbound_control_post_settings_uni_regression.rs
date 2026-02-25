extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_uni_stream_after_settings_is_still_validated() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_stream_id = StreamId(3);
    let second_uni_id = StreamId(7);

    let mut control_buf = [0u8; 16];
    let control_type_len = varint::encode(consts::STREAM_TYPE_CONTROL, &mut control_buf)
        .expect("control type encodes");
    let settings_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        },
        &mut control_buf[control_type_len..],
    )
    .expect("settings frame encodes");

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: control_stream_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: control_stream_id,
            data: alloc::vec::Vec::from(&control_buf[..control_type_len + settings_len]),
            fin: false,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicOpen {
            id: second_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: second_uni_id,
            data: alloc::vec::Vec::from(&[0x01][..]),
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_GENERAL_PROTOCOL_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
