extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_control_stream_first_frame_non_settings_closes_frame_unexpected() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut buf = [0u8; 16];
    let stream_ty_len =
        varint::encode(consts::STREAM_TYPE_CONTROL, &mut buf).expect("control stream type encodes");
    let frame_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 0,
        },
        &mut buf[stream_ty_len..],
    )
    .expect("frame header encodes");

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&buf[..stream_ty_len + frame_header_len]),
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_UNEXPECTED,
        }),
        ScriptStep::ExpectNone,
    ]);
}
