extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn request_data_buffered_before_control_is_processed_after_settings() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let request_stream_id = StreamId(0);
    let control_stream_id = StreamId(3);

    let mut req_header_buf = [0u8; 16];
    let req_payload = [0xaa, 0xbb];
    let req_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: req_payload.len() as u64,
        },
        &mut req_header_buf,
    )
    .expect("request frame header encodes");
    let mut req_bytes = alloc::vec::Vec::with_capacity(req_header_len + req_payload.len());
    req_bytes.extend_from_slice(&req_header_buf[..req_header_len]);
    req_bytes.extend_from_slice(&req_payload);

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
    .expect("settings frame header encodes");
    let control_total = control_type_len + control_frame_len;

    let mut resp_headers_header_buf = [0u8; 16];
    let resp_headers_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: 1,
        },
        &mut resp_headers_header_buf,
    )
    .expect("response frame header encodes");
    let mut resp_headers_prefix = alloc::vec::Vec::with_capacity(resp_headers_header_len + 1);
    resp_headers_prefix.extend_from_slice(&resp_headers_header_buf[..resp_headers_header_len]);
    resp_headers_prefix.push(0x00);

    let mut resp_data_header_buf = [0u8; 16];
    let resp_data_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 1,
        },
        &mut resp_data_header_buf,
    )
    .expect("response data frame header encodes");
    let mut resp_data_prefix = alloc::vec::Vec::with_capacity(resp_data_header_len + 1);
    resp_data_prefix.extend_from_slice(&resp_data_header_buf[..resp_data_header_len]);
    resp_data_prefix.push(0x01);

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: request_stream_id,
            kind: StreamKind::Bidi,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: request_stream_id,
            data: req_bytes,
            fin: false,
        },
        ScriptStep::ExpectNone,
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
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: resp_headers_prefix,
            fin: false,
        }),
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: resp_data_prefix,
            fin: true,
        }),
        ScriptStep::ExpectNone,
    ]);
}
