//! This module defines the architecture to read the files that describe the scene to be renderer.

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
// Tokens
// ==========================================

// This is the backbone: MUST BE UPDATED!!!

// Notes for programmers: I don't know the best way to solve this.
// The best way to understand is try different solutions.
//
// The problem is:
/*
Token:  - Keyword:   - NEW
                    - MATERIAL
                    - PLANE
                    - ...
        - Identifier: - String
        - LiteralString: - String
        - LiteralNumber: - float
        - Symbol:   - String
 */
// Is it better to define a struct { SourceLocation , Token }
// or better (SourceLocation) everywhere in the enum?
// I'd opt for the second one...
pub enum Token {
    Keyword(Keyword, SourceLocation),
    Identifier(String, SourceLocation),
    LiteralString(String, SourceLocation),
    LiteralNumber(f32, SourceLocation),
    Symbol(String, SourceLocation),
    StopToken,
}

pub enum Keyword {}
