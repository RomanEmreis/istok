extern crate alloc;

use istok_core::codec::varint;
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_control_stream_frame_header_truncated_with_fin_closes_frame_error() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut data = [0u8; 16];
    let stream_ty_len = varint::encode(consts::STREAM_TYPE_CONTROL, &mut data)
        .expect("control stream type encodes");

    // Start a frame header, but only provide one byte of an 8-byte varint and then FIN.
    data[stream_ty_len] = 0b1100_0000;

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&data[..stream_ty_len + 1]),
            fin: true,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
