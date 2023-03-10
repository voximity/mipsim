use self::inst::Inst;

pub mod inst;
pub mod lexer;
pub mod parser;

pub struct InstCall {
    inst: &'static Inst,
    args: InstCallArgs,
}

pub enum InstCallArgs {
    R { rs: u8, rt: u8, rd: u8, shamt: u8 },
    I { rs: u8, rt: u8, imm: u16 },
    J(u32),
}

impl InstCall {
    pub fn encode(&self) -> u32 {
        // TODO: this method will incorrectly encode when certain fields
        // TODO: are too big, should we check here or assume the best?
        match &self.args {
            InstCallArgs::R { rs, rt, rd, shamt } => {
                (self.inst.opcode as u32) << 26
                    | (*rs as u32) << 21
                    | (*rt as u32) << 16
                    | (*rd as u32) << 11
                    | (*shamt as u32) << 6
                    | (self.inst.func as u32)
            }
            InstCallArgs::I { rs, rt, imm } => {
                (self.inst.opcode as u32) << 26
                    | (*rs as u32) << 21
                    | (*rt as u32) << 16
                    | (*imm as u32)
            }
            InstCallArgs::J(addr) => (self.inst.opcode as u32) << 26 | *addr,
        }
    }
}
