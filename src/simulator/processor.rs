use std::{io, mem::transmute, num::ParseIntError};

use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use crossbeam::channel::RecvError;
use thiserror::Error;

use crate::assembler::inst::{Inst, InstType, INST_OPCODE_FUNC};

use super::{registers::Registers, Memory, ADDR_TEXT, REG_A0, REG_V0};

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum ExecError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("io recv error: {0}")]
    IoRecvError(#[from] RecvError),
    #[error("int parse error: {0}")]
    IntParseError(#[from] ParseIntError),
}

#[derive(Debug)]
pub struct Processor {
    pub regs: Registers,
    pub mem: Memory,
    pub pc: usize,
    pub loaded: bool,
}

#[inline]
fn to_signed_imm(imm: u16) -> i16 {
    unsafe { transmute(imm) }
}

impl Processor {
    pub fn new() -> Self {
        Self {
            regs: Registers::default(),
            mem: Memory::new(),
            pc: ADDR_TEXT,
            loaded: false,
        }
    }

    pub fn reset(&mut self) {
        self.regs = Registers::default();
        self.mem = Memory::new();
        self.pc = ADDR_TEXT;
        self.loaded = false;
    }

    pub fn step(
        &mut self,
        io_tx: crossbeam::channel::Sender<String>,
        io_rx: crossbeam::channel::Receiver<String>,
    ) -> Result<(), ExecError> {
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

                match func {
                    0x0c => match self.regs.get_u32(REG_V0) {
                        // print integer
                        1 => {
                            let _ = io_tx.send(self.regs.get_i32(REG_A0).to_string());
                        }

                        // print string
                        4 => {
                            let string_addr = self.regs.get_u32(REG_A0) as usize;
                            self.mem.set_pos(string_addr);
                            let mut bytes = vec![];

                            loop {
                                match self.mem.read_u8()? {
                                    0 => break,
                                    b => bytes.push(b),
                                }

                                if bytes.len() > 1024 {
                                    // TODO: remove this?
                                    panic!("string too long");
                                }
                            }

                            let _ = io_tx.send(
                                String::from_utf8(bytes)
                                    .unwrap_or_else(|_| "invalid utf-8 string".into()),
                            );
                        }

                        // read int
                        5 => {
                            let input = io_rx.recv()?;
                            let parsed = str::parse::<i32>(&input)?;
                            self.regs.set_i32(REG_V0, parsed);
                        }

                        code => {
                            println!("unimplemented syscall {code}");
                        }
                    },
                    _ => self.call_rtype(data, inst)?,
                }
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
                    InstType::I | InstType::Ils => self.call_itype(data, inst)?,
                    InstType::J => self.call_jtype(data, inst)?,
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }

    pub fn call_rtype(&mut self, encoded: u32, inst: &'static Inst) -> Result<(), ExecError> {
        let rs = ((encoded >> 21) & 0x1f) as u8;
        let rt = ((encoded >> 16) & 0x1f) as u8;
        let rd = ((encoded >> 11) & 0x1f) as u8;
        let shamt = ((encoded >> 6) & 0x1f) as u8;
        let mut inc_pc = true;

        match inst.func {
            // add
            0x20 => self.regs.set_i32(
                rd,
                self.regs.get_i32(rs).wrapping_add(self.regs.get_i32(rt)),
            ),

            // addu
            0x21 => self.regs.set_u32(
                rd,
                self.regs.get_u32(rs).wrapping_add(self.regs.get_u32(rt)),
            ),

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
            0x22 => self.regs.set_i32(
                rd,
                self.regs.get_i32(rs).wrapping_sub(self.regs.get_i32(rt)),
            ),

            // subu
            0x23 => self.regs.set_u32(
                rt,
                self.regs.get_u32(rs).wrapping_sub(self.regs.get_u32(rt)),
            ),

            // xor
            0x26 => self
                .regs
                .set_u32(rd, self.regs.get_u32(rs) ^ self.regs.get_u32(rt)),

            // jr
            0x08 => {
                self.pc = (self.regs.get_u32(rs) as usize) << 2;
                inc_pc = false;
            }

            _ => unreachable!(),
        }

        if inc_pc {
            self.pc += 4;
        }

        Ok(())
    }

    pub fn call_itype(&mut self, encoded: u32, inst: &'static Inst) -> io::Result<()> {
        let rs = ((encoded >> 21) & 0x1f) as u8;
        let rt = ((encoded >> 16) & 0x1f) as u8;
        let imm = (encoded & 0xffff) as u16;
        let mut inc_pc = true;

        match inst.opcode {
            // addi
            0x08 => self.regs.set_i32(
                rt,
                self.regs
                    .get_i32(rs)
                    .wrapping_add(to_signed_imm(imm) as i32),
            ),

            // addiu
            0x09 => self
                .regs
                .set_u32(rt, self.regs.get_u32(rs).wrapping_add(imm as u32)),

            // andi
            0x0c => self.regs.set_u32(rt, self.regs.get_u32(rs) & imm as u32),

            // lui
            0x0f => self.regs.set_u32(rt, (imm as u32) << 16),

            // ori
            0x0d => self.regs.set_u32(rt, self.regs.get_u32(rs) | imm as u32),

            // slti
            0x0a => self.regs.set_u32(
                rt,
                if self.regs.get_i32(rs) < to_signed_imm(imm) as i32 {
                    1
                } else {
                    0
                },
            ),

            // sltiu
            0x0b => self.regs.set_u32(
                rt,
                if self.regs.get_u32(rs) < imm as u32 {
                    1
                } else {
                    0
                },
            ),

            // lbu
            0x24 => {
                self.mem
                    .set_pos((self.regs.get_u32(rs) as i64 + to_signed_imm(imm) as i64) as usize);
                self.regs.set_u32(rt, self.mem.read_u8()? as u32);
            }

            // lhu
            0x25 => {
                self.mem
                    .set_pos((self.regs.get_u32(rs) as i64 + to_signed_imm(imm) as i64) as usize);
                self.regs.set_u32(rt, self.mem.read_u16::<BE>()? as u32);
            }

            // lw
            0x23 => {
                self.mem
                    .set_pos((self.regs.get_u32(rs) as i64 + to_signed_imm(imm) as i64) as usize);
                self.regs.set_u32(rt, self.mem.read_u32::<BE>()?);
            }

            // sb
            0x28 => {
                self.mem
                    .set_pos((self.regs.get_u32(rs) as i64 + to_signed_imm(imm) as i64) as usize);
                self.mem.write_u8(self.regs.get_u32(rt) as u8)?;
            }

            // sh
            0x29 => {
                self.mem
                    .set_pos((self.regs.get_u32(rs) as i64 + to_signed_imm(imm) as i64) as usize);
                self.mem.write_u16::<BE>(self.regs.get_u32(rt) as u16)?;
            }

            // sw
            0x2b => {
                self.mem
                    .set_pos((self.regs.get_u32(rs) as i64 + to_signed_imm(imm) as i64) as usize);
                self.mem.write_u32::<BE>(self.regs.get_u32(rt))?;
            }

            // beq
            0x04 => {
                if self.regs.get_u32(rt) == self.regs.get_u32(rs) {
                    inc_pc = false;
                    self.pc =
                        (self.pc as isize + 4 + ((to_signed_imm(imm) as isize) << 2)) as usize;
                }
            }

            // bne
            0x05 => {
                if self.regs.get_u32(rt) != self.regs.get_u32(rs) {
                    inc_pc = false;
                    self.pc =
                        (self.pc as isize + 4 + ((to_signed_imm(imm) as isize) << 2)) as usize;
                }
            }

            _ => unreachable!(),
        }

        if inc_pc {
            self.pc += 4;
        }

        Ok(())
    }

    pub fn call_jtype(&mut self, encoded: u32, inst: &'static Inst) -> io::Result<()> {
        let addr = encoded & 0x3ffffff;

        match inst.opcode {
            // j
            0x02 => {
                self.pc = (addr as usize) << 2;
            }

            // jal
            0x03 => {
                // set ra to the current pc
                self.regs.set_u32(31, (self.pc >> 2) as u32);
                self.pc = (addr as usize) << 2;
            }

            _ => unreachable!(),
        }

        Ok(())
    }
}
