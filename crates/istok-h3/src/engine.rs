use istok_transport::{QuicCommand, QuicEvent};

/// Events that the engine consumes (from QUIC + timers + shutdown).
pub enum EngineEvent<'a> {
    Boot,
    Quic(QuicEvent<'a>),
    TimerFired(TimerId),
    Shutdown,
}

/// Things engine wants the runtime to do.
pub enum EngineCommand<'a> {
    Quic(QuicCommand<'a>),
    ArmTimer { id: TimerId, deadline_ms_from_now: u64 },
    CancelTimer { id: TimerId },
}

/// Stable ids for protocol timers (PTO, delayed ACK, etc). Expand later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(pub u32);

/// Object-safe command sink.
pub trait CommandSink<'a> {
    fn push(&mut self, cmd: EngineCommand<'a>);
}

/// Pure engine step: consumes exactly one event and yields zero or more commands.
pub trait Engine {
    fn on_event<'a>(&mut self, ev: EngineEvent<'a>, out: &mut dyn CommandSink<'a>);
}
