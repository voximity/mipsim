use std::{
    collections::HashMap,
    io::{self, Seek, SeekFrom, Write},
    mem::transmute,
};

use byteorder::{ReadBytesExt, WriteBytesExt, BE};

use crate::assembler::{
    inst::{Inst, INST_ADDR_RELATIVE, INST_MNEMONICS},
    parser::{Directive, Node, NodeImm, NodeKind, Section},
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

pub struct LoadContext<'a> {
    /// The processor we are loading into.
    processor: &'a mut Processor,

    /// The parsed nodes.
    parsed: &'a [Node<'a>],

    /// A map of label to address.
    labels: HashMap<&'a str, usize>,

    /// A vector of all nodes with labels.
    nodes_with_labels: Vec<(usize, &'a Node<'a>)>,

    /// A map of PC address to source line.
    addr_lines: Vec<(usize, u32)>,
}

impl<'a> LoadContext<'a> {
    pub fn new(processor: &'a mut Processor, parsed: &'a [Node<'a>]) -> Self {
        Self {
            processor,
            parsed,
            labels: HashMap::new(),
            nodes_with_labels: Vec::new(),
            addr_lines: Vec::new(),
        }
    }

    pub fn load(mut self) -> Result<HashMap<usize, u32>, AssembleError<'a>> {
        self.processor.reset();
        for node in self.parsed.iter() {
            match &node.kind {
                NodeKind::Section(sec) => {
                    match sec {
                        Section::Data => self
                            .processor
                            .mem
                            .seek(SeekFrom::Start(ADDR_STATIC as u64))?,
                        Section::Text => {
                            self.processor.mem.seek(SeekFrom::Start(ADDR_TEXT as u64))?
                        }
                    };
                }

                NodeKind::Label(label) => {
                    self.labels.insert(label, self.processor.mem.pos());
                }

                NodeKind::Directive(Directive::Byte(byte)) => self.processor.mem.write_u8(*byte)?,
                NodeKind::Directive(Directive::Half(half)) => {
                    self.processor.mem.write_u16::<BE>(*half)?
                }
                NodeKind::Directive(Directive::Word(word)) => {
                    self.processor.mem.write_u32::<BE>(*word)?
                }
                NodeKind::Directive(Directive::Asciiz(string)) => {
                    self.processor.mem.write_all(string.as_bytes())?;
                    self.processor.mem.write_u8(0)?;
                }
                NodeKind::Directive(Directive::Stringz(string)) => {
                    self.processor.mem.write_all(string.as_bytes())?;
                    self.processor.mem.write_u8(0)?;
                    self.processor.mem.align(4);
                }
                NodeKind::Directive(Directive::Align(pow)) => {
                    self.processor.mem.align(2usize.pow(*pow as u32));
                }

                NodeKind::InstR {
                    inst,
                    rs,
                    rt,
                    rd,
                    shamt,
                } => {
                    self.load_rtype(node, inst, *rs, *rt, *rd, *shamt)?;
                }

                NodeKind::InstI { inst, rs, rt, imm } => {
                    self.load_itype(node, inst, *rs, *rt, imm)?;
                }

                NodeKind::InstJ { inst, addr } => {
                    self.load_jtype(node, inst, addr)?;
                }

                NodeKind::InstPseudo {
                    inst,
                    rs: _rs,
                    rt,
                    rd: _rd,
                    addr: _addr,
                } => match inst.mnemonic {
                    "la" => {
                        // push an addr ref at the first instruction
                        self.nodes_with_labels
                            .push((self.processor.mem.pos(), node));
                        self.load_itype(node, INST_MNEMONICS["lui"], 0, *rt, &NodeImm::Half(0))?;
                        self.load_itype(node, INST_MNEMONICS["ori"], *rt, *rt, &NodeImm::Half(0))?;
                    }

                    _ => unimplemented!(),
                },
            }
        }

        for (addr, node) in self.nodes_with_labels {
            match &node.kind {
                NodeKind::InstI { inst, imm, .. }
                | NodeKind::InstJ {
                    inst, addr: imm, ..
                } => {
                    self.processor.mem.set_pos(addr);
                    let mut encoded = self.processor.mem.read_u32::<BE>()?;
                    let label = match imm {
                        NodeImm::Label(label) => self
                            .labels
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

                    self.processor.mem.set_pos(addr);
                    self.processor.mem.write_u32::<BE>(encoded)?;
                }

                NodeKind::InstPseudo {
                    inst,
                    addr: inst_addr,
                    rt: _rt,
                    ..
                } => {
                    self.processor.mem.set_pos(addr);

                    match inst.mnemonic {
                        "la" => {
                            let mut lui = self.processor.mem.read_u32::<BE>()?;
                            let mut ori = self.processor.mem.read_u32::<BE>()?;

                            let target_addr = match inst_addr {
                                NodeImm::Label(label) => *self
                                    .labels
                                    .get(label)
                                    .ok_or(AssembleError::UnknownLabel(label))?,
                                NodeImm::Half(half) => *half as usize,
                                NodeImm::Addr(addr) => *addr as usize,
                            };

                            // store the upper 16 bits in lui and the lower 16 bits in ori
                            lui = lui & 0xffff0000 | ((target_addr & 0xffff0000) >> 16) as u32;
                            ori = ori & 0xffff0000 | (target_addr & 0x0000ffff) as u32;

                            self.processor.mem.set_pos(addr);
                            self.processor.mem.write_u32::<BE>(lui)?;
                            self.processor.mem.write_u32::<BE>(ori)?;
                        }

                        _ => unimplemented!(),
                    }
                }

                _ => unreachable!(),
            }
        }

        self.processor.loaded = true;
        Ok(self.addr_lines.into_iter().collect())
    }

    pub fn load_rtype(
        &mut self,
        node: &'a Node,
        inst: &'static Inst,
        rs: u8,
        rt: u8,
        rd: u8,
        shamt: u8,
    ) -> Result<(), AssembleError<'a>> {
        self.addr_lines
            .push((self.processor.mem.pos(), node.lexeme.line));

        let encoded = (inst.opcode as u32) << 26
            | (rs as u32) << 21
            | (rt as u32) << 16
            | (rd as u32) << 11
            | (shamt as u32) << 6
            | (inst.func as u32);

        self.processor.mem.write_u32::<BE>(encoded)?;

        Ok(())
    }

    pub fn load_itype(
        &mut self,
        node: &'a Node,
        inst: &'static Inst,
        rs: u8,
        rt: u8,
        imm: &'a NodeImm,
    ) -> Result<(), AssembleError<'a>> {
        self.addr_lines
            .push((self.processor.mem.pos(), node.lexeme.line));

        let mut encoded = (inst.opcode as u32) << 26 | (rs as u32) << 21 | (rt as u32) << 16;

        match imm {
            // TODO: this may overflow the other register data
            NodeImm::Half(half) => encoded |= *half as u32,
            NodeImm::Addr(addr) => encoded |= *addr as u16 as u32 >> 2,
            NodeImm::Label(_) => {
                self.nodes_with_labels
                    .push((self.processor.mem.pos(), node));
            }
        }

        self.processor.mem.write_u32::<BE>(encoded)?;

        Ok(())
    }

    pub fn load_jtype(
        &mut self,
        node: &'a Node,
        inst: &'static Inst,
        addr: &'a NodeImm,
    ) -> Result<(), AssembleError<'a>> {
        self.addr_lines
            .push((self.processor.mem.pos(), node.lexeme.line));

        let mut encoded = (inst.opcode as u32) << 26;

        match addr {
            // TODO: this may overflow the opcode
            NodeImm::Half(half) => encoded |= *half as u32 >> 2,
            NodeImm::Addr(addr) => encoded |= *addr >> 2,
            NodeImm::Label(_) => {
                self.nodes_with_labels
                    .push((self.processor.mem.pos(), node));
            }
        }

        self.processor.mem.write_u32::<BE>(encoded)?;

        Ok(())
    }
}
