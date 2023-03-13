use egui::TextStyle;

use crate::{
    simulator::{Io, ProcMessage, ProcTx},
    util::ParBuf,
};

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
    pub fn show(&mut self, tab: OutputTab, ui: &mut egui::Ui, proc_tx: &ProcTx) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| match tab {
                OutputTab::Io => {
                    for line in self.io.lines.iter() {
                        ui.monospace(line);
                    }

                    ui.horizontal(|ui| {
                        if !self.io.buf.is_empty() {
                            ui.monospace(&self.io.buf);
                        }
                    });

                    {
                        let input = egui::TextEdit::singleline(&mut self.io.in_buf)
                            .font(TextStyle::Monospace)
                            .desired_width(f32::INFINITY);
                        // .frame(false);
                        let data = input.show(ui);
                        if data.response.lost_focus()
                            && data.response.ctx.input(|i| i.key_down(egui::Key::Enter))
                        {
                            // submit data
                            let string = std::mem::take(&mut self.io.in_buf);
                            self.io.add(format!("{string}\n"));
                            let _ = proc_tx.send(ProcMessage::Io(string));
                        }
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
