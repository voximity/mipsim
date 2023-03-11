use std::{
    collections::HashMap,
    io::{self, Seek, SeekFrom, Write},
    mem::transmute,
};

use byteorder::{ReadBytesExt, WriteBytesExt, BE};

use crate::assembler::{
    inst::INST_ADDR_RELATIVE,
    parser::{Directive, Node, NodeImm, Section},
};

use super::{Processor, ADDR_STATIC, ADDR_TEXT};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssembleError<'a> {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("unknown label {0}")]
    UnknownLabel(&'a str),
}

impl Processor {
    /// Load a program into the processor's memory.
    pub fn load<'a>(&mut self, parsed: &[Node<'a>]) -> Result<(), AssembleError<'a>> {
        let mut labels = HashMap::new();
        let mut nodes_with_labels: Vec<(usize, &Node<'a>)> = vec![];

        for node in parsed.iter() {
            match node {
                Node::Section(sec) => {
                    match sec {
                        Section::Data => self.mem.seek(SeekFrom::Start(ADDR_STATIC as u64))?,
                        Section::Text => self.mem.seek(SeekFrom::Start(ADDR_TEXT as u64))?,
                    };
                }

                Node::Label(label) => {
                    labels.insert(label, self.mem.pos());
                }

                Node::Directive(Directive::Byte(byte)) => self.mem.write_u8(*byte)?,
                Node::Directive(Directive::Half(half)) => self.mem.write_u16::<BE>(*half)?,
                Node::Directive(Directive::Word(word)) => self.mem.write_u32::<BE>(*word)?,
                Node::Directive(Directive::Asciiz(string)) => {
                    self.mem.write_all(string.as_bytes())?;
                    self.mem.write_u8(0)?;
                }
                Node::Directive(Directive::Stringz(string)) => {
                    self.mem.write_all(string.as_bytes())?;
                    self.mem.write_u8(0)?;
                    self.mem.align(4);
                }
                Node::Directive(Directive::Align(pow)) => {
                    self.mem.align(2usize.pow(*pow as u32));
                }

                Node::InstR {
                    inst,
                    rs,
                    rt,
                    rd,
                    shamt,
                } => {
                    let encoded = (inst.opcode as u32) << 26
                        | (*rs as u32) << 21
                        | (*rt as u32) << 16
                        | (*rd as u32) << 1
                        | (*shamt as u32) << 6
                        | (inst.func as u32);

                    self.mem.write_u32::<BE>(encoded)?;
                }

                Node::InstI { inst, rs, rt, imm } => {
                    let mut encoded =
                        (inst.opcode as u32) << 26 | (*rs as u32) << 21 | (*rt as u32) << 16;

                    match imm {
                        // TODO: this may overflow the other register data
                        NodeImm::Half(half) => encoded |= *half as u32,
                        NodeImm::Word(word) => encoded |= *word as u16 as u32,
                        NodeImm::Addr(addr) => encoded |= *addr as u16 as u32 >> 2,
                        NodeImm::Label(_) => {
                            nodes_with_labels.push((self.mem.pos(), node));
                        }
                    }

                    self.mem.write_u32::<BE>(encoded)?;
                }

                Node::InstJ { inst, addr } => {
                    let mut encoded = (inst.opcode as u32) << 26;

                    match addr {
                        // TODO: this may overflow the opcode
                        NodeImm::Half(half) => encoded |= *half as u32 >> 2,
                        NodeImm::Word(word) => encoded |= *word >> 2,
                        NodeImm::Addr(addr) => encoded |= *addr >> 2,
                        NodeImm::Label(_) => {
                            nodes_with_labels.push((self.mem.pos(), node));
                        }
                    }

                    self.mem.write_u32::<BE>(encoded)?;
                }
            }
        }

        for (addr, node) in nodes_with_labels {
            match node {
                Node::InstI { inst, imm, .. }
                | Node::InstJ {
                    inst, addr: imm, ..
                } => {
                    self.mem.set_pos(addr);
                    let mut encoded = self.mem.read_u32::<BE>()?;
                    let label = match imm {
                        NodeImm::Label(label) => labels
                            .get(label)
                            .ok_or(AssembleError::UnknownLabel(label))?,
                        _ => unreachable!(),
                    };

                    // handle relative-addressed instructions
                    if INST_ADDR_RELATIVE.contains(&inst.mnemonic) {
                        encoded |= unsafe {
                            transmute::<i32, u32>((*label as i32 - (addr as i32 + 4)) >> 2)
                        };
                    } else {
                        encoded |= *label as u32 >> 2;
                    }

                    self.mem.set_pos(addr);
                    self.mem.write_u32::<BE>(encoded)?;
                }

                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
