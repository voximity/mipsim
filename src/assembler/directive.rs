use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::app::tabs::editor::LexemeHint;

#[derive(Debug, Clone)]
pub struct Directive {
    name: &'static str,
    desc: &'static str,
}

impl LexemeHint for Directive {
    fn show(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new(self.name).monospace().strong());
        ui.label(self.desc);
    }
}

macro_rules! directives {
    { $( $name:literal : $desc:literal ),*, } => {
        lazy_static! {
            pub static ref DIRECTIVES: Vec<Directive> = vec![
                $(
                    Directive {
                        name: $name,
                        desc: $desc,
                    },
                )*
            ];

            pub static ref DIRECTIVE_NAMES: HashMap<&'static str, &'static Directive> =
                DIRECTIVES.iter().map(|d| (d.name, d)).collect();
        }
    }
}

directives! {
    ".byte":    "Writes a literal byte to the binary.",
    ".half":    "Writes a literal 16-bit integer to the binary.",
    ".word":    "Writes a literal 32-bit integer to the binary.",
    ".asciiz":  "Writes a string followed by a nul terminator to the binary.",
    ".align":   "Aligns the writer to the nearest 2^n-th byte, where n is the number given.",
    ".stringz": "Shorthand for .asciiz STRING .align 2.",
}
