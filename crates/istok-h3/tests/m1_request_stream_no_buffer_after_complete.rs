extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_h3::H3Engine;
use istok_transport::{StreamId, StreamKind};

#[test]
fn readable_after_request_completion_closes_connection() {
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

    let mut request_buf = [0u8; 16];
    let request_payload = [0xaa, 0xbb];
    let request_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: request_payload.len() as u64,
        },
        &mut request_buf,
    )
    .expect("request header encodes");

    let mut request = alloc::vec::Vec::with_capacity(request_header_len + request_payload.len());
    request.extend_from_slice(&request_buf[..request_header_len]);
    request.extend_from_slice(&request_payload);

    let mut response_buf = [0u8; 16];
    let response_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: 1,
        },
        &mut response_buf,
    )
    .expect("response header encodes");

    let mut response_prefix = alloc::vec::Vec::with_capacity(response_header_len + 1);
    response_prefix.extend_from_slice(&response_buf[..response_header_len]);
    response_prefix.push(0x00);

    let extra = alloc::vec![0x11; 64 * 1024];

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
            data: request,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: response_prefix,
            fin: true,
        }),
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: request_stream_id,
            data: extra,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_GENERAL_PROTOCOL_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
