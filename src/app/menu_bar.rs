use std::sync::Arc;

use crate::{assembler::parser::Parser, simulator::LoadContext};

use super::App;

pub fn show_menu_bar(app: &mut App, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("panel_toolbar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New File...").clicked() {
                    ui.close_menu();
                    app.set_file(None, frame);
                }

                if ui.button("Open File...").clicked() {
                    ui.close_menu();

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("MIPS Assembly Files", &["s"])
                        .pick_file()
                    {
                        app.load_file(path, frame).expect("failed to load file");
                    }
                }

                if ui
                    .add(egui::Button::new("Save...").shortcut_text("Ctrl+S"))
                    .clicked()
                {
                    ui.close_menu();
                    app.save_file(false, frame).expect("failed to save file");
                }

                if ui.button("Save As...").clicked() {
                    ui.close_menu();
                    app.save_file(true, frame).expect("failed to save file");
                }
            });
        });

        ui.horizontal(|ui| {
            let loaded = app.processor.read().loaded;

            if ui.add(egui::Button::new("Assemble")).clicked() {
                let parser = Parser::new(&app.body);
                let parsed = match parser.parse() {
                    Ok(v) => v,
                    Err(e) => {
                        app.output.log.tx.send(format!("Parse error: {e}")).unwrap();
                        return;
                    }
                };

                app.output
                    .log
                    .tx
                    .send("Parsed, loading into the processor...".into())
                    .unwrap();

                let mut processor = app.processor.write();
                let load_ctx = LoadContext::new(&mut processor, &parsed);

                app.pc_line_map = Some(load_ctx.load().expect("failed to load into processor"));

                app.output
                    .log
                    .tx
                    .send("Loaded into processor".into())
                    .unwrap();
            }

            if ui
                .add_enabled(app.processor.read().loaded, egui::Button::new("Reset"))
                .clicked()
            {
                app.processor.write().reset();
                app.output.io.reset();
            }

            if ui.add_enabled(loaded, egui::Button::new("Step")).clicked() {
                let log_tx = app.output.log.tx.clone();
                let proc_arc = Arc::clone(&app.processor);

                let in_tx = app.output.io.in_tx.clone();
                let in_rx = app.output.io.in_rx.clone();

                std::thread::spawn(move || {
                    let mut processor = proc_arc.write();
                    if let Err(e) = processor.step(in_tx, in_rx) {
                        log_tx.send(format!("Step error: {e}")).unwrap();
                    } else {
                        log_tx
                            .send(format!("Stepped (new PC {})", processor.pc))
                            .unwrap();
                    }
                });
            }
        });

        ui.add_space(0.0);
    });

    ctx.input(|i| {
        if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
            app.save_file(false, frame).expect("failed to save file");
        }
    });
}
