//! This module defines the architecture to read the files that describe the scene to be renderer.

use anyhow::anyhow;
use std::io::BufRead;

// ==========================================
// SourceLocation
// ==========================================
#[derive(Debug, Copy, Clone, PartialEq)]
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
    /// The bufreader.
    pub stream: B,
    /// The position of the last-read char.
    pub source_location: SourceLocation,
    /// The saved char.
    pub saved_char: Option<char>,
    /// The SourceLocation of the saved char.
    pub saved_location: Option<SourceLocation>,
    /// The `\t` spaces.
    pub tabulation: usize,
}

impl<B: BufRead> InputStream<B> {
    fn new(stream: B, file_index: usize, tabulation: usize) -> Self {
        // Might change saved_location definition: it depends on the usage of this struct.
        Self {
            stream,
            source_location: SourceLocation {
                file_index,
                line_number: 0, // Is the convention right? Check Tomasi's
                col_number: 0,
            },
            saved_char: None,
            saved_location: None,
            tabulation,
        }
    }
}

impl<B: BufRead> InputStream<B> {
    fn update_pos(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.source_location.line_number += 1;
                self.source_location.col_number = 0;
                // This is because the first element will be the read char.
            }
            '\t' => {
                self.source_location.col_number += self.tabulation;
            }
            _ => {
                self.source_location.col_number += 1;
            }
        }
    }

    pub fn read_char(&mut self) -> anyhow::Result<Option<char>> {
        let ch: char;
        if self.saved_char.is_some() {
            ch = self.saved_char.unwrap();
            self.saved_char = None;
        } else {
            let buf = self.stream.fill_buf()?;
            if buf.is_empty() {
                return Ok(None);
            } else {
                ch = buf[0] as char;
                self.stream.consume(1);
            }
            self.saved_location = Some(self.source_location);
            self.update_pos(ch);
        }

        Ok(Some(ch))
    }
    pub fn unread_char(&mut self, ch: char) -> anyhow::Result<()> {
        if self.saved_char.is_none() {
            self.saved_char = Some(ch);
            self.saved_location = Some(self.source_location);
            Ok(())
        } else {
            Err(anyhow!("Cannot unread more than one character!"))
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

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    static TEST_FILE: &str = "float clock(150)

material sky_material(
    diffuse(uniform(<0, 0, 0>)),
    uniform(<0.7, 0.5, 1>)
)

# Here is a comment

material ground_material(
    diffuse(checkered(<0.3, 0.5, 0.1>,
                      <0.1, 0.2, 0.5>, 4)),
    uniform(<0, 0, 0>)
)

material sphere_material(
    specular(uniform(<0.5, 0.5, 0.5>)),
    uniform(<0, 0, 0>)
)

point_light([10, 10, 10], <1, 1, 1>, 1)

plane (sky_material, translation([0, 0, 100]) * rotation_y(clock))
plane (ground_material, identity)

sphere(sphere_material, translation([0, 0, 1]))

camera(perspective, rotation_z(30) * translation([-4, 0, 1]), 1.0, 1.0)";

    fn setup1() -> InputStream<Cursor<&'static str>> {
        let stream = std::io::Cursor::new(TEST_FILE);
        InputStream::new(stream, 0, 8)
    }

    #[test]
    fn test_constructor() {
        let input_stream = setup1();
        assert_eq!(input_stream.saved_char, None);
        assert!(input_stream.saved_location.is_none());
        assert_eq!(input_stream.tabulation, 8);
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 0);
        assert_eq!(pos.col_number, 0);
        assert_eq!(pos.file_index, 0);
    }
    #[test]
    fn test_update_pos_n() {
        let mut input_stream = setup1();
        input_stream.update_pos('\n');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 1);
        assert_eq!(pos.col_number, 0);
        assert_eq!(pos.file_index, 0);
        assert_eq!(input_stream.source_location, pos);
    }
    #[test]
    fn test_update_pos_t() {
        let mut input_stream = setup1();
        input_stream.update_pos('\t');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 0);
        assert_eq!(pos.col_number, 8);
        assert_eq!(pos.file_index, 0);
    }
    #[test]
    fn test_update_pos_() {
        let mut input_stream = setup1();
        input_stream.update_pos('2');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 0);
        assert_eq!(pos.col_number, 1);
        assert_eq!(pos.file_index, 0);

        input_stream.update_pos('a');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 0);
        assert_eq!(pos.col_number, 2);
        assert_eq!(pos.file_index, 0);
    }

    fn setup2() -> InputStream<Cursor<&'static str>> {
        let text: &str = "";
        InputStream::new(Cursor::new(text), 0, 4)
    }
    #[test]
    fn test_read_char_empty() {
        let mut stream = setup2();
        let output = stream.read_char();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), None);
    }

    #[test]
    fn test_read_char() {
        let mut stream = setup1();
        let output = stream.read_char();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), Some('f'));
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 0, 0));
        assert_eq!(stream.source_location, SourceLocation::new(0, 0, 1));

        let output = stream.read_char();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), Some('l'));
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 0, 1));
        assert_eq!(stream.source_location, SourceLocation::new(0, 0, 2));

        let output = stream.read_char();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), Some('o'));
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 0, 2));
        assert_eq!(stream.source_location, SourceLocation::new(0, 0, 3));
    }

    #[test]
    #[should_panic(expected = "Cannot unread more than one character!")]
    fn test_unread_char_empty() {
        let mut stream = setup1();
        let _ = stream.unread_char('a');
        let _ = stream.unread_char('b').unwrap();
    }
    #[test]
    fn test_unread_char() {
        let mut stream = setup1();
        let ch = stream.read_char().unwrap();
        stream.unread_char(ch.unwrap()).unwrap();
        let ch = stream.read_char().unwrap();
        assert_eq!(ch, Some('f'));

        let ch = stream.read_char().unwrap();
        assert_eq!(ch, Some('l'));
        assert_eq!(stream.saved_char, None);
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 0, 1));
        assert_eq!(stream.source_location, SourceLocation::new(0, 0, 2));

        let str = String::from("oat clock(150)");
        for _ in str.chars() {
            stream.read_char().unwrap();
        }

        let ch = stream.read_char().unwrap();
        assert_eq!(ch, Some('\n'));
        assert_eq!(stream.saved_char, None);
        assert_eq!(
            stream.saved_location.unwrap(),
            SourceLocation::new(0, 0, 16)
        );
        assert_eq!(stream.source_location, SourceLocation::new(0, 1, 0));
    }
}
