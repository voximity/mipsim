use crate::simulator::Registers;

use self::{editor::Editor, output::OutputTab};

use super::App;

pub mod editor;
pub mod memory;
pub mod output;

#[derive(Debug)]
pub enum AppTab {
    Editor,
    Memory,
    Log,
    Io,
    Registers,
}

#[allow(dead_code)]
pub static TABS_LIST: &[AppTab] = &[
    AppTab::Editor,
    AppTab::Memory,
    AppTab::Log,
    AppTab::Io,
    AppTab::Registers,
];

impl egui_dock::TabViewer for App {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            AppTab::Editor => "Editor",
            AppTab::Memory => "Memory",
            AppTab::Log => "Log",
            AppTab::Io => "Program I/O",
            AppTab::Registers => "Registers",
        }
        .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            AppTab::Editor => Editor::show(self, ui),
            AppTab::Memory => self.memory.show(ui, &self.proc.mem),
            AppTab::Log => self.output.show(OutputTab::Log, ui, &self.proc_tx),
            AppTab::Io => self.output.show(OutputTab::Io, ui, &self.proc_tx),
            AppTab::Registers => Registers::show(self, ui),
        }
    }
}
