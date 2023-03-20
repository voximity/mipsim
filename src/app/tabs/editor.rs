use egui::Color32;

use crate::{
    app::highlighting::highlight,
    assembler::{
        directive::DIRECTIVE_NAMES,
        inst::{INST_MNEMONICS, PSEUDO_INST_MNEMONICS},
        lexer::{Lexeme, LexemeKind},
    },
    App,
};

pub trait LexemeHint {
    fn show(&self, ui: &mut egui::Ui);
}

pub struct Editor;

impl Editor {
    pub fn show_lexeme_hint(ui: &mut egui::Ui, app: &App, lexeme: &Lexeme) {
        let hint: &dyn LexemeHint = match lexeme {
            Lexeme {
                kind: LexemeKind::Inst,
                ref slice,
                ..
            } => {
                // instructions
                let value = &app.body[slice.clone()];
                let hint = INST_MNEMONICS
                    .get(value)
                    .map(|v| *v as &dyn LexemeHint)
                    .or_else(|| {
                        PSEUDO_INST_MNEMONICS
                            .get(value)
                            .map(|v| *v as &dyn LexemeHint)
                    });

                match hint {
                    Some(hint) => hint,
                    None => return,
                }
            }

            Lexeme {
                kind: LexemeKind::Sect,
                ref slice,
                ..
            } => {
                // directives
                let value = &app.body[slice.clone()];
                let hint = DIRECTIVE_NAMES.get(value).map(|v| *v as &dyn LexemeHint);

                match hint {
                    Some(hint) => hint,
                    None => return,
                }
            }

            _ => return,
        };

        egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("tooltip_lexeme_hover"), |ui| {
            hint.show(ui)
        });
    }

    pub fn show(app: &mut App, ui: &mut egui::Ui) {
        let mut layouter = |ui: &egui::Ui, body: &str, _| {
            let (job, _) = highlight(ui.ctx(), body);
            ui.fonts(|f| f.layout_job(job))
        };

        let editor = egui::TextEdit::multiline(&mut app.body)
            .code_editor()
            .frame(false)
            .hint_text("Write some assembly here...")
            .layouter(&mut layouter)
            .show(ui);

        if editor.response.changed() {
            app.unsaved = true;
        }

        if let Some(row) = app
            .proc
            .pc_lines
            .as_ref()
            .and_then(|map| map.get(&app.proc.pc).copied())
            .and_then(|idx| editor.galley.rows.get(idx as usize))
        {
            let painter = ui.painter_at(editor.response.rect);
            painter.rect_filled(
                row.rect.translate(editor.text_draw_pos.to_vec2()),
                0.0,
                Color32::from_rgba_unmultiplied(255, 0, 0, 20),
            );
        }

        // lexeme hovering
        if let Some(hover_pos) = ui.input(|p| p.pointer.hover_pos()) {
            if ui.clip_rect().contains(hover_pos) && editor.response.rect.contains(hover_pos) {
                let local_pos = hover_pos - editor.response.rect.left_top();
                let hover_cursor = editor.galley.cursor_from_pos(local_pos);

                if editor.galley.rect.contains(local_pos.to_pos2()) {
                    let (_, lexemes) = highlight(ui.ctx(), &app.body);

                    if let Some((_, lexeme)) =
                        lexemes.range(..hover_cursor.ccursor.index).next_back()
                    {
                        Self::show_lexeme_hint(ui, app, lexeme);
                    }
                }
            }
        }
    }
}
