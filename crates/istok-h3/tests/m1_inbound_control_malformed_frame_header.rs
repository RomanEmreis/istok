extern crate alloc;

use istok_core::codec::varint;
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_control_stream_malformed_frame_header_closes_frame_error() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut data = [0u8; 8];
    let stream_ty_len = varint::encode(consts::STREAM_TYPE_CONTROL, &mut data)
        .expect("control stream type encodes");

    // Non-canonical varint for frame type (value 0 using 2-byte form) => malformed header.
    data[stream_ty_len] = 0b0100_0000;
    data[stream_ty_len + 1] = 0x00;

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&data[..stream_ty_len + 2]),
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
