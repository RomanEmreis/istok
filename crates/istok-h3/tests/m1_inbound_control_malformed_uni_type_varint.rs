extern crate alloc;

use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_control_stream_type_malformed_varint_closes_general_protocol_error() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    // Non-canonical QUIC varint encoding of 0 (2-byte form) => malformed.
    let malformed_stream_type = alloc::vec::Vec::from(&[0b0100_0000, 0x00][..]);

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: malformed_stream_type,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_GENERAL_PROTOCOL_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
