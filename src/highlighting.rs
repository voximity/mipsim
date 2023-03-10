use std::collections::BTreeMap;

use egui::{
    text::LayoutJob,
    util::cache::{ComputerMut, FrameCache},
    Color32, TextFormat,
};

use crate::assembler::lexer::{Lexeme, LexemeKind, Lexer};

#[derive(Default)]
struct Highlighting;

impl LexemeKind {
    pub fn into_text_format(self) -> TextFormat {
        let font_id = egui::FontId::monospace(12.0);
        match self {
            Self::Comment => TextFormat::simple(font_id, Color32::DARK_GRAY),
            Self::Imm => TextFormat::simple(font_id, Color32::LIGHT_GREEN),
            Self::Inst => TextFormat::simple(font_id, Color32::GOLD),
            Self::Label => TextFormat::simple(font_id, Color32::from_rgb(0x46, 0x80, 0xc4)),
            Self::Punct | Self::Whitespace => TextFormat::simple(font_id, Color32::GRAY),
            Self::Reg => TextFormat::simple(font_id, Color32::from_rgb(0x9c, 0xdc, 0xfe)),
            Self::Sect => TextFormat::simple(font_id, Color32::from_rgb(0xc5, 0x86, 0xc0)),
        }
    }
}

pub type HighlightingCtx = (LayoutJob, BTreeMap<usize, Lexeme>);
type HighlightingCache = FrameCache<HighlightingCtx, Highlighting>;

impl ComputerMut<&str, HighlightingCtx> for Highlighting {
    fn compute(&mut self, key: &str) -> HighlightingCtx {
        let mut job = LayoutJob::default();
        let lexemes = Lexer::new(key)
            .with_comments(true)
            .with_whitespace(true)
            .lex();

        for lexeme in &lexemes {
            job.append(
                &key[lexeme.slice.clone()],
                0.0,
                lexeme.kind.into_text_format(),
            );
        }

        (job, Lexer::lexemes_into_btree(lexemes))
    }
}

/// Highlight a bit of text. Memoized, so multiple calls in a frame will not
/// compute anything new.
pub fn highlight(ctx: &egui::Context, text: &str) -> HighlightingCtx {
    ctx.memory_mut(|m| m.caches.cache::<HighlightingCache>().get(text))
}
