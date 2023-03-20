pub mod commands;

use crate::AppContainer;

use self::commands::{CommandCtx, CATEGORIES, COMMANDS, COMMAND_CATEGORIES};

pub fn show_menu_bar(container: &mut AppContainer, ctx: &egui::Context, frame: &mut eframe::Frame) {
    let app = &mut container.app;

    egui::TopBottomPanel::top("panel_toolbar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            for category in CATEGORIES {
                ui.menu_button(category.name(), |ui| {
                    for command in &COMMAND_CATEGORIES[category] {
                        let mut button = egui::Button::new(command.name);
                        if let Some(shortcut) = &command.keybind {
                            button = button.shortcut_text(ui.ctx().format_shortcut(shortcut));
                        }

                        if ui.add(button).clicked() {
                            ui.close_menu();
                            (command.action)(CommandCtx { app, ctx, frame });
                        }
                    }
                });
            }
        });
    });

    ctx.input_mut(|i| {
        if i.keys_down.is_empty() {
            return;
        }

        // TODO: ideally we use a hash map to do this, but egui doesn't
        // TODO: make hashing key shortcuts very easy
        for command in COMMANDS.iter().filter(|c| c.keybind.is_some()) {
            if i.consume_shortcut(command.keybind.as_ref().unwrap()) {
                (command.action)(CommandCtx { app, ctx, frame });
                break;
            }
        }
    });
}
