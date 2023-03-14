use std::sync::Arc;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
};
use parking_lot::RwLock;

use crate::simulator::{Memory, ADDR_MEM_MAX, ADDR_STATIC};

pub const MEMORY_VIEW_BYTES: usize = 256; // 64 words * 4 bytes

#[derive(Debug)]
pub struct MemoryViewer {
    pub offset: usize,
    pub cur_offset: usize,
    pub view: [u8; MEMORY_VIEW_BYTES],
    pub request_refresh: bool,
}

impl Default for MemoryViewer {
    fn default() -> Self {
        Self {
            offset: ADDR_STATIC,
            cur_offset: ADDR_STATIC,
            view: [0u8; MEMORY_VIEW_BYTES],
            request_refresh: true,
        }
    }
}

impl MemoryViewer {
    pub fn request_refresh(&mut self) {
        self.request_refresh = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, mem: &Arc<RwLock<Memory>>) {
        if self.request_refresh || self.offset != self.cur_offset {
            self.request_refresh = false;
            self.cur_offset = self.offset;

            mem.read()
                .read_view(self.cur_offset, &mut self.view)
                .expect("failed to read memory");
        }

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut offset = 0;

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.offset != 0, egui::Button::new("Previous"))
                        .clicked()
                    {
                        self.offset = self.offset.saturating_sub(MEMORY_VIEW_BYTES);
                    }

                    if ui
                        .add_enabled(
                            self.offset + MEMORY_VIEW_BYTES < ADDR_MEM_MAX,
                            egui::Button::new("Next"),
                        )
                        .clicked()
                    {
                        self.offset += MEMORY_VIEW_BYTES;
                    }

                    if self.offset < ADDR_MEM_MAX >> 2 {
                        let button = ui.button("Shift offset left 2 bits");
                        
                        if button.hovered() {
                            egui::show_tooltip_for(ui.ctx(), egui::Id::new("tooltip_memory_shl"), &button.rect, |ui| ui.label("The offset address could be shifted left two bits. If you jumped to this address from a register, it may have been shifted right two bits by the assembler."));
                        }

                        if button.clicked() {
                            self.offset <<= 2;
                        }
                    }
                });

                egui::Grid::new("grid_memory_viewer")
                    .num_columns(3)
                    .striped(true)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Addr");
                        ui.strong("Data");
                        ui.strong("Ascii");
                        ui.end_row();

                        for chunk in self.view.chunks(16) {
                            ui.monospace(format!("{:08x}", self.offset + offset));
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

                                for (i, byte) in chunk.iter().enumerate() {
                                    let mut text = egui::RichText::new(format!(
                                        "{byte:02x}{}",
                                        if i % 4 == 3 { "  " } else { " " }
                                    ))
                                    .monospace();

                                    if *byte == 0 {
                                        text = text.color(egui::Color32::DARK_GRAY);
                                    }

                                    ui.label(text);
                                }
                            });
                            ui.label(ui.memory_mut(|m| {
                                m.caches
                                    .cache::<FrameCache<LayoutJob, ChunkAscii>>()
                                    .get(chunk)
                            }));
                            ui.end_row();

                            offset += chunk.len();
                        }
                    });
            });
    }
}

#[derive(Default)]
struct ChunkAscii;

impl ComputerMut<&[u8], LayoutJob> for ChunkAscii {
    fn compute(&mut self, key: &[u8]) -> LayoutJob {
        let font_id = egui::FontId::monospace(12.0);

        let mut job = LayoutJob::default();
        let mut buf = String::new();

        let add_buf = |job: &mut LayoutJob, buf: &mut String| {
            if buf.is_empty() {
                return;
            }

            job.append(
                &std::mem::take(buf),
                0.0,
                egui::TextFormat {
                    color: egui::Color32::GRAY,
                    font_id: font_id.clone(),
                    ..Default::default()
                },
            );
        };

        for &byte in key {
            match byte {
                32..=126 => buf.push(byte as char),
                b'\n' => {
                    add_buf(&mut job, &mut buf);
                    job.append(
                        "↵",
                        0.0,
                        egui::TextFormat {
                            color: egui::Color32::DARK_GRAY,
                            font_id: font_id.clone(),
                            ..Default::default()
                        },
                    );
                }
                _ => buf.push(' '),
            }
        }

        add_buf(&mut job, &mut buf);

        job
    }
}
