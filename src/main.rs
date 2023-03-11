use std::path::PathBuf;

use assembler::parser::Parser;
use editor::Editor;
use output::Output;
use simulator::Processor;

mod assembler;
mod editor;
mod highlighting;
mod output;
mod simulator;
mod util;

fn main() {
    eframe::run_native(
        "mipsim",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            let app = Box::<App>::default();
            app.output.log.tx.send("Welcome to mipsim!".into()).unwrap();
            app
        }),
    )
    .unwrap();
}

#[derive(Default, Debug)]
pub struct App {
    // editor
    body: String,
    output: Output,
    file: Option<PathBuf>,
    unsaved: bool,

    // simulator
    processor: Processor,
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

        egui::TopBottomPanel::top("panel_toolbar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New File...").clicked() {
                        ui.close_menu();
                        self.set_file(None, frame);
                    }

                    if ui.button("Open File...").clicked() {
                        ui.close_menu();

                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("MIPS Assembly Files", &["s"])
                            .pick_file()
                        {
                            self.load_file(path, frame).expect("failed to load file");
                        }
                    }

                    if ui.button("Save...").clicked() {
                        ui.close_menu();
                        self.save_file(false, frame).expect("failed to save file");
                    }

                    if ui.button("Save As...").clicked() {
                        ui.close_menu();
                        self.save_file(true, frame).expect("failed to save file");
                    }
                });

                ui.menu_button("Assemble", |ui| {
                    if ui.button("Parse test").clicked() {
                        match Parser::new(&self.body).parse() {
                            Ok(v) => self
                                .output
                                .log
                                .tx
                                .send(format!("Parse result: {v:#?}"))
                                .unwrap(),
                            Err(e) => self
                                .output
                                .log
                                .tx
                                .send(format!("Error while parsing: {e}"))
                                .unwrap(),
                        };
                    }
                });
            })
        });

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

        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                self.save_file(false, frame).expect("failed to save file");
            }
        });
    }
}
