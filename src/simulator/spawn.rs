use std::{collections::HashMap, sync::Arc, thread};

use parking_lot::RwLock;

use crate::assembler::parser::Parser;

use super::{LoadContext, Memory, Processor, Register};

/// Messages from the app to the processor.
pub enum ProcMessage {
    /// Reset the processor state.
    Reset,

    /// Load some source code into the processor.
    Load(String),

    /// Step the processor.
    Step,

    /// Send some stdin to the processor.
    Io(String),
}

/// Messages from the processor to the app.
pub enum AppMessage {
    /// Send messages to the program I/O.
    Io(String),

    /// Send messages to the app log.
    Log(String),

    /// Notify the app of the PC addr <-> line relationship.
    PcLines(HashMap<usize, u32>),

    /// Something about the processor state has changed that we want
    /// to see reflected in the app.
    Sync(ProcSync),
}

/// Data to synchronize the app and the processor.
pub struct ProcSync {
    pub pc: usize,
    pub regs: RegSync,
    pub active: bool,
}

pub enum RegSync {
    Diff(HashMap<u8, i32>),
    Set([Register; 32]),
}

pub type ProcTx = crossbeam::channel::Sender<ProcMessage>;
pub type ProcRx = crossbeam::channel::Receiver<ProcMessage>;
pub type AppTx = crossbeam::channel::Sender<AppMessage>;
pub type AppRx = crossbeam::channel::Receiver<AppMessage>;

/// Return result of spawning a processor.
pub struct ProcSpawn {
    pub proc_tx: ProcTx,
    pub app_rx: AppRx,
    pub mem: Arc<RwLock<Memory>>,
}

impl Processor {
    pub fn spawn() -> ProcSpawn {
        let (proc_tx, proc_rx) = crossbeam::channel::unbounded::<ProcMessage>();
        let (app_tx, app_rx) = crossbeam::channel::unbounded::<AppMessage>();

        let mut proc = Self::new(app_tx.clone(), proc_rx.clone());
        let mem = proc.clone_mem_arc();

        thread::spawn(move || {
            // sync once with the editor
            app_tx.send(AppMessage::Sync(proc.sync_hard())).unwrap();

            while let Ok(message) = proc_rx.recv() {
                match message {
                    ProcMessage::Reset => {
                        app_tx.send(AppMessage::Sync(proc.reset())).unwrap();
                    }

                    ProcMessage::Load(body) => {
                        let parser = Parser::new(&body);
                        let parsed = match parser.parse() {
                            Ok(p) => p,
                            Err(e) => {
                                app_tx
                                    .send(AppMessage::Log(format!("Parse error: {e}")))
                                    .unwrap();
                                return;
                            }
                        };
                        match LoadContext::new(&mut proc, &parsed).load() {
                            Ok(map) => {
                                app_tx.send(AppMessage::Sync(proc.sync_hard())).unwrap();
                                app_tx.send(AppMessage::PcLines(map)).unwrap();
                                app_tx
                                    .send(AppMessage::Log("Processor loaded".to_string()))
                                    .unwrap();
                            }
                            Err(e) => {
                                app_tx
                                    .send(AppMessage::Log(format!("Load error: {e}")))
                                    .unwrap();
                            }
                        }
                    }

                    ProcMessage::Step => match proc.step() {
                        Ok(()) => {
                            app_tx.send(AppMessage::Sync(proc.sync())).unwrap();
                            app_tx
                                .send(AppMessage::Log(format!("New PC: {}", proc.pc)))
                                .unwrap();
                        }
                        Err(e) => {
                            app_tx
                                .send(AppMessage::Log(format!("Step error: {e}")))
                                .unwrap();
                        }
                    },

                    ProcMessage::Io(_) => (),
                }
            }
        });

        ProcSpawn {
            proc_tx,
            app_rx,
            mem,
        }
    }

    pub fn io_recv(&mut self) -> Result<String, ()> {
        while let Ok(message) = self.proc_rx.recv() {
            match message {
                ProcMessage::Io(string) => return Ok(string),
                ProcMessage::Step => continue,
                _ => return Err(()),
            }
        }

        Err(())
    }
}
