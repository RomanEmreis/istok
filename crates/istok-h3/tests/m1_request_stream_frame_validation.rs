extern crate alloc;

use istok_core::codec::h3_frame;
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

fn control_settings_bytes() -> alloc::vec::Vec<u8> {
    alloc::vec![
        consts::STREAM_TYPE_CONTROL as u8,
        consts::FRAME_TYPE_SETTINGS as u8,
        0x00,
    ]
}

#[test]
fn request_first_frame_data_closes_frame_unexpected() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_stream_id = StreamId(3);
    let request_stream_id = StreamId(0);

    let mut data_header = [0u8; 16];
    let data_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 0,
        },
        &mut data_header,
    )
    .expect("data frame header encodes");

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: control_stream_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: control_stream_id,
            data: control_settings_bytes(),
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
            data: alloc::vec::Vec::from(&data_header[..data_header_len]),
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_UNEXPECTED,
        }),
    ]);
}

// Note: we do not currently add a "non-BufferTooSmall decode_frame_header error" request-stream
// test because h3_frame::decode_frame_header delegates to varint::decode, and with the current
// codec there is no deterministic byte pattern that triggers a varint decode error other than
// BufferTooSmall for frame headers.
