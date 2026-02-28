extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn extra_bytes_after_declared_headers_payload_closes_connection() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let control_stream_id = StreamId(3);
    let request_stream_id = StreamId(0);

    let mut control_buf = [0u8; 16];
    let control_type_len = varint::encode(consts::STREAM_TYPE_CONTROL, &mut control_buf)
        .expect("control type encodes");
    let control_frame_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        },
        &mut control_buf[control_type_len..],
    )
    .expect("control settings frame encodes");
    let control_total = control_type_len + control_frame_len;

    let mut request_header = [0u8; 16];
    let request_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: 2,
        },
        &mut request_header,
    )
    .expect("request headers frame header encodes");

    let mut request_data = alloc::vec::Vec::with_capacity(request_header_len + 3);
    request_data.extend_from_slice(&request_header[..request_header_len]);
    request_data.extend_from_slice(&[0xaa, 0xbb, 0xcc]);

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
            data: request_data,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
