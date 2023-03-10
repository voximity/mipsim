use thiserror::Error;

use super::{
    inst::Inst,
    lexer::{Lexeme, LexemeKind, Lexer},
};

#[derive(Debug, Error)]
pub enum ParseError<'a> {
    #[error("unknown section or directive \"{0}\"")]
    UnknownSectDirective(&'a str),
}

/// A node in the assembly tree.
#[derive(Debug, Clone)]
pub enum Node<'a> {
    /// An instruction call.
    Inst {
        /// The instruction definition.
        inst: &'static Inst,
        rs: u8,
        rt: u8,
        rd: u8,
        imm: NodeImm<'a>,
    },

    /// A label definition.
    Label(&'a str),

    /// A section, e.g. `.text` or `.data`.
    Section(Section),

    /// A directive, e.g. `.word` or `.asciiz`.
    Directive(Directive<'a>),
}

/// An immediate value type for a node.
/// Can be a literal word value, or a label (the address referred to by it).
#[derive(Debug, Clone)]
pub enum NodeImm<'a> {
    /// A literal word.
    Word(u32),

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
pub enum Directive<'a> {
    Byte(u8),
    Half(u16),
    Word(u32),
    Asciiz(&'a str),
    /// Equivalent to `.asciiz "string" .align 2`.
    Stringz(&'a str),
    Align(u8),
}

#[derive(Debug, Default)]
pub struct Parser<'a> {
    source: &'a str,
    lexemes: Vec<Lexeme>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            lexemes: Lexer::new(source).lex(),
            pos: 0,
        }
    }

    pub fn parse(self) -> Result<Vec<Node<'a>>, ParseError<'a>> {
        let mut nodes = vec![];

        while self.pos < self.lexemes.len() {
            let lexeme = &self.lexemes[self.pos];
            let slice = &self.source[lexeme.slice.clone()];

            match lexeme.kind {
                LexemeKind::Sect => {
                    let name = &slice[1..];
                    match name {
                        "data" => nodes.push(Node::Section(Section::Data)),
                        "text" => nodes.push(Node::Section(Section::Text)),

                        "byte" => {
                            // TODO
                        }

                        _ => return Err(ParseError::UnknownSectDirective(name)),
                    };
                }
                _ => (),
            }
        }

        Ok(nodes)
    }
}
