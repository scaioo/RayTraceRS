//! This module defines the architecture to read the files that describe the scene to be renderer.

use std::io::BufRead;

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
    pub saved_char: Option<char>, //Another way might exist to add this feature.
    pub saved_location: Option<SourceLocation>,
    pub tabulation: u8,
}

impl<B: BufRead> InputStream<B> {
    fn new(stream: B, source_location: SourceLocation, tabulation: u8) -> Self {
        // Might change saved_location definition: it depends on the usage of this struct.
        Self {
            stream,
            source_location,
            saved_char: None,
            saved_location: None,
            tabulation,
        }
    }
    fn update_pos(&self, ch: char) {}
}

// ==========================================
// Tokens
// ==========================================

pub enum Token {
    Keyword(Keyword, SourceLocation),
    Identifier(String, SourceLocation),
    LiteralString(String, SourceLocation),
    LiteralNumber(f32, SourceLocation),
    Symbol(String, SourceLocation),
    StopToken,
}

pub enum Keyword {
    // This is to be filled next lesson
}
