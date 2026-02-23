use crate::engine::{CommandSink, Engine, EngineCommand, EngineEvent};
use alloc::vec::Vec;
use istok_core::codec::{h3_frame, varint};
use istok_core::h3::{consts, settings::Settings};
use istok_transport::{QuicCommand, QuicEvent, StreamId, StreamKind};

/// Minimal H3 engine skeleton.
/// Codex will implement real behavior using istok-core codecs and settings encoder.
pub struct H3Engine {
    control_stream: Option<StreamId>,
    inbound_uni_pending_type: Option<StreamId>,
    inbound_uni_pending_buf: Vec<u8>,
    inbound_uni_state: InboundUniState,
    inbound_control_stream: Option<StreamId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InboundUniState {
    NeedType,
    NeedFrameHeader,
    NeedPayload { len: usize },
}

impl H3Engine {
    pub fn new() -> Self {
        Self {
            control_stream: None,
            inbound_uni_pending_type: None,
            inbound_uni_pending_buf: Vec::new(),
            inbound_uni_state: InboundUniState::NeedType,
            inbound_control_stream: None,
        }
    }

    fn close_with<'a>(&self, out: &mut dyn CommandSink<'a>, app_error: u64) {
        out.push(EngineCommand::Quic(QuicCommand::CloseConnection {
            app_error,
        }));
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

                // M0: settings payload is empty; later this will be a real buffer.
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
                out.push(EngineCommand::Quic(QuicCommand::StreamWriteOwned {
                    id,
                    data: bytes[..total].to_vec(),
                    fin: false,
                }));
            }
            EngineEvent::Quic(QuicEvent::StreamOpened {
                id,
                kind: StreamKind::Uni,
            }) => {
                if self.inbound_uni_pending_type.is_none() {
                    self.inbound_uni_pending_type = Some(id);
                    self.inbound_uni_pending_buf.clear();
                    self.inbound_uni_state = InboundUniState::NeedType;
                }
            }
            EngineEvent::Quic(QuicEvent::StreamReadable { id, data, .. }) => {
                if self.inbound_uni_pending_type != Some(id) {
                    return;
                }

                self.inbound_uni_pending_buf.extend_from_slice(data);

                loop {
                    match self.inbound_uni_state {
                        InboundUniState::NeedType => {
                            let (stream_ty, consumed) =
                                match varint::decode(&self.inbound_uni_pending_buf) {
                                    Ok(parsed) => parsed,
                                    Err(varint::VarIntError::BufferTooSmall) => return,
                                    Err(_) => {
                                        self.close_with(out, consts::H3_GENERAL_PROTOCOL_ERROR);
                                        return;
                                    }
                                };

                            if stream_ty != consts::STREAM_TYPE_CONTROL {
                                self.close_with(out, consts::H3_GENERAL_PROTOCOL_ERROR);
                                return;
                            }

                            self.inbound_uni_pending_buf.drain(0..consumed);
                            self.inbound_uni_state = InboundUniState::NeedFrameHeader;
                        }
                        InboundUniState::NeedFrameHeader => {
                            let (frame_header, consumed) = match h3_frame::decode_frame_header(
                                &self.inbound_uni_pending_buf,
                            ) {
                                Ok(parsed) => parsed,
                                Err(h3_frame::Error::VarInt(
                                    varint::VarIntError::BufferTooSmall,
                                )) => return,
                                Err(_) => {
                                    self.close_with(out, consts::H3_FRAME_ERROR);
                                    return;
                                }
                            };

                            if frame_header.ty != consts::FRAME_TYPE_SETTINGS {
                                self.close_with(out, consts::H3_FRAME_UNEXPECTED);
                                return;
                            }

                            let payload_len = match usize::try_from(frame_header.len) {
                                Ok(len) => len,
                                Err(_) => {
                                    self.close_with(out, consts::H3_FRAME_ERROR);
                                    return;
                                }
                            };

                            if payload_len != 0 {
                                self.close_with(out, consts::H3_FRAME_ERROR);
                                return;
                            }

                            self.inbound_uni_pending_buf.drain(0..consumed);
                            self.inbound_uni_state =
                                InboundUniState::NeedPayload { len: payload_len };
                        }
                        InboundUniState::NeedPayload { len } => {
                            if self.inbound_uni_pending_buf.len() < len {
                                return;
                            }

                            self.inbound_uni_pending_buf.drain(0..len);
                            self.inbound_uni_pending_type = None;
                            self.inbound_control_stream = Some(id);
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
