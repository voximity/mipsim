use crate::util::ParBuf;

#[derive(Debug, Default)]
pub enum OutputTab {
    Io,
    #[default]
    Log,
}

#[derive(Debug)]
pub struct Output {
    pub tab: OutputTab,
    pub io: ParBuf<String>,
    pub log: ParBuf<String>,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            tab: OutputTab::Log,
            io: ParBuf::new().limit(100),
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
            .show(ui, |ui| {
                let buf = match self.tab {
                    OutputTab::Io => &self.io,
                    OutputTab::Log => &self.log,
                };

                for line in buf.iter() {
                    ui.monospace(line);
                }
            });
    }
}
