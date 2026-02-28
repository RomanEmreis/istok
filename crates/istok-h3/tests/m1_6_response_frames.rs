extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn response_is_headers_fin_false_then_data_fin_true() {
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

    let mut request_header_buf = [0u8; 16];
    let request_payload = [0xaa, 0xbb];
    let request_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: request_payload.len() as u64,
        },
        &mut request_header_buf,
    )
    .expect("request frame header encodes");

    let mut request_data =
        alloc::vec::Vec::with_capacity(request_header_len + request_payload.len());
    request_data.extend_from_slice(&request_header_buf[..request_header_len]);
    request_data.extend_from_slice(&request_payload);

    let mut response_headers_header_buf = [0u8; 16];
    let response_headers_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: 1,
        },
        &mut response_headers_header_buf,
    )
    .expect("response headers frame header encodes");

    let mut response_headers_prefix =
        alloc::vec::Vec::with_capacity(response_headers_header_len + 1);
    response_headers_prefix
        .extend_from_slice(&response_headers_header_buf[..response_headers_header_len]);
    response_headers_prefix.push(0x00);

    let mut response_data_header_buf = [0u8; 16];
    let response_data_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 1,
        },
        &mut response_data_header_buf,
    )
    .expect("response data frame header encodes");

    let mut response_data_prefix = alloc::vec::Vec::with_capacity(response_data_header_len + 1);
    response_data_prefix.extend_from_slice(&response_data_header_buf[..response_data_header_len]);
    response_data_prefix.push(0x01);

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
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: response_headers_prefix,
            fin: false,
        }),
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: response_data_prefix,
            fin: true,
        }),
        ScriptStep::ExpectNone,
    ]);
}

#[test]
fn fin_only_readable_after_m1_6_response_is_ignored() {
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

    let mut request_header_buf = [0u8; 16];
    let request_payload = [0xaa, 0xbb];
    let request_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: request_payload.len() as u64,
        },
        &mut request_header_buf,
    )
    .expect("request frame header encodes");

    let mut request_data =
        alloc::vec::Vec::with_capacity(request_header_len + request_payload.len());
    request_data.extend_from_slice(&request_header_buf[..request_header_len]);
    request_data.extend_from_slice(&request_payload);

    let mut response_headers_header_buf = [0u8; 16];
    let response_headers_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_HEADERS,
            len: 1,
        },
        &mut response_headers_header_buf,
    )
    .expect("response headers frame header encodes");

    let mut response_headers_prefix =
        alloc::vec::Vec::with_capacity(response_headers_header_len + 1);
    response_headers_prefix
        .extend_from_slice(&response_headers_header_buf[..response_headers_header_len]);
    response_headers_prefix.push(0x00);

    let mut response_data_header_buf = [0u8; 16];
    let response_data_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 1,
        },
        &mut response_data_header_buf,
    )
    .expect("response data frame header encodes");

    let mut response_data_prefix = alloc::vec::Vec::with_capacity(response_data_header_len + 1);
    response_data_prefix.extend_from_slice(&response_data_header_buf[..response_data_header_len]);
    response_data_prefix.push(0x01);

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
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: response_headers_prefix,
            fin: false,
        }),
        ScriptStep::Expect(ExpectCommand::QuicStreamWrite {
            id: request_stream_id,
            data_prefix: response_data_prefix,
            fin: true,
        }),
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: request_stream_id,
            data: alloc::vec::Vec::new(),
            fin: true,
        },
        ScriptStep::ExpectNone,
    ]);
}
