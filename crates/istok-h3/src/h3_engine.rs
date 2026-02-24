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
    pending_request_stream: Option<StreamId>,
    inbound_request_stream: Option<StreamId>,
    request_stream_claimed: bool,
    claimed_request_stream_id: Option<StreamId>,
    pending_request_fin: bool,
    inbound_request_buf: Vec<u8>,
    inbound_request_state: InboundRequestState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InboundUniState {
    NeedType,
    NeedFrameHeader,
    NeedPayload { len: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InboundRequestState {
    NeedFrameHeader,
    NeedPayload { len: usize },
    Complete,
}

const RESPONSE_HEADERS_PAYLOAD: [u8; 1] = [0x00];
const MAX_REQUEST_HEADERS_PAYLOAD: usize = 16 * 1024;

impl H3Engine {
    pub fn new() -> Self {
        Self {
            control_stream: None,
            inbound_uni_pending_type: None,
            inbound_uni_pending_buf: Vec::new(),
            inbound_uni_state: InboundUniState::NeedType,
            inbound_control_stream: None,
            pending_request_stream: None,
            inbound_request_stream: None,
            request_stream_claimed: false,
            claimed_request_stream_id: None,
            pending_request_fin: false,
            inbound_request_buf: Vec::new(),
            inbound_request_state: InboundRequestState::NeedFrameHeader,
        }
    }

    fn parse_request_stream<'a>(&mut self, id: StreamId, fin: bool, out: &mut dyn CommandSink<'a>) {
        if self.inbound_request_stream != Some(id) {
            return;
        }

        if self.inbound_request_state == InboundRequestState::Complete {
            self.inbound_request_stream = None;
            self.inbound_request_buf.clear();
            self.pending_request_fin = false;
            return;
        }

        loop {
            match self.inbound_request_state {
                InboundRequestState::NeedFrameHeader => {
                    let (frame_header, consumed) =
                        match h3_frame::decode_frame_header(&self.inbound_request_buf) {
                            Ok(parsed) => parsed,
                            Err(h3_frame::Error::VarInt(varint::VarIntError::BufferTooSmall)) => {
                                if fin {
                                    self.close_with(out, consts::H3_FRAME_ERROR);
                                }
                                return;
                            }
                            Err(_) => {
                                self.close_with(out, consts::H3_FRAME_ERROR);
                                return;
                            }
                        };

                    if frame_header.ty != consts::FRAME_TYPE_HEADERS {
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

                    if payload_len > MAX_REQUEST_HEADERS_PAYLOAD {
                        self.close_with(out, consts::H3_FRAME_ERROR);
                        return;
                    }

                    self.inbound_request_buf.drain(0..consumed);
                    self.inbound_request_state = InboundRequestState::NeedPayload { len: payload_len };
                }
                InboundRequestState::NeedPayload { len } => {
                    if self.inbound_request_buf.len() < len {
                        if fin {
                            self.close_with(out, consts::H3_FRAME_ERROR);
                        }
                        return;
                    }

                    self.inbound_request_buf.drain(0..len);
                    if !self.inbound_request_buf.is_empty() {
                        self.close_with(out, consts::H3_FRAME_ERROR);
                        return;
                    }

                    self.inbound_request_state = InboundRequestState::Complete;
                    self.inbound_request_stream = None;
                    self.inbound_request_buf.clear();
                    self.pending_request_fin = false;

                    let mut frame_header = [0u8; 16];
                    let header_len = match h3_frame::encode_frame_header(
                        h3_frame::FrameHeader {
                            ty: consts::FRAME_TYPE_HEADERS,
                            len: RESPONSE_HEADERS_PAYLOAD.len() as u64,
                        },
                        &mut frame_header,
                    ) {
                        Ok(len) => len,
                        Err(_) => {
                            self.close_with(out, consts::H3_FRAME_ERROR);
                            return;
                        }
                    };

                    let mut response =
                        Vec::with_capacity(header_len + RESPONSE_HEADERS_PAYLOAD.len());
                    response.extend_from_slice(&frame_header[..header_len]);
                    response.extend_from_slice(&RESPONSE_HEADERS_PAYLOAD);

                    out.push(EngineCommand::Quic(QuicCommand::StreamWriteOwned {
                        id,
                        data: response,
                        fin: false,
                    }));
                    return;
                }
                InboundRequestState::Complete => return,
            }
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
            EngineEvent::Quic(QuicEvent::StreamOpened {
                id,
                kind: StreamKind::Bidi,
            }) => {
                if self.request_stream_claimed {
                    return;
                }

                if self.inbound_request_stream.is_none() && self.pending_request_stream.is_none() {
                    self.pending_request_stream = Some(id);
                    self.request_stream_claimed = true;
                    self.claimed_request_stream_id = Some(id);
                }
            }
            EngineEvent::Quic(QuicEvent::StreamReadable { id, data, fin }) => {
                if self.inbound_uni_pending_type == Some(id) {
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

                                // M1/M1.2 simplicity: front-drain from Vec. This is O(n);
                                // a cursor/ring-buffer is a likely M2+ follow-up.
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

                                if let Some(req_id) = self.pending_request_stream {
                                    self.pending_request_stream = None;
                                    self.inbound_request_stream = Some(req_id);
                                    self.inbound_request_state = InboundRequestState::NeedFrameHeader;
                                    self.parse_request_stream(req_id, self.pending_request_fin, out);
                                }
                                return;
                            }
                        }
                    }
                }

                if self.pending_request_stream == Some(id) && self.inbound_control_stream.is_none() {
                    self.inbound_request_buf.extend_from_slice(data);
                    self.pending_request_fin |= fin;
                    return;
                }

                if self.inbound_request_stream.is_none()
                    && self.pending_request_stream.is_none()
                    && self.claimed_request_stream_id == Some(id)
                {
                    self.close_with(out, consts::H3_GENERAL_PROTOCOL_ERROR);
                    return;
                }

                if self.inbound_request_stream.is_none()
                    && self.inbound_control_stream.is_some()
                    && self.pending_request_stream == Some(id)
                {
                    self.pending_request_stream = None;
                    self.inbound_request_stream = Some(id);
                    self.inbound_request_state = InboundRequestState::NeedFrameHeader;
                }

                if self.inbound_request_stream != Some(id) {
                    return;
                }

                self.inbound_request_buf.extend_from_slice(data);
                self.parse_request_stream(id, fin, out);
            }
            _ => {}
        }
    }
}
