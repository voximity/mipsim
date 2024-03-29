use std::{collections::HashMap, str::FromStr};

use egui::{text::LayoutJob, Color32, TextFormat};
use lazy_static::lazy_static;

use crate::app::tabs::editor::LexemeHint;

use super::lexer::{LexemeKind, Lexer};

#[derive(Debug, Clone)]
pub struct Inst {
    /// The instruction mnemonic.
    pub mnemonic: &'static str,

    /// The instruction name.
    pub name: &'static str,

    /// The type of instruction (R, I, or J).
    pub ty: InstType,

    /// The instruction description.
    pub desc: &'static str,

    /// The arguments of the instruction.
    pub args: [InstArg; 3],

    /// The instruction opcode.
    pub opcode: u8,

    /// The instruction func value, if the instruction is R-type.
    pub func: u8,
}

#[derive(Debug, Clone)]
pub struct PseudoInst {
    pub mnemonic: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub args: [InstArg; 3],
}

impl LexemeHint for Inst {
    fn show(&self, ui: &mut egui::Ui) {
        let ty_ils = matches!(self.ty, InstType::Ils);

        ui.horizontal(|ui| {
            ui.strong(self.name);

            let mut usage_job = LayoutJob::default();

            // mnemonic
            usage_job.append(self.mnemonic, 0.0, LexemeKind::Inst.into_text_format());

            // arguments
            for (i, arg) in self.args.iter().enumerate() {
                if matches!(arg, InstArg::None) {
                    break;
                }

                if ty_ils {
                    match i {
                        1 => usage_job.append(", ", 0.0, LexemeKind::Punct.into_text_format()),
                        2 => usage_job.append("(", 0.0, LexemeKind::Punct.into_text_format()),
                        _ => (),
                    }
                } else if i > 0 {
                    usage_job.append(", ", 0.0, LexemeKind::Punct.into_text_format());
                }

                usage_job.append(
                    arg.to_name(),
                    if i == 0 { 8.0 } else { 0.0 },
                    TextFormat {
                        color: arg.to_color(),
                        font_id: egui::FontId::monospace(12.0),
                        ..Default::default()
                    },
                );
            }

            if ty_ils {
                usage_job.append(")", 0.0, LexemeKind::Punct.into_text_format());
            }

            ui.label(usage_job);
        });

        let mut desc_job = LayoutJob::default();
        desc_job.wrap.max_width = 100.0;
        let desc_lex = Lexer::new(self.desc).lex_registers_only();
        for lexeme in desc_lex.into_iter() {
            let mut slice = &self.desc[lexeme.slice];

            let format = match lexeme.kind {
                // starts with a $
                LexemeKind::Reg => {
                    slice = &slice[1..];
                    TextFormat {
                        color: str::parse::<InstArg>(slice)
                            .unwrap_or(InstArg::None)
                            .to_color(),
                        font_id: egui::FontId::monospace(12.0),
                        ..Default::default()
                    }
                }

                // everything else
                _ => TextFormat {
                    color: Color32::GRAY,
                    font_id: egui::FontId::proportional(12.0),
                    ..Default::default()
                },
            };

            desc_job.append(slice, 0.0, format);
        }

        ui.label(desc_job);
    }
}

impl LexemeHint for PseudoInst {
    fn show(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.strong(self.name);
            ui.label("(pseudo)");

            let mut usage_job = LayoutJob::default();

            // mnemonic
            usage_job.append(self.mnemonic, 0.0, LexemeKind::Inst.into_text_format());

            // arguments
            for (i, arg) in self.args.iter().enumerate() {
                if matches!(arg, InstArg::None) {
                    break;
                }

                if i > 0 {
                    usage_job.append(", ", 0.0, LexemeKind::Punct.into_text_format());
                }

                usage_job.append(
                    arg.to_name(),
                    if i == 0 { 8.0 } else { 0.0 },
                    TextFormat {
                        color: arg.to_color(),
                        font_id: egui::FontId::monospace(12.0),
                        ..Default::default()
                    },
                );
            }

            ui.label(usage_job);
        });

        let mut desc_job = LayoutJob::default();
        desc_job.wrap.max_width = 100.0;
        let desc_lex = Lexer::new(self.desc).lex_registers_only();
        for lexeme in desc_lex.into_iter() {
            let mut slice = &self.desc[lexeme.slice];

            let format = match lexeme.kind {
                // starts with a $
                LexemeKind::Reg => {
                    slice = &slice[1..];
                    TextFormat {
                        color: str::parse::<InstArg>(slice)
                            .unwrap_or(InstArg::None)
                            .to_color(),
                        font_id: egui::FontId::monospace(12.0),
                        ..Default::default()
                    }
                }

                // everything else
                _ => TextFormat {
                    color: Color32::GRAY,
                    font_id: egui::FontId::proportional(12.0),
                    ..Default::default()
                },
            };

            desc_job.append(slice, 0.0, format);
        }

        ui.label(desc_job);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstType {
    /// R-type.
    R,
    /// I-type.
    I,
    /// I-type, load-store style.
    Ils,
    /// J-type.
    J,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstArg {
    /// The rs register.
    Rs,

    /// The rt register.
    Rt,

    /// The rd register.
    Rd,

    /// The shift amount.
    Shamt,

    /// Signed 16-bit immediate.
    SImm,

    /// Unsigned 16-bit immediate.
    UImm,

    /// An address (immediate or label).
    Addr,

    /// A word (only usable by pseudo instructions).
    Word,

    /// Nothing.
    None,
}

impl InstArg {
    pub fn to_name(self) -> &'static str {
        match self {
            Self::Rs => "rs",
            Self::Rt => "rt",
            Self::Rd => "rd",
            Self::Shamt => "shamt",
            Self::SImm => "imm",
            Self::UImm => "uimm",
            Self::Addr => "addr",
            Self::Word => "word",
            Self::None => "",
        }
    }

    pub fn to_color(self) -> Color32 {
        match self {
            Self::Rs => Color32::LIGHT_BLUE,
            Self::Rt => Color32::KHAKI,
            Self::Rd => Color32::LIGHT_RED,
            Self::Shamt => Color32::YELLOW,
            Self::SImm => Color32::LIGHT_GREEN,
            Self::UImm => Color32::LIGHT_GREEN,
            Self::Addr => Color32::LIGHT_GREEN,
            Self::Word => Color32::LIGHT_GREEN,
            Self::None => Color32::WHITE,
        }
    }
}

impl FromStr for InstArg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rs" => Ok(Self::Rs),
            "rt" => Ok(Self::Rt),
            "rd" => Ok(Self::Rd),
            "shamt" => Ok(Self::Shamt),
            "imm" | "offset" => Ok(Self::SImm),
            "uimm" => Ok(Self::UImm),
            "addr" => Ok(Self::Addr),
            _ => Err(()),
        }
    }
}

macro_rules! instructions {
    { $( $mnemonic:literal $name:literal ($ty:ident, $op:literal/$func:literal) : $desc:literal => [$($arg:ident),*] ),*, } => {
        lazy_static! {
            pub static ref INSTRUCTIONS: Vec<Inst> = vec![
                $(Inst {
                    mnemonic: $mnemonic,
                    name: $name,
                    ty: InstType::$ty,
                    desc: $desc,
                    args: [$(InstArg::$arg,)*],
                    opcode: $op,
                    func: $func,
                },)*
            ];

            pub static ref INST_MNEMONICS: HashMap<&'static str, &'static Inst> =
                INSTRUCTIONS.iter().map(|i| (i.mnemonic, i)).collect();

            pub static ref INST_OPCODE_FUNC: HashMap<(u8, u8), &'static Inst> =
                INSTRUCTIONS.iter().map(|i| ((i.opcode, i.func), i)).collect();
        }
    }
}

macro_rules! pseudo_instructions {
    { $( $mnemonic:literal $name:literal : $desc:literal => [$($arg:ident),*] ),*,} => {
        lazy_static! {
            pub static ref PSEUDO_INSTRUCTIONS: Vec<PseudoInst> = vec![
                $(PseudoInst {
                    mnemonic: $mnemonic,
                    name: $name,
                    desc: $desc,
                    args: [$(InstArg::$arg,)*],
                },)*
            ];

            pub static ref PSEUDO_INST_MNEMONICS: HashMap<&'static str, &'static PseudoInst> =
                PSEUDO_INSTRUCTIONS.iter().map(|i| (i.mnemonic, i)).collect();
        }
    }
}

/// Instruction mnemonics that store addresses as relative to their
/// address, NOT absolutely.
pub static INST_ADDR_RELATIVE: &[&str] = &["beq", "bne"];

instructions! {
    // mnem. name                               (T, Opco/Func): description => [Arg1, Arg2, Arg3],
    "add"    "Add"                              (R, 0x00/0x20): "Performs $rd = $rs + $rt." => [Rd, Rs, Rt],
    "addi"   "Add Immediate"                    (I, 0x08/0x00): "Performs $rt = $rs + $imm." => [Rt, Rs, SImm],
    "addiu"  "Add Immediate Unsigned"           (I, 0x09/0x00): "Performs $rt = $rs + $imm, unsigned." => [Rt, Rs, UImm],
    "addu"   "Add Unsigned"                     (R, 0x00/0x21): "Performs $rd = $rs + $rt, unsigned." => [Rd, Rs, Rt],
    "and"    "AND"                              (R, 0x00/0x24): "Performs $rd = $rs & $rt." => [Rd, Rs, Rt],
    "andi"   "AND Immediate"                    (I, 0x0c/0x00): "Performs $rt = $rs & $imm." => [Rt, Rs, SImm],
    "lui"    "Load Upper Immediate"             (I, 0x0f/0x00): "Performs $rt = $imm << 16." => [Rt, UImm, None],
    "nor"    "NOR"                              (R, 0x00/0x27): "Not OR. Performs $rd = ~($rs | $rt)." => [Rs, Rt, Rd],
    "or"     "OR"                               (R, 0x00/0x25): "Performs $rd = $rs | $rt." => [Rd, Rs, Rt],
    "ori"    "OR Immediate"                     (I, 0x0d/0x00): "Performs $rt = $rs | $imm." => [Rt, Rs, SImm],
    "slt"    "Set Less Than"                    (R, 0x00/0x2a): "Performs $rd = $rs < $rt." => [Rd, Rs, Rt],
    "slti"   "Set Less Than Immediate"          (I, 0x0a/0x00): "Performs $rt = $rs < $imm." => [Rt, Rs, SImm],
    "sltiu"  "Set Less Than Immediate Unsigned" (I, 0x0b/0x00): "Performs $rt = $rs < $imm, unsigned." => [Rt, Rs, UImm],
    "sltu"   "Set Less Than Unsigned"           (R, 0x00/0x2b): "Performs $rd = $rs < $rt, unsigned." => [Rd, Rs, Rt],
    "sll"    "Shift Left Logical"               (R, 0x00/0x00): "Performs $rd = $rt << $shamt." => [Rd, Rt, Shamt],
    "sra"    "Shift Right Arithmetic"           (R, 0x00/0x03): "Performs $rd = $rt >> $shamt." => [Rd, Rt, Shamt],
    "srl"    "Shift Right Logical"              (R, 0x00/0x02): "Performs $rd = $rt >> $shamt." => [Rd, Rt, Shamt],
    "sub"    "Subtract"                         (R, 0x00/0x22): "Performs $rd = $rs - $rt." => [Rd, Rs, Rt],
    "subu"   "Subtract Unsigned"                (R, 0x23/0x00): "Performs $rd = $rs - $rt, unsigned." => [Rd, Rs, Rt],
    "xor"    "XOR"                              (R, 0x00/0x26): "Performs $rd = $rs ^ $rt." => [Rd, Rs, Rt],

    "lbu"    "Load Byte Unsigned"               (Ils, 0x24/0x00): "Loads $mem($rs + $imm) into $rt." => [Rt, SImm, Rs],
    "lhu"    "Load Half Unsigned"               (Ils, 0x25/0x00): "Loads two bytes at $mem($rs + $imm) into $rt." => [Rt, SImm, Rs],
    "lw"     "Load Word"                        (Ils, 0x23/0x00): "Loads a word at $mem($rs + $imm) into $rt." => [Rt, SImm, Rs],
    "sb"     "Store Byte"                       (Ils, 0x28/0x00): "Store a byte of $rt at $mem($rs + $imm)." => [Rt, SImm, Rs],
    "sh"     "Store Half"                       (Ils, 0x29/0x00): "Store two bytes of $rt at $mem($rs + $imm)." => [Rt, SImm, Rs],
    "sw"     "Store Word"                       (Ils, 0x2b/0x00): "Store a word of $rt at $mem($rs + $imm)." => [Rt, SImm, Rs],

    "beq"    "Branch on Equal"                  (I, 0x04/0x00): "If $rt == $rs, branch to $imm." => [Rt, Rs, SImm],
    "bne"    "Branch on Not Equal"              (I, 0x05/0x00): "If $rt != $rs, branch to $imm." => [Rt, Rs, SImm],
    "j"      "Jump"                             (J, 0x02/0x00): "Jump to $addr." => [Addr, None, None],
    "jal"    "Jump and Link"                    (J, 0x03/0x00): "Set $ra to $pc, then jump to $addr." => [Addr, None, None],
    "jr"     "Jump Register"                    (R, 0x00/0x08): "Jump to the address specified by $rs." => [Rs, None, None],
    "syscall" "System Call"                     (R, 0x00/0x0c): "Perform a system call." => [None, None, None],
}

pseudo_instructions! {
    "la"    "Load Address": "Load $addr (literally) into $rt. $addr can be a label name or a literal 32-bit value. Expands into a call to lui and ori." => [Rt, Addr, None],
    "nop"   "No Operation": "Does nothing. Expands to a blank call to sll." => [None, None, None],
    "li"    "Load Immediate": "Loads $imm into $rt." => [Rt, Word, None],
    "move"  "Move": "Copies $rs into $rt." => [Rt, Rs, None],
}
