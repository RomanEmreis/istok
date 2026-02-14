#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

/// A borrowed datagram payload.
/// - Must be valid until the call returns.
/// - The transport implementation copies it or sends it immediately.
pub struct DatagramRef<'a> {
    pub bytes: &'a [u8],
}

/// Endpoint address abstraction.
/// Keep it opaque so we can support IP/port now, and potentially other transports later.
pub trait Endpoint: Clone + Eq + fmt::Debug {}

/// A minimal datagram transport abstraction.
/// H3 runs over QUIC which runs over UDP-like datagrams.
/// This trait does NOT prescribe QUIC; it is for the lowest "packet I/O" boundary.
pub trait DatagramTransport {
    type Endpoint: Endpoint;
    type Error: fmt::Debug;

    /// Receive one datagram. Non-blocking: returns `Ok(None)` when no packet is available.
    fn try_recv(&mut self, buf: &mut [u8]) -> Result<Option<(usize, Self::Endpoint)>, Self::Error>;

    /// Send one datagram to a remote endpoint.
    fn send(&mut self, to: &Self::Endpoint, datagram: DatagramRef<'_>) -> Result<(), Self::Error>;

    /// Max size supported for outgoing datagrams (including headers).
    fn max_datagram_size(&self) -> usize;
}

/// Optional: a lightweight event sink for transporting internal signals from timers/IO to the engine.
/// This keeps core runtime deterministic and avoids locking.
/// You can implement it with an mpsc channel in std/tokio.
pub trait EventSink<E> {
    type Error: fmt::Debug;

    fn push(&mut self, event: E) -> Result<(), Self::Error>;
}
