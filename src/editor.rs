use crate::{
    assembler::{
        inst::INSTRUCTIONS,
        lexer::{Lexeme, LexemeKind},
    },
    highlighting::highlight,
    App,
};

pub struct Editor;

impl Editor {
    pub fn show_lexeme_hint(&self, ui: &mut egui::Ui, app: &App, lexeme: &Lexeme) {
        let value = match lexeme {
            Lexeme {
                kind: LexemeKind::Inst,
                ref slice,
            } => &app.body[slice.clone()],
            _ => return,
        };

        let inst_def = match INSTRUCTIONS.get(value) {
            Some(def) => def,
            None => return,
        };

        egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("tooltip_lexeme_hover"), |ui| {
            inst_def.show(ui)
        });
    }

    pub fn show(&self, app: &mut App, ui: &mut egui::Ui) {
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

        // lexeme hovering
        if let Some(hover_pos) = ui.input(|p| p.pointer.hover_pos()) {
            if editor.response.rect.contains(hover_pos) {
                let local_pos = hover_pos - editor.response.rect.left_top();
                let hover_cursor = editor.galley.cursor_from_pos(local_pos);

                if editor.galley.rect.contains(local_pos.to_pos2()) {
                    let (_, lexemes) = highlight(ui.ctx(), &app.body);

                    if let Some((_, lexeme)) =
                        lexemes.range(..hover_cursor.ccursor.index).next_back()
                    {
                        self.show_lexeme_hint(ui, app, lexeme);
                    }
                }
            }
        }
    }
}
