use std::{collections::BTreeMap, ops::Range};

use crate::util::IndexedChars;

#[derive(Default, Debug, Clone)]
pub struct Lexeme {
    pub slice: Range<usize>,
    pub line: u32,
    pub kind: LexemeKind,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LexemeKind {
    /// Punctuation (or other characters), e.g. `,`.
    #[default]
    Punct,

    /// A section, e.g. `.text`.
    Sect,

    /// A label, e.g. `label:`.
    Label,

    /// An instruction, e.g. `addi`.
    Inst,

    /// A register, e.g. `$t0`.
    Reg,

    /// An immediate value, e.g. `42`.
    Imm,

    /// A comment, e.g. `; comment`.
    Comment,

    /// Whitespace.
    Whitespace,
}

pub struct Lexer<'a> {
    chars: IndexedChars<'a>,
    whitespace: bool,
    comments: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            chars: IndexedChars::new(text),
            whitespace: false,
            comments: false,
        }
    }

    pub fn with_whitespace(mut self, value: bool) -> Self {
        self.whitespace = value;
        self
    }

    pub fn with_comments(mut self, value: bool) -> Self {
        self.comments = value;
        self
    }

    fn peek_is<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        matches!(self.chars.peek(), Some((_, c)) if f(*c))
    }

    fn take_while<F>(&mut self, base: usize, mut f: F) -> Range<usize>
    where
        F: FnMut(char) -> bool,
    {
        let mut res = base..self.chars.offset();

        while self.peek_is(&mut f) {
            self.chars.next();
            res.end = self.chars.offset();
        }

        res
    }

    fn append_or_add_lexeme(
        &mut self,
        lexemes: &mut Vec<Lexeme>,
        idx: usize,
        line: u32,
        kind: LexemeKind,
    ) {
        match lexemes.last_mut() {
            Some(Lexeme {
                kind: top_kind,
                ref mut slice,
                ..
            }) if *top_kind == kind => {
                slice.end = self.chars.peek_boundary();
            }
            _ => {
                lexemes.push(Lexeme {
                    kind,
                    line,
                    slice: idx..self.chars.peek_boundary(),
                });
            }
        }
    }

    pub fn lex(mut self) -> Vec<Lexeme> {
        let mut lexemes = vec![];
        let mut line = 0u32;
        let mut line_has_inst = false;

        while let Some((idx, c)) = self.chars.next() {
            match c {
                // comments
                ';' | '#' => {
                    let slice = self.take_while(idx, |c| c != '\n');
                    if self.comments {
                        lexemes.push(Lexeme {
                            slice,
                            line,
                            kind: LexemeKind::Comment,
                        });
                    }
                }

                // sections
                '.' if self.peek_is(char::is_alphabetic) => {
                    lexemes.push(Lexeme {
                        slice: self.take_while(idx, char::is_alphabetic),
                        line,
                        kind: LexemeKind::Sect,
                    });
                }

                // registers
                '$' => lexemes.push(Lexeme {
                    slice: self.take_while(idx, char::is_alphanumeric),
                    line,
                    kind: LexemeKind::Reg,
                }),

                // either a label or an instruction
                _ if c.is_alphabetic() => {
                    let mut slice = self.take_while(idx, |c| c == '_' || c.is_alphanumeric());

                    if self.peek_is(|c| c == ':') {
                        // a label marker
                        self.chars.next();
                        slice.end = self.chars.offset();

                        lexemes.push(Lexeme {
                            slice,
                            line,
                            kind: LexemeKind::Label,
                        });
                    } else if line_has_inst {
                        // if this line already had an instruction, this is a
                        // label reference
                        lexemes.push(Lexeme {
                            slice,
                            line,
                            kind: LexemeKind::Label,
                        });
                    } else {
                        // otherwise, this is an instruction
                        line_has_inst = true;
                        lexemes.push(Lexeme {
                            slice,
                            line,
                            kind: LexemeKind::Inst,
                        });
                    }
                }

                '-' if self.peek_is(char::is_numeric) => lexemes.push(Lexeme {
                    slice: self.take_while(idx, char::is_numeric),
                    line,
                    kind: LexemeKind::Imm,
                }),

                // immediates
                _ if c.is_numeric() => {
                    if c == '0' && self.peek_is(|c| c == 'x') {
                        // hexadecimal
                        self.chars.next();

                        lexemes.push(Lexeme {
                            slice: self.take_while(idx, |ref c| {
                                c.is_numeric() || ('a'..='f').contains(c) || ('A'..='F').contains(c)
                            }),
                            line,
                            kind: LexemeKind::Imm,
                        })
                    } else {
                        lexemes.push(Lexeme {
                            slice: self.take_while(idx, char::is_numeric),
                            line,
                            kind: LexemeKind::Imm,
                        });
                    }
                }

                // strings (when used with .asciiz/.stringz)
                '"' => {
                    let mut escape = false;
                    let mut end = false;

                    lexemes.push(Lexeme {
                        slice: self.take_while(idx, |c| {
                            if end {
                                return false;
                            }

                            match c {
                                '\\' if !escape => {
                                    escape = true;
                                }

                                '"' if !escape => {
                                    end = true;
                                }

                                _ => {
                                    escape = false;
                                }
                            }

                            true
                        }),
                        line,
                        kind: LexemeKind::Imm,
                    });
                }

                // whitespace
                _ if c.is_whitespace() => {
                    if self.whitespace {
                        self.append_or_add_lexeme(&mut lexemes, idx, line, LexemeKind::Whitespace);
                    }

                    if c == '\n' {
                        line += 1;
                        line_has_inst = false;
                    }
                }

                // catch all other characters into a Punct lexeme
                _ => self.append_or_add_lexeme(&mut lexemes, idx, line, LexemeKind::Punct),
            }
        }

        lexemes
    }

    pub fn lex_registers_only(mut self) -> Vec<Lexeme> {
        let mut lexemes = vec![];

        while let Some((idx, c)) = self.chars.next() {
            match c {
                // registers
                '$' => lexemes.push(Lexeme {
                    slice: self.take_while(idx, char::is_alphanumeric),
                    line: 0,
                    kind: LexemeKind::Reg,
                }),

                // catch all other characters into a Punct lexeme
                _ => self.append_or_add_lexeme(&mut lexemes, idx, 0, LexemeKind::Punct),
            }
        }

        lexemes
    }

    pub fn lexemes_into_btree(lexemes: Vec<Lexeme>) -> BTreeMap<usize, Lexeme> {
        lexemes
            .into_iter()
            .map(|l| (l.slice.start, l))
            .collect::<_>()
    }
}
