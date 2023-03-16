use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::RwLock;

use crate::simulator::{AppMessage, AppRx, Memory, ProcSync, ProcTx, RegSync, Register};

use self::tabs::{memory::MemoryViewer, output::Output};

pub mod highlighting;
pub mod menu;
pub mod tabs;

#[derive(Debug)]
pub struct App {
    // editor
    pub body: String,
    pub output: Output,
    pub file: Option<PathBuf>,
    pub unsaved: bool,

    // memory
    pub memory: MemoryViewer,

    // processor synchronization
    pub proc: ProcState,
    pub proc_tx: ProcTx,
    pub app_rx: AppRx,
}

#[derive(Debug)]
pub struct ProcState {
    pub regs: [Register; 32],
    pub mem: Arc<RwLock<Memory>>,
    pub pc: usize,
    pub pc_lines: Option<HashMap<usize, u32>>,
    pub active: bool,
}

impl ProcState {
    fn sync(&mut self, sync: ProcSync) {
        self.pc = sync.pc;
        self.active = sync.active;

        match sync.regs {
            RegSync::Set(regs) => {
                self.regs = regs;
            }
            RegSync::Diff(diff) => {
                for (index, value) in diff.into_iter() {
                    self.regs[index as usize] = Register(value);
                }
            }
        }
    }
}

impl App {
    pub fn new(proc_tx: ProcTx, app_rx: AppRx, mem: Arc<RwLock<Memory>>) -> Self {
        Self {
            body: String::new(),
            output: Output::default(),
            file: None,
            unsaved: false,

            memory: MemoryViewer::default(),

            proc: ProcState {
                regs: [Register(0); 32],
                mem,
                pc: 0,
                pc_lines: None,
                active: false,
            },
            proc_tx,
            app_rx,
        }
    }

    fn log(&self, message: impl Into<String>) {
        self.output
            .log
            .tx
            .send(message.into())
            .expect("failed to log message");
    }

    fn set_file(&mut self, path: Option<PathBuf>, frame: &mut eframe::Frame) {
        match path {
            Some(path) => {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    frame.set_window_title(&format!("mipsim - {file_name}"));
                }
                self.file = Some(path);
            }
            None => {
                frame.set_window_title("mipsim");
                self.file = None;
            }
        }
    }

    fn load_file(&mut self, path: PathBuf, frame: &mut eframe::Frame) -> std::io::Result<()> {
        self.body = std::fs::read_to_string(&path)?;
        self.set_file(Some(path), frame);
        self.log("Loaded file");
        Ok(())
    }

    fn save_file(&mut self, save_as: bool, frame: &mut eframe::Frame) -> std::io::Result<()> {
        if !self.unsaved {
            return Ok(());
        }

        match &self.file {
            Some(file) if !save_as => std::fs::write(file, &self.body)?,
            _ => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MIPS Assembly Files", &["s"])
                    .save_file()
                {
                    std::fs::write(&path, &self.body)?;
                    self.set_file(Some(path), frame);
                }
            }
        }

        self.unsaved = false;
        self.output.log.tx.send("File saved".into()).unwrap();

        Ok(())
    }

    pub fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.app_rx.try_recv() {
            match message {
                AppMessage::Sync(sync) => {
                    self.proc.sync(sync);
                    self.memory.request_refresh();
                }
                AppMessage::PcLines(map) => {
                    self.proc.pc_lines = Some(map);
                }
                AppMessage::Io(string) => {
                    self.output.io.add(string);
                }
                AppMessage::Log(string) => {
                    self.output.log.tx.send(string).unwrap();
                }
            }
        }

        // update output buffers
        self.output.log.update();
    }
}

// impl eframe::App for App {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         while let Ok(message) = self.app_rx.try_recv() {
//             match message {
//                 AppMessage::Sync(sync) => {
//                     self.proc.sync(sync);
//                 }
//                 AppMessage::PcLines(map) => {
//                     self.proc.pc_lines = Some(map);
//                 }
//                 AppMessage::Io(string) => {
//                     self.output.io.add(string);
//                 }
//                 AppMessage::Log(string) => {
//                     self.output.log.tx.send(string).unwrap();
//                 }
//             }
//         }

//         // update output buffers
//         self.output.log.update();

//         menu_bar::show_menu_bar(self, ctx, frame);

//         egui::CentralPanel::default()
//             .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
//             .show(ctx, |ui| {
//                 egui_dock::DockArea::new(&mut self.tree)
//                     // .style(style)
//                     .show_inside(ui, self.tab_viewer);
//             });

//         // egui::SidePanel::right("panel_registers")
//         //     .resizable(true)
//         //     .width_range(200.0..=400.0)
//         //     .default_width(200.0)
//         //     .show(ctx, |ui| {
//         //         Registers::show(ui, &self.proc.regs);
//         //     });

//         // egui::TopBottomPanel::bottom("panel_output")
//         //     .resizable(true)
//         //     .height_range(200.0..=400.0)
//         //     .default_height(200.0)
//         //     .show(ctx, |ui| {
//         //         ui.heading("Output");
//         //         self.output.show(ui, &self.proc_tx);
//         //     });

//         // egui::CentralPanel::default().show(ctx, |ui| {
//         //     ui.with_layout(
//         //         egui::Layout::top_down_justified(egui::Align::Min).with_main_justify(true),
//         //         |ui| {
//         //             egui::ScrollArea::both()
//         //                 .auto_shrink([false, false])
//         //                 .show(ui, |ui| {
//         //                     Editor.show(self, ui);
//         //                 })
//         //         },
//         //     );
//         // });
//     }
// }
