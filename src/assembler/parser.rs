use std::{cell::Cell, mem::transmute, num::ParseIntError};

use thiserror::Error;

use crate::simulator::Registers;

use super::{
    inst::{Inst, InstArg, InstType, INSTRUCTIONS},
    lexer::{Lexeme, LexemeKind, Lexer},
};

// pub type SrcLexeme<'a> = (&'a Lexeme, &'a str);

// TODO: make these errors better
#[derive(Debug, Error)]
pub enum ParseError<'a> {
    #[error("unknown section or directive \"{0}\"")]
    UnknownSectDirective(&'a str),
    #[error("expected {0:?}, got {1:?}")]
    ExpectedLexeme(LexemeKind, Option<&'a Lexeme>),
    #[error("unexpected {0:?}")]
    UnexpectedLexeme(&'a Lexeme),
    #[error("integer parse error")]
    ParseIntError(#[from] ParseIntError),
    #[error("string parse error")]
    ParseStringError(&'a Lexeme),
    #[error("unterminated string at {0:?}")]
    UnterminatedString(&'a Lexeme),
    #[error("unknown instruction {0}")]
    UnknownInstruction(&'a str),
    #[error("expected {0}, got {1:?}")]
    ExpectedPunct(&'static str, &'a Lexeme),
    #[error("expected immediate, got {0:?}")]
    ExpectedImm(Option<&'a Lexeme>),
    #[error("unknown register {0:?}")]
    UnknownRegister(&'a Lexeme),
}

/// A node in the assembly tree.
#[derive(Debug, Clone)]
pub enum Node<'a> {
    /// An R-type instruction call.
    InstR {
        inst: &'static Inst,
        rs: u8,
        rt: u8,
        rd: u8,
        shamt: u8,
    },

    /// An I-type instruction call.
    InstI {
        inst: &'static Inst,
        rs: u8,
        rt: u8,
        imm: NodeImm<'a>,
    },

    /// A J-type instruction call.
    InstJ {
        inst: &'static Inst,
        addr: NodeImm<'a>,
    },

    /// A label definition.
    Label(&'a str),

    /// A section, e.g. `.text` or `.data`.
    Section(Section),

    /// A directive, e.g. `.word` or `.asciiz`.
    Directive(Directive),
}

/// An immediate value type for a node.
/// Can be a literal word value, or a label (the address referred to by it).
#[derive(Debug, Clone)]
pub enum NodeImm<'a> {
    /// A literal half.
    Half(u16),

    /// A literal word.
    Word(u32),

    /// An address. Will be shifted right two bits by the assembler.
    Addr(u32),

    /// A label reference. Dereferences to its address.
    Label(&'a str),
}

/// A section in the assembly, e.g. `.text` or `.data`.
#[derive(Debug, Clone)]
pub enum Section {
    Text,
    Data,
}

#[derive(Debug, Clone)]
pub enum Directive {
    Byte(u8),
    Half(u16),
    Word(u32),
    Asciiz(String),
    /// Equivalent to `.asciiz "string" .align 2`.
    Stringz(String),
    Align(u8),
}

#[derive(Debug, Default)]
pub struct Parser<'a> {
    source: &'a str,
    lexemes: Vec<Lexeme>,

    // TODO: does this need interior mutability?
    pos: Cell<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            lexemes: Lexer::new(source).lex(),
            pos: Cell::new(0),
        }
    }

    pub fn pos(&self) -> usize {
        self.pos.get()
    }

    pub fn skip(&self) {
        self.pos.set(self.pos.get() + 1);
    }

    pub fn peek(&'a self) -> Option<(&'a Lexeme, &'a str)> {
        self.lexemes
            .get(self.pos())
            .map(|l| (l, &self.source[l.slice.clone()]))
    }

    pub fn peek_kind(&'a self) -> Option<LexemeKind> {
        self.peek().map(|l| l.0.kind)
    }

    pub fn next(&'a self) -> Option<(&'a Lexeme, &'a str)> {
        match self.peek() {
            Some(l) => {
                self.skip();
                Some(l)
            }
            None => None,
        }
    }

    pub fn next_expect_kind(
        &'a self,
        expected: LexemeKind,
    ) -> Result<(&'a Lexeme, &'a str), ParseError<'a>> {
        match self.peek() {
            Some((lexeme, slice)) if lexeme.kind == expected => {
                self.skip();
                Ok((lexeme, slice))
            }
            x => Err(ParseError::ExpectedLexeme(expected, x.map(|l| l.0))),
        }
    }

    pub fn expect_punct(&'a self, punct: &'static str) -> Result<(), ParseError<'a>> {
        let (lexeme, slice) = self.next_expect_kind(LexemeKind::Punct)?;
        if slice == punct {
            Ok(())
        } else {
            Err(ParseError::ExpectedPunct(punct, lexeme))
        }
    }

    pub fn parse_u8(&'a self) -> Result<u8, ParseError<'a>> {
        let (_, slice) = self.next_expect_kind(LexemeKind::Imm)?;

        if let Some(stripped) = slice.strip_prefix("0x") {
            // hexadecimal
            Ok(u8::from_str_radix(stripped, 16)?)
        } else {
            // try to parse normally
            Ok(str::parse(slice)?)
        }
    }

    pub fn parse_u16(&'a self) -> Result<u16, ParseError<'a>> {
        let (_, slice) = self.next_expect_kind(LexemeKind::Imm)?;

        if let Some(stripped) = slice.strip_prefix("0x") {
            // hexadecimal
            Ok(u16::from_str_radix(stripped, 16)?)
        } else {
            // try to parse normally
            Ok(str::parse(slice)?)
        }
    }

    pub fn parse_i16(&'a self) -> Result<u16, ParseError<'a>> {
        let (_, slice) = self.next_expect_kind(LexemeKind::Imm)?;

        Ok(unsafe { transmute::<i16, u16>(str::parse::<i16>(slice)?) })
    }

    pub fn parse_u32(&'a self) -> Result<u32, ParseError<'a>> {
        let (_, slice) = self.next_expect_kind(LexemeKind::Imm)?;

        if let Some(stripped) = slice.strip_prefix("0x") {
            // hexadecimal
            Ok(u32::from_str_radix(stripped, 16)?)
        } else {
            // try to parse normally
            Ok(str::parse(slice)?)
        }
    }

    pub fn parse_string(&'a self) -> Result<String, ParseError<'a>> {
        let (lex, slice) = self.next_expect_kind(LexemeKind::Imm)?;
        if !slice.starts_with('"') {
            return Err(ParseError::ParseStringError(lex));
        }

        let mut buf = String::new();
        let mut escape = false;
        for c in slice.chars().skip(1) {
            match c {
                '\\' if !escape => {
                    escape = true;
                }
                '"' if !escape => {
                    return Ok(buf);
                }
                _ => {
                    escape = false;
                    buf.push(c);
                }
            }
        }

        Err(ParseError::UnterminatedString(lex))
    }

    pub fn parse_register(&'a self) -> Result<u8, ParseError<'a>> {
        let (lex, slice) = self.next_expect_kind(LexemeKind::Reg)?;

        if let Some(stripped) = slice.strip_prefix('$') {
            Ok(Registers::index(stripped).ok_or(ParseError::UnknownRegister(lex))? as u8)
        } else {
            panic!("bad input to parser from lexer");
        }
    }

    pub fn parse(&'a self) -> Result<Vec<Node<'a>>, ParseError<'a>> {
        let mut nodes = vec![];

        while let Some((lexeme, slice)) = self.next() {
            #[allow(clippy::single_match)]
            match lexeme.kind {
                // sections
                LexemeKind::Sect => {
                    let name = &slice[1..];
                    match name {
                        "data" => nodes.push(Node::Section(Section::Data)),
                        "text" => nodes.push(Node::Section(Section::Text)),

                        // TODO: it is assumed that each of these are unsigned
                        "byte" => nodes.push(Node::Directive(Directive::Byte(self.parse_u8()?))),
                        "half" => nodes.push(Node::Directive(Directive::Half(self.parse_u16()?))),
                        "word" => nodes.push(Node::Directive(Directive::Word(self.parse_u32()?))),

                        "asciiz" => {
                            nodes.push(Node::Directive(Directive::Asciiz(self.parse_string()?)))
                        }
                        "stringz" => {
                            nodes.push(Node::Directive(Directive::Stringz(self.parse_string()?)))
                        }

                        "align" => nodes.push(Node::Directive(Directive::Align(self.parse_u8()?))),

                        _ => return Err(ParseError::UnknownSectDirective(name)),
                    };
                }

                // labels
                LexemeKind::Label => nodes.push(Node::Label(
                    slice
                        .strip_suffix(':')
                        .expect("lexer gave bad input to parser"),
                )),

                // instructions
                LexemeKind::Inst => {
                    let inst = INSTRUCTIONS
                        .get(slice)
                        .ok_or(ParseError::UnknownInstruction(slice))?;
                    let ty_ils = matches!(inst.ty, InstType::Ils);

                    let mut rs = 0;
                    let mut rt = 0;
                    let mut rd = 0;
                    let mut shamt = 0;
                    let mut imm = NodeImm::Half(0);

                    for (i, arg) in inst.args.iter().enumerate() {
                        if matches!(arg, InstArg::None) {
                            break;
                        }

                        if ty_ils {
                            match i {
                                0 => (),
                                1 => self.expect_punct(",")?,
                                2 => self.expect_punct("(")?,
                                _ => unreachable!(),
                            }
                        } else if i > 0 {
                            self.expect_punct(",")?;
                        }

                        match arg {
                            InstArg::None => break,
                            InstArg::Rs => {
                                rs = self.parse_register()?;
                            }
                            InstArg::Rt => {
                                rt = self.parse_register()?;
                            }
                            InstArg::Rd => {
                                rd = self.parse_register()?;
                            }
                            InstArg::Shamt => {
                                shamt = self.parse_u8()?;
                            }
                            InstArg::SImm => match self.peek_kind() {
                                Some(LexemeKind::Imm) => {
                                    imm = NodeImm::Half(self.parse_i16()?);
                                }
                                Some(LexemeKind::Label) => {
                                    imm = NodeImm::Label(self.next().unwrap().1);
                                }
                                _ => return Err(ParseError::ExpectedImm(self.next().map(|l| l.0))),
                            },
                            InstArg::UImm => match self.peek_kind() {
                                Some(LexemeKind::Imm) => {
                                    imm = NodeImm::Half(self.parse_u16()?);
                                }
                                Some(LexemeKind::Label) => {
                                    imm = NodeImm::Label(self.next().unwrap().1);
                                }
                                _ => return Err(ParseError::ExpectedImm(self.next().map(|l| l.0))),
                            },
                            InstArg::Addr => match self.peek_kind() {
                                Some(LexemeKind::Imm) => {
                                    imm = NodeImm::Addr(self.parse_u32()?);
                                }
                                Some(LexemeKind::Label) => {
                                    imm = NodeImm::Label(self.next().unwrap().1);
                                }
                                _ => return Err(ParseError::ExpectedImm(self.next().map(|l| l.0))),
                            },
                        }

                        if ty_ils && i == 2 {
                            self.expect_punct(")")?;
                        }
                    }

                    nodes.push(match inst.ty {
                        InstType::R => Node::InstR {
                            inst,
                            rs,
                            rt,
                            rd,
                            shamt,
                        },
                        InstType::I | InstType::Ils => Node::InstI { inst, rs, rt, imm },
                        InstType::J => Node::InstJ { inst, addr: imm },
                    });
                }

                _ => return Err(ParseError::UnexpectedLexeme(lexeme)),
            }
        }

        Ok(nodes)
    }
}
