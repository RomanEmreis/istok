use crate::engine::{CommandSink, Engine, EngineCommand, EngineEvent};
use alloc::boxed::Box;
use alloc::vec::Vec;
use istok_core::codec::{h3_frame, varint};
use istok_core::h3::{consts, settings::Settings};
use istok_transport::{QuicCommand, StreamId};

/// Minimal H3 engine skeleton.
/// Codex will implement real behavior using istok-core codecs and settings encoder.
pub struct H3Engine {
    control_stream: Option<StreamId>,
}

impl H3Engine {
    pub fn new() -> Self {
        Self {
            control_stream: None,
        }
    }
}

impl Default for H3Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for H3Engine {
    fn on_event<'a>(&mut self, ev: EngineEvent<'a>, out: &mut dyn CommandSink<'a>) {
        match ev {
            EngineEvent::Boot => {
                let id = StreamId(2);
                self.control_stream = Some(id);

                out.push(EngineCommand::Quic(QuicCommand::OpenUni {
                    id_hint: Some(id),
                }));

                let settings = Settings::new();
                let mut payload_buf = [0u8; 0];
                let payload_len = match settings.encode_payload(&mut payload_buf) {
                    Ok(len) => len,
                    Err(_) => {
                        out.push(EngineCommand::Quic(QuicCommand::CloseConnection {
                            app_error: consts::H3_FRAME_ERROR,
                        }));
                        return;
                    }
                };

                let mut bytes = [0u8; 32];

                let stream_type_len = match varint::encode(consts::STREAM_TYPE_CONTROL, &mut bytes)
                {
                    Ok(len) => len,
                    Err(_) => {
                        out.push(EngineCommand::Quic(QuicCommand::CloseConnection {
                            app_error: consts::H3_FRAME_ERROR,
                        }));
                        return;
                    }
                };

                let header = h3_frame::FrameHeader {
                    ty: consts::FRAME_TYPE_SETTINGS,
                    len: payload_len as u64,
                };
                let header_len =
                    match h3_frame::encode_frame_header(header, &mut bytes[stream_type_len..]) {
                        Ok(len) => len,
                        Err(_) => {
                            out.push(EngineCommand::Quic(QuicCommand::CloseConnection {
                                app_error: consts::H3_FRAME_ERROR,
                            }));
                            return;
                        }
                    };

                let total = stream_type_len + header_len;
                let write_bytes: &'static [u8] =
                    Box::leak(Vec::from(&bytes[..total]).into_boxed_slice());
                out.push(EngineCommand::Quic(QuicCommand::StreamWrite {
                    id,
                    data: write_bytes,
                    fin: false,
                }));
            }
            _ => {}
        }
    }
}
