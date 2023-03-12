use crate::{simulator::Io, util::ParBuf};

#[derive(Debug, Default)]
pub enum OutputTab {
    Io,
    #[default]
    Log,
}

#[derive(Debug)]
pub struct Output {
    pub tab: OutputTab,
    pub io: Io,
    pub log: ParBuf<String>,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            tab: OutputTab::Log,
            io: Io::new(),
            log: ParBuf::new().limit(100),
        }
    }
}

impl Output {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            macro_rules! tabs {
                { $($variant:ident => $name:literal),*, } => {
                    $(
                        if ui.selectable_label(
                            matches!(self.tab, OutputTab::$variant), $name
                        )
                        .clicked() {
                            self.tab = OutputTab::$variant;
                        }
                    )*
                }
            }

            tabs! {
                Io => "Program IO",
                Log => "Logs",
            }
        });

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| match self.tab {
                OutputTab::Io => {
                    for line in self.io.lines.iter() {
                        ui.monospace(line);
                    }

                    if !self.io.buf.is_empty() {
                        ui.monospace(&self.io.buf);
                    }
                }
                OutputTab::Log => {
                    for line in self.log.iter() {
                        ui.monospace(line);
                    }
                }
            });
    }
}
