extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::StreamId;

#[test]
fn boot_opens_control_and_sends_settings() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_id = StreamId(2);

    let mut prefix = [0u8; 8];
    let stream_type_len = varint::encode(consts::STREAM_TYPE_CONTROL, &mut prefix)
        .expect("stream type varint should encode");
    let header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        },
        &mut prefix[stream_type_len..],
    )
    .expect("settings frame header should encode");

    h.run_script(&[
        ScriptStep::InBoot,
        ScriptStep::Expect(ExpectCommand::QuicOpenUni),
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: control_id,
            data_prefix: alloc::vec::Vec::from(&prefix[..stream_type_len + header_len]),
            fin: false,
        }),
        ScriptStep::ExpectNone,
    ]);
}
