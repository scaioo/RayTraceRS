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

pub struct InputStream {
    pub stream : Box<dyn BufRead>, //Is it necessary to store this? 
    pub source_location : SourceLocation,
    pub saved_char : Option<char>, //Another way might exist to add this feature. 
    pub saved_location : Option<SourceLocation>,
    pub tabulation: u8
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
