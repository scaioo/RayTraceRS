//! This module defines the architecture to read the files that describe the scene to be renderer.

use std::io::BufRead;
use anyhow::anyhow;

// ==========================================
// SourceLocation
// ==========================================
#[derive(Debug, Copy, Clone)]
pub struct SourceLocation {
    pub file_index: usize,
    pub line_number: usize,
    pub col_number: usize,
}

impl SourceLocation {
    fn new(file_index: usize, line_number: usize, col_number: usize) -> Self {
        Self {
            file_index,
            line_number,
            col_number,
        }
    }
}

// ==========================================
// InputStream
// STATUS : DRAFT
// ==========================================

pub struct InputStream<B: BufRead> {
    pub stream: B,
    pub source_location: SourceLocation,
    pub saved_char: Option<char>,
    pub saved_location: Option<SourceLocation>,
    pub tabulation: usize,
}

impl<B: BufRead> InputStream<B> {
    fn new(stream: B, source_location: SourceLocation, tabulation: usize) -> Self {
        // Might change saved_location definition: it depends on the usage of this struct.
        Self {
            stream,
            source_location,
            saved_char: None,
            saved_location: None,
            tabulation,
        }
    }
}

impl<B: BufRead> InputStream<B> {
// We must check elsewhere if the file is finished.
    fn update_pos(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.source_location.line_number += 1;
                self.source_location.col_number = 1;
            }
            '\t' => {
                self.source_location.col_number += self.tabulation;
            }
            _ => {
                self.source_location.col_number += 1;
            }
        }
    }

    pub fn read_byte(&mut self) -> anyhow::Result<Option<char>> {
        match self.saved_char {
            Some(saved_char) => {
                self.saved_char = None;
                self.saved_location = None;
                Ok(Some(saved_char))
            }
            None => {
                let buf = self.stream.fill_buf()?;
                if buf.is_empty() {
                    Ok(None)
                } else {
                    let byte = buf[0];
                    self.stream.consume(1);
                    self.update_pos(byte as char);
                    Ok(Some(byte as char))
                }
            }
        }

    }
    pub fn skip_whitespace(&mut self) -> anyhow::Result<()> {
        panic!("Write function!")
    }
}

// ==========================================
// Tokens
// ==========================================

pub enum TokenKind {
    Keyword(Keyword),
    Identifier(String),
    LiteralString(String),
    LiteralNumber(f32),
    Symbol(char),
    StopToken,
}

pub struct Token {
    pub kind: TokenKind,
    pub loc: SourceLocation,
}

pub enum Keyword {
    // This is to be filled next lesson
}

// ==========================================
// read_token
// ==========================================

impl<B: BufRead> InputStream<B> {
    pub fn read_token(&mut self) -> Token {
        // 1. Skip White spaces

        // 2. If cascade - can we match?
        panic!("Finish writing function!")
    }
}