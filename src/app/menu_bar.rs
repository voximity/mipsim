use crate::{simulator::ProcMessage, AppContainer};

pub fn show_menu_bar(container: &mut AppContainer, ctx: &egui::Context, frame: &mut eframe::Frame) {
    let app = &mut container.app;

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
                app.proc_tx
                    .send(ProcMessage::Load(app.body.clone()))
                    .unwrap();
            }

            if ui
                .add_enabled(app.proc.active, egui::Button::new("Reset"))
                .clicked()
            {
                app.proc.pc_lines = None;
                app.output.io.reset();
                app.proc_tx.send(ProcMessage::Reset).unwrap();
            }

            if ui
                .add_enabled(app.proc.active, egui::Button::new("Step"))
                .clicked()
            {
                app.proc_tx.send(ProcMessage::Step).unwrap();
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
