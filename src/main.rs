use app::{tabs::AppTab, App};
use egui_dock::NodeIndex;
use simulator::Processor;

mod app;
mod assembler;
mod simulator;
mod util;

fn main() {
    let (proc_tx, app_rx) = Processor::spawn();

    eframe::run_native(
        "mipsim",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            let mut tree = egui_dock::Tree::new(vec![AppTab::Editor]);

            let [node_editor, _] =
                tree.split_right(NodeIndex::root(), 0.8, vec![AppTab::Registers]);

            let [_, _] = tree.split_below(node_editor, 0.8, vec![AppTab::Log, AppTab::Io]);

            let container = Box::new(AppContainer {
                app: App::new(proc_tx, app_rx),
                tree,
            });

            container
                .app
                .output
                .log
                .tx
                .send("Welcome to mipsim!".into())
                .unwrap();
            container
        }),
    )
    .unwrap();
}

pub struct AppContainer {
    pub app: App,
    pub tree: egui_dock::Tree<AppTab>,
}

impl eframe::App for AppContainer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);

        app::menu_bar::show_menu_bar(self, ctx, frame);

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                egui_dock::DockArea::new(&mut self.tree)
                    .style(
                        egui_dock::StyleBuilder::from_egui(&ctx.style())
                            .show_close_buttons(false)
                            .build(),
                    )
                    .show_inside(ui, &mut self.app);
            });
    }
}
