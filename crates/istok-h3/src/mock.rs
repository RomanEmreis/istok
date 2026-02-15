#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;

use crate::engine::{CommandSink, Engine, EngineCommand, EngineEvent, TimerId};
use istok_transport::{QuicCommand, QuicEvent, StreamId, StreamKind};

#[derive(Clone, Debug)]
pub enum ScriptStep {
    InBoot,
    InQuicOpen {
        id: StreamId,
        kind: StreamKind,
    },
    InQuicData {
        id: StreamId,
        data: Vec<u8>,
        fin: bool,
    },
    InTimer(TimerId),
    InShutdown,

    // Expectations about commands produced immediately after the last input step.
    Expect(ExpectCommand),
    ExpectNone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpectCommand {
    QuicOpenUni,
    QuicStreamWrite {
        id: StreamId,
        /// Expected prefix of written bytes (useful when payload has varints etc).
        data_prefix: Vec<u8>,
        fin: bool,
    },
    QuicCloseConnection {
        app_error: u64,
    },
    ArmTimer {
        id: TimerId,
    },
    CancelTimer {
        id: TimerId,
    },
}

pub struct MockHarness<E: Engine> {
    engine: E,
    pending: Vec<EngineCommandOwned>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EngineCommandOwned {
    QuicOpenUni,
    QuicStreamWrite {
        id: StreamId,
        data: Vec<u8>,
        fin: bool,
    },
    QuicCloseConnection {
        app_error: u64,
    },
    ArmTimer {
        id: TimerId,
        deadline_ms_from_now: u64,
    },
    CancelTimer {
        id: TimerId,
    },
}

struct VecSink<'a> {
    out: Vec<EngineCommand<'a>>,
}

impl<'a> VecSink<'a> {
    fn new() -> Self {
        Self { out: Vec::new() }
    }
}

impl<'a> CommandSink<'a> for VecSink<'a> {
    fn push(&mut self, cmd: EngineCommand<'a>) {
        self.out.push(cmd);
    }
}

impl<E: Engine> MockHarness<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            pending: Vec::new(),
        }
    }

    pub fn run_script(&mut self, script: &[ScriptStep]) {
        // Each input step produces a fresh batch of pending commands.
        // Expectation steps consume from `pending`.
        for step in script {
            match step {
                ScriptStep::InBoot => self.step(EngineEvent::Boot),
                ScriptStep::InQuicOpen { id, kind } => {
                    let ev = QuicEvent::StreamOpened {
                        id: *id,
                        kind: *kind,
                    };
                    self.step(EngineEvent::Quic(ev));
                }
                ScriptStep::InQuicData { id, data, fin } => {
                    let ev = QuicEvent::StreamReadable {
                        id: *id,
                        data: data.as_slice(),
                        fin: *fin,
                    };
                    self.step(EngineEvent::Quic(ev));
                }
                ScriptStep::InTimer(id) => self.step(EngineEvent::TimerFired(*id)),
                ScriptStep::InShutdown => self.step(EngineEvent::Shutdown),

                ScriptStep::Expect(exp) => self.expect_one(exp),
                ScriptStep::ExpectNone => self.expect_none(),
            }
        }

        // If script ends without consuming all pending expectations, fail loudly.
        if !self.pending.is_empty() {
            panic!("script ended with unconsumed commands: {:?}", self.pending);
        }
    }

    fn step<'a>(&mut self, ev: EngineEvent<'a>) {
        let mut sink = VecSink::new();
        self.engine.on_event(ev, &mut sink);

        // Convert to owned commands so expectations can safely inspect bytes.
        for cmd in sink.out {
            self.pending.push(to_owned(cmd));
        }
    }

    fn expect_one(&mut self, exp: &ExpectCommand) {
        if self.pending.is_empty() {
            panic!("expected {:?} but no commands were produced", exp);
        }
        let got = self.pending.remove(0);
        match (exp, got) {
            (ExpectCommand::QuicOpenUni, EngineCommandOwned::QuicOpenUni) => {}
            (
                ExpectCommand::QuicCloseConnection { app_error },
                EngineCommandOwned::QuicCloseConnection { app_error: a },
            ) => {
                assert_eq!(*app_error, a);
            }
            (ExpectCommand::ArmTimer { id }, EngineCommandOwned::ArmTimer { id: got, .. }) => {
                assert_eq!(*id, got);
            }
            (ExpectCommand::CancelTimer { id }, EngineCommandOwned::CancelTimer { id: got }) => {
                assert_eq!(*id, got);
            }
            (
                ExpectCommand::QuicStreamWrite {
                    id,
                    data_prefix,
                    fin,
                },
                EngineCommandOwned::QuicStreamWrite {
                    id: got_id,
                    data,
                    fin: got_fin,
                },
            ) => {
                assert_eq!(*id, got_id);
                assert_eq!(*fin, got_fin);
                assert!(
                    data.starts_with(data_prefix),
                    "write prefix mismatch\nexpected prefix: {:x?}\nactual: {:x?}",
                    data_prefix,
                    data
                );
            }
            (exp, got) => panic!("unexpected command\nexpected: {:?}\n   got: {:?}", exp, got),
        }
    }

    fn expect_none(&mut self) {
        if !self.pending.is_empty() {
            panic!("expected no commands, but got: {:?}", self.pending);
        }
    }
}

fn to_owned<'a>(cmd: EngineCommand<'a>) -> EngineCommandOwned {
    match cmd {
        EngineCommand::Quic(q) => match q {
            QuicCommand::OpenUni { .. } => EngineCommandOwned::QuicOpenUni,
            QuicCommand::StreamWrite { id, data, fin } => EngineCommandOwned::QuicStreamWrite {
                id,
                data: data.to_vec(),
                fin,
            },
            QuicCommand::StreamWriteOwned { id, data, fin } => {
                EngineCommandOwned::QuicStreamWrite { id, data, fin }
            }
            QuicCommand::CloseConnection { app_error } => {
                EngineCommandOwned::QuicCloseConnection { app_error }
            }
            // Add more as engine grows:
            QuicCommand::ResetStream {
                id: _,
                app_error: _,
            } => {
                panic!("ResetStream not yet supported by MockHarness expectations")
            }
            QuicCommand::StopSending {
                id: _,
                app_error: _,
            } => {
                panic!("StopSending not yet supported by MockHarness expectations")
            }
        },
        EngineCommand::ArmTimer {
            id,
            deadline_ms_from_now,
        } => EngineCommandOwned::ArmTimer {
            id,
            deadline_ms_from_now,
        },
        EngineCommand::CancelTimer { id } => EngineCommandOwned::CancelTimer { id },
    }
}
