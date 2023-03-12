use std::{collections::HashMap, path::PathBuf};

use crate::simulator::Processor;

use self::{editor::Editor, output::Output};

mod editor;
mod highlighting;
mod menu_bar;
mod output;

#[derive(Debug)]
pub struct App {
    // editor
    pub body: String,
    pub output: Output,
    pub file: Option<PathBuf>,
    pub unsaved: bool,

    // simulator
    pub processor: Processor,
    pub pc_line_map: Option<HashMap<usize, u32>>,
}

impl Default for App {
    fn default() -> Self {
        let output = Output::default();
        let processor = Processor::new(output.io.out_tx.clone());

        Self {
            body: String::new(),
            output,
            file: None,
            unsaved: false,

            processor,
            pc_line_map: None,
        }
    }
}

impl App {
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // update output buffers
        self.output.io.update();
        self.output.log.update();

        menu_bar::show_menu_bar(self, ctx, frame);

        egui::SidePanel::right("panel_registers")
            .resizable(true)
            .width_range(200.0..=400.0)
            .default_width(200.0)
            .show(ctx, |ui| {
                self.processor.regs.show(ui);
            });

        egui::TopBottomPanel::bottom("panel_output")
            .resizable(true)
            .height_range(200.0..=400.0)
            .default_height(200.0)
            .show(ctx, |ui| {
                ui.heading("Output");
                self.output.show(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Min).with_main_justify(true),
                |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            Editor.show(self, ui);
                        })
                },
            );
        });
    }
}
