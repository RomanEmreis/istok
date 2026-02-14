#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

/// QUIC stream id is a QUIC varint in the wire, but we keep it as u64.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamId(pub u64);

/// Stream direction/type hint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    Uni,
    Bidi,
}

/// Why a stream was reset/closed (subset; expand later).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamError {
    Reset(u64),
    StopSending(u64),
}

/// Events emitted by the QUIC layer toward upper protocols (H3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuicEvent<'a> {
    /// New peer-initiated stream exists.
    StreamOpened { id: StreamId, kind: StreamKind },

    /// Bytes received on stream (ordered, reliable).
    StreamReadable { id: StreamId, data: &'a [u8], fin: bool },

    /// Peer reset/stop-sending.
    StreamError { id: StreamId, err: StreamError },

    /// Connection-level close.
    ConnectionClosed { app_error: Option<u64> },
}

/// Commands issued by upper protocol toward QUIC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuicCommand<'a> {
    /// Open a uni stream initiated by us (needed for H3 control/QPACK streams).
    OpenUni { id_hint: Option<StreamId> },

    /// Write bytes to stream (may be partial at runtime; mock can enforce full).
    StreamWrite { id: StreamId, data: &'a [u8], fin: bool },

    /// Reset stream (sender side).
    ResetStream { id: StreamId, app_error: u64 },

    /// Ask peer to stop sending on this stream.
    StopSending { id: StreamId, app_error: u64 },

    /// Close connection with application error code.
    CloseConnection { app_error: u64 },
}

/// Abstract QUIC connection driver interface.
/// Real impl might be async; for core + deterministic tests we use poll-style.
pub trait QuicTransport {
    type Error: fmt::Debug;

    /// Feed one event from QUIC into protocol.
    /// Runtime chooses how to produce events (socket/async/etc).
    fn next_event<'a>(&'a mut self) -> Result<Option<QuicEvent<'a>>, Self::Error>;

    /// Apply command(s) produced by protocol.
    fn apply_command<'a>(&mut self, cmd: QuicCommand<'a>) -> Result<(), Self::Error>;
}
