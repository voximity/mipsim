use std::io;

use byteorder::{ReadBytesExt, BE};

use crate::assembler::inst::{Inst, InstType, INST_OPCODE_FUNC};

use super::{registers::Registers, Memory, ADDR_TEXT};

#[derive(Debug)]
pub struct Processor {
    pub regs: Registers,
    pub mem: Memory,
    pub pc: usize,
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    pub fn new() -> Self {
        Self {
            regs: Registers::default(),
            mem: Memory::new(),
            pc: ADDR_TEXT,
        }
    }

    pub fn step(&mut self) -> io::Result<()> {
        // TODO: use the UI logging

        self.mem.set_pos(self.pc);
        let data = self.mem.read_u32::<BE>()?;
        let opcode = (data >> 26) as u8;

        match opcode {
            // R-type
            0x00 => {
                let func = (data & 0x3f) as u8;
                let inst = match INST_OPCODE_FUNC.get(&(0x00, func)) {
                    Some(inst) => inst,
                    None => {
                        println!("unknown R-type func {func}");
                        return Ok(());
                    }
                };

                self.call_rtype(data, inst);
            }

            // I- or J-type
            _ => {
                let inst = match INST_OPCODE_FUNC.get(&(opcode, 0x00)) {
                    Some(inst) => inst,
                    None => {
                        println!("unknown I- or J-type opcode {opcode}");
                        return Ok(());
                    }
                };

                match inst.ty {
                    InstType::I | InstType::Ils => self.call_itype(data, inst),
                    InstType::J => self.call_jtype(data, inst),
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }

    pub fn call_rtype(&mut self, encoded: u32, inst: &'static Inst) {
        let rs = ((encoded >> 21) & 0x1f) as u8;
        let rt = ((encoded >> 16) & 0x1f) as u8;
        let rd = ((encoded >> 11) & 0x1f) as u8;
        let shamt = ((encoded >> 6) & 0x1f) as u8;
        let mut inc_pc = true;

        match inst.func {
            // add
            0x20 => self
                .regs
                .set_i32(rd, self.regs.get_i32(rs) + self.regs.get_i32(rt)),

            // addu
            0x21 => self
                .regs
                .set_u32(rd, self.regs.get_u32(rs) + self.regs.get_u32(rt)),

            // and
            0x24 => self
                .regs
                .set_u32(rd, self.regs.get_u32(rs) & self.regs.get_u32(rt)),

            // nor
            0x27 => self
                .regs
                .set_u32(rd, !(self.regs.get_u32(rs) | self.regs.get_u32(rt))),

            // or
            0x25 => self
                .regs
                .set_u32(rd, self.regs.get_u32(rs) | self.regs.get_u32(rt)),

            // slt
            0x2a => self.regs.set_i32(
                rd,
                if self.regs.get_i32(rs) < self.regs.get_i32(rt) {
                    1
                } else {
                    0
                },
            ),

            // sltu
            0x2b => self.regs.set_u32(
                rd,
                if self.regs.get_u32(rs) < self.regs.get_u32(rt) {
                    1
                } else {
                    0
                },
            ),

            // sll
            0x00 => self.regs.set_u32(rd, self.regs.get_u32(rs) << shamt as u32),

            // sra
            0x03 => self.regs.set_i32(rd, self.regs.get_i32(rs) >> shamt as i32),

            // srl
            0x02 => self.regs.set_u32(rd, self.regs.get_u32(rs) >> shamt as u32),

            // sub
            0x22 => self
                .regs
                .set_u32(rd, self.regs.get_u32(rs) - self.regs.get_u32(rt)),

            // xor
            0x26 => self
                .regs
                .set_u32(rd, self.regs.get_u32(rs) ^ self.regs.get_u32(rt)),

            // jr
            0x08 => {
                self.pc = self.regs.get_u32(rs) as usize;
                inc_pc = false;
            }

            // syscall
            0x0c => {
                todo!();
            }

            _ => unreachable!(),
        }

        if inc_pc {
            self.pc += 4;
        }
    }

    pub fn call_itype(&mut self, encoded: u32, inst: &'static Inst) {
        // ...
    }

    pub fn call_jtype(&mut self, encoded: u32, inst: &'static Inst) {
        // ...
    }
}
