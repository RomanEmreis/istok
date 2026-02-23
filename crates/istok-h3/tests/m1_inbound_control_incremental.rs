extern crate alloc;

use istok_core::codec::{h3_frame, varint};
use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_control_stream_accepts_split_stream_type_varint() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    let mut frame_buf = [0u8; 16];
    let frame_len = h3_frame::encode_frame_header(
        h3_frame::FrameHeader {
            ty: consts::FRAME_TYPE_SETTINGS,
            len: 0,
        },
        &mut frame_buf,
    )
    .expect("frame header encodes");

    // Non-minimal two-byte varint encoding for value 0 to force split stream-type parsing.
    let split_stream_type_first = [0x40u8];
    let split_stream_type_second = [0x00u8];

    let mut second_chunk = alloc::vec::Vec::from(&split_stream_type_second[..]);
    second_chunk.extend_from_slice(&frame_buf[..frame_len]);

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&split_stream_type_first[..]),
            fin: false,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: second_chunk,
            fin: false,
        },
        ScriptStep::ExpectNone,
    ]);
}

#[test]
fn inbound_control_stream_accepts_split_frame_header() {
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

    let frame_start = stream_ty_len;
    let split_at = frame_start + 1;

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&buf[..split_at]),
            fin: false,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: alloc::vec::Vec::from(&buf[split_at..stream_ty_len + frame_len]),
            fin: false,
        },
        ScriptStep::ExpectNone,
    ]);
}
