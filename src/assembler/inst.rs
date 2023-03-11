use std::{collections::HashMap, str::FromStr};

use egui::{text::LayoutJob, Color32, TextFormat};
use lazy_static::lazy_static;

use super::lexer::{LexemeKind, Lexer};

#[derive(Debug, Clone)]
pub struct Inst {
    /// The instruction mnemonic.
    pub mnemonic: &'static str,

    /// The instruction name,
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

impl Inst {
    pub fn show(&self, ui: &mut egui::Ui) {
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
    Rs,
    Rt,
    Rd,
    Shamt,
    Imm,
    Addr,
    None,
}

impl InstArg {
    pub fn to_name(self) -> &'static str {
        match self {
            Self::Rs => "rs",
            Self::Rt => "rt",
            Self::Rd => "rd",
            Self::Shamt => "shamt",
            Self::Imm => "imm",
            Self::Addr => "addr",
            Self::None => "",
        }
    }

    pub fn to_color(self) -> Color32 {
        match self {
            Self::Rs => Color32::LIGHT_BLUE,
            Self::Rt => Color32::KHAKI,
            Self::Rd => Color32::LIGHT_RED,
            Self::Shamt => Color32::YELLOW,
            Self::Imm => Color32::LIGHT_GREEN,
            Self::Addr => Color32::LIGHT_GREEN,
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
            "imm" | "offset" => Ok(Self::Imm),
            "addr" => Ok(Self::Addr),
            _ => Err(()),
        }
    }
}

macro_rules! instructions {
    { $( $mnemonic:literal $name:literal ($ty:ident, $op:literal/$func:literal) : $desc:literal => [$($arg:ident),*] ),*, } => {
        lazy_static! {
            pub static ref INSTRUCTIONS: HashMap<&'static str, Inst> = vec![
                $(($mnemonic, Inst {
                    mnemonic: $mnemonic,
                    name: $name,
                    ty: InstType::$ty,
                    desc: $desc,
                    args: [$(InstArg::$arg,)*],
                    opcode: $op,
                    func: $func,
                }),)*
            ].into_iter().collect();
        }
    }
}

instructions! {
    // mnem. name                               (T, Opco/Func): description => [Arg1, Arg2, Arg3],
    "add"    "Add"                              (R, 0x00/0x20): "Performs $rd = $rs + $rt." => [Rd, Rs, Rt],
    "addi"   "Add Immediate"                    (I, 0x08/0x00): "Performs $rt = $rs + $imm." => [Rt, Rs, Imm],
    "addiu"  "Add Immediate Unsigned"           (I, 0x09/0x00): "Performs $rt = $rs + $imm, unsigned." => [Rt, Rs, Imm],
    "addu"   "Add Unsigned"                     (R, 0x00/0x21): "Performs $rd = $rs + $rt, unsigned." => [Rd, Rs, Rt],
    "and"    "AND"                              (R, 0x00/0x24): "Performs $rd = $rs & $rt." => [Rd, Rs, Rt],
    "andi"   "AND Immediate"                    (I, 0x0c/0x00): "Performs $rt = $rs & $imm." => [Rt, Rs, Imm],
    "lui"    "Load Upper Immediate"             (I, 0x0f/0x00): "Performs $rt = $imm << 16." => [Rt, Imm, None],
    "nor"    "NOR"                              (R, 0x00/0x27): "Not OR. Performs $rd = ~($rs | $rt)." => [Rs, Rt, Rd],
    "or"     "OR"                               (R, 0x00/0x25): "Performs $rd = $rs | $rt." => [Rd, Rs, Rt],
    "ori"    "OR Immediate"                     (I, 0x0d/0x00): "Performs $rt = $rs | $imm." => [Rt, Rs, Imm],
    "slt"    "Set Less Than"                    (R, 0x00/0x2a): "Performs $rd = $rs < $rt." => [Rd, Rs, Rt],
    "slti"   "Set Less Than Immediate"          (I, 0x0a/0x00): "Performs $rt = $rs < $imm." => [Rt, Rs, Imm],
    "sltiu"  "Set Less Than Immediate Unsigned" (I, 0x0b/0x00): "Performs $rt = $rs < $imm, unsigned." => [Rt, Rs, Imm],
    "sltu"   "Set Less Than Unsigned"           (R, 0x00/0x2b): "Performs $rd = $rs < $rt, unsigned." => [Rd, Rs, Rt],
    "sll"    "Shift Left Logical"               (R, 0x00/0x00): "Performs $rd = $rt << $shamt." => [Rd, Rt, Shamt],
    "sra"    "Shift Right Arithmetic"           (R, 0x00/0x03): "Performs $rd = $rt >> $shamt." => [Rd, Rt, Shamt],
    "srl"    "Shift Right Logical"              (R, 0x00/0x02): "Performs $rd = $rt >> $shamt." => [Rd, Rt, Shamt],
    "sub"    "Subtract"                         (R, 0x00/0x22): "Performs $rd = $rs - $rt." => [Rd, Rs, Rt],
    "subu"   "Subtract Unsigned"                (I, 0x00/0x23): "Performs $rt = $rs - $imm." => [Rt, Rs, Imm],
    "xor"    "XOR"                              (R, 0x00/0x26): "Performs $rd = $rs ^ $rt." => [Rd, Rs, Rt],

    //                                              treated the same as lw
    "la"     "Load Address"                     (I, 0x23/0x00): "Loads $mem($imm) into $rt." => [Rt, Imm, None],
    "lbu"    "Load Byte Unsigned"               (Ils, 0x24/0x00): "Loads $mem($rs + $imm) into $rt." => [Rt, Imm, Rs],
    "lhu"    "Load Half Unsigned"               (Ils, 0x25/0x00): "Loads two bytes at $mem($rs + $imm) into $rt." => [Rt, Imm, Rs],
    "lw"     "Load Word"                        (Ils, 0x23/0x00): "Loads a word at $mem($rs + $imm) into $rt." => [Rt, Imm, Rs],
    "sb"     "Store Byte"                       (Ils, 0x28/0x00): "Store a byte of $rt at $mem($rs + $imm)." => [Rt, Imm, Rs],
    "sh"     "Store Half"                       (Ils, 0x29/0x00): "Store two bytes of $rt at $mem($rs + $imm)." => [Rt, Imm, Rs],
    "sw"     "Store Word"                       (Ils, 0x2b/0x00): "Store a word of $rt at $mem($rs + $imm)." => [Rt, Imm, Rs],

    "beq"    "Branch on Equal"                  (I, 0x04/0x00): "If $rt == $rs, branch to $imm." => [Rt, Rs, Imm],
    "bne"    "Branch on Not Equal"              (I, 0x05/0x00): "If $rt != $rs, branch to $imm." => [Rt, Rs, Imm],
    "j"      "Jump"                             (J, 0x02/0x00): "Jump to $imm." => [Addr, None, None],
    "jal"    "Jump and Link"                    (J, 0x03/0x00): "Set $ra to $pc, then jump to $imm." => [Addr, None, None],
    "jr"     "Jump Register"                    (R, 0x00/0x08): "Jump to the address specified by $rs." => [Rs, None, None],
    "syscall" "System Call"                     (R, 0x00/0x0c): "Perform a system call." => [None, None, None],
}
