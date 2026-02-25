extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_control_stream_accepts_empty_settings_without_closing() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut buf = [0u8; 16];
    let stream_ty_len =
        varint::encode(consts::STREAM_TYPE_CONTROL, &mut buf).expect("stream type encodes");
    let frame_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
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
            data: alloc::vec::Vec::from(&buf[..stream_ty_len + frame_len]),
            fin: false,
        },
        ScriptStep::ExpectNone,
    ]);
}

#[test]
fn inbound_control_stream_accepts_empty_settings_with_extra_bytes() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut buf = [0u8; 32];
    let stream_ty_len =
        varint::encode(consts::STREAM_TYPE_CONTROL, &mut buf).expect("stream type encodes");
    let frame_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        },
        &mut buf[stream_ty_len..],
    )
    .expect("frame header encodes");

    let data_header_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_DATA,
            len: 0,
        },
        &mut buf[stream_ty_len + frame_len..],
    )
    .expect("data frame header encodes");

    let total = stream_ty_len + frame_len + data_header_len;

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&buf[..total]),
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_FRAME_UNEXPECTED,
        }),
    ]);
}

#[test]
fn inbound_control_stream_fin_only_empty_after_settings_is_ignored() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut buf = [0u8; 16];
    let stream_ty_len =
        varint::encode(consts::STREAM_TYPE_CONTROL, &mut buf).expect("stream type encodes");
    let frame_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
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
            data: alloc::vec::Vec::from(&buf[..stream_ty_len + frame_len]),
            fin: false,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::new(),
            fin: true,
        },
        ScriptStep::ExpectNone,
    ]);
}

#[test]
fn inbound_pending_stream_type_is_not_overwritten_by_second_uni_open() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let first_uni_id = StreamId(3);
    let second_uni_id = StreamId(7);

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: first_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicOpen {
            id: second_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        // If pending stream id were overwritten, this would try to parse stream type and close.
        ScriptStep::InQuicData {
            id: second_uni_id,
            data: alloc::vec::Vec::from(&[0xff]),
            fin: false,
        },
        ScriptStep::ExpectNone,
    ]);
}
