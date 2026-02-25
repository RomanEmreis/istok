extern crate alloc;

use istok_core::h3::consts;
use istok_h3::H3Engine;
use istok_h3::mock::{ExpectCommand, MockHarness, ScriptStep};
use istok_transport::{StreamId, StreamKind};

#[test]
fn inbound_uni_stream_type_non_control_closes_general_protocol_error() {
    let engine = H3Engine::new();
    let mut h = MockHarness::new(engine);

    let peer_uni_id = StreamId(3);

    // 0x01 is a valid varint but not the HTTP/3 control stream type.
    let unexpected_stream_type = alloc::vec::Vec::from(&[0x01][..]);

    h.run_script(&[
        ScriptStep::InQuicOpen {
            id: peer_uni_id,
            kind: StreamKind::Uni,
        },
        ScriptStep::ExpectNone,
        ScriptStep::InQuicData {
            id: peer_uni_id,
            data: unexpected_stream_type,
            fin: false,
        },
        ScriptStep::Expect(ExpectCommand::QuicCloseConnection {
            app_error: consts::H3_GENERAL_PROTOCOL_ERROR,
        }),
        ScriptStep::ExpectNone,
    ]);
}
