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

                let load_ctx = LoadContext::new(&mut app.processor, &parsed);

                app.pc_line_map = Some(load_ctx.load().expect("failed to load into processor"));

                app.output
                    .log
                    .tx
                    .send("Loaded into processor".into())
                    .unwrap();
            }

            if ui
                .add_enabled(app.processor.loaded, egui::Button::new("Reset"))
                .clicked()
            {
                app.processor.reset();
                app.output.io.reset();
            }

            if ui
                .add_enabled(app.processor.loaded, egui::Button::new("Step"))
                .clicked()
            {
                if !app.processor.loaded {
                    return;
                }

                app.processor.step().expect("failed to step processor");
                app.output
                    .log
                    .tx
                    .send(format!("Stepped processor, new PC: {}", app.processor.pc))
                    .unwrap();
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
