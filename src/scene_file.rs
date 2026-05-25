//! This module defines the architecture to read the files that describe the scene to be renderer.

use crate::scene_file::TokenKind::StopToken;
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
        // Not included \r handling! Ask Tomasi!!!
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

    /// TODO: write properly this doc!
    /// Returns true if EOF, else returns false.
    pub fn skip_whitespace(&mut self) -> anyhow::Result<bool> {
        // Not included \r handling! Ask Tomasi!!!
        let new_line: String = String::from("\n");

        let op = self.read_char()?;
        match op {
            None => Ok(true),
            Some(a) => {
                let mut ch = a;
                loop {
                    if ch == '#' {
                        loop {
                            if new_line.chars().any(|c| c == ch) {
                                break;
                            } else {
                                match self.read_char()? {
                                    None => return Ok(true),
                                    Some(a) => ch = a,
                                }
                            }
                        }
                    } else if WHITESPACE.contains(ch) {
                        match self.read_char()? {
                            None => return Ok(true),
                            Some(a) => ch = a,
                        }
                    } else {
                        break;
                    }
                }
                self.unread_char(ch)?;
                Ok(false)
            }
        }
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
    NEW,
    MATERIAL,
    PLANE,
    SPHERE,
    DIFFUSE,
    SPECULAR,
    UNIFORM,
    CHECKERED,
    IMAGE,
    IDENTITY,
    TRANSLATION,
    RotationX,
    RotationY,
    RotationZ,
    SCALING,
    CAMERA,
    ORTHOGONAL,
    PERSPECTIVE,
    FLOAT,
    PointLight,
}

static SYMBOLS: &str = "()<>[],*";
static WHITESPACE: &str = " \t\r\n";

// ==========================================
// read_token
// ==========================================

impl<B: BufRead> InputStream<B> {

    pub fn read_string(&mut self, ch: char) -> anyhow::Result<String> {
        if ch.is_alphabetic() || ch == '_' {
            let mut s = String::from(ch);
            let mut new_ch : char;
            loop {
                match self.read_char()? {
                    None => return Ok(s),
                    Some(a) => new_ch = a,
                }
                if new_ch.is_alphanumeric() || new_ch == '_' {
                    s.push(new_ch);
                } else { self.unread_char(new_ch)?; break }
            }
            Ok(s)
        } else {
            Err(anyhow!("Unexpected input character in string!"))
        }

    }

    pub fn parse_string_token(&mut self, ch: char, loc: SourceLocation) -> anyhow::Result<Token> {
        let kind = match self.read_string(ch)?.as_str() {
            "new"         => TokenKind::Keyword(Keyword::NEW),
            "material"    => TokenKind::Keyword(Keyword::MATERIAL),
            "plane"       => TokenKind::Keyword(Keyword::PLANE),
            "sphere"      => TokenKind::Keyword(Keyword::SPHERE),
            "diffuse"     => TokenKind::Keyword(Keyword::DIFFUSE),
            "specular"    => TokenKind::Keyword(Keyword::SPECULAR),
            "uniform"     => TokenKind::Keyword(Keyword::UNIFORM),
            "checkered"   => TokenKind::Keyword(Keyword::CHECKERED),
            "image"       => TokenKind::Keyword(Keyword::IMAGE),
            "identity"    => TokenKind::Keyword(Keyword::IDENTITY),
            "translation" => TokenKind::Keyword(Keyword::TRANSLATION),
            "rotation_x"  => TokenKind::Keyword(Keyword::RotationX),
            "rotation_y"  => TokenKind::Keyword(Keyword::RotationY),
            "rotation_z"  => TokenKind::Keyword(Keyword::RotationZ),
            "scaling"     => TokenKind::Keyword(Keyword::SCALING),
            "camera"      => TokenKind::Keyword(Keyword::CAMERA),
            "orthogonal"  => TokenKind::Keyword(Keyword::ORTHOGONAL),
            "perspective" => TokenKind::Keyword(Keyword::PERSPECTIVE),
            "float"       => TokenKind::Keyword(Keyword::FLOAT),
            "point_light" => TokenKind::Keyword(Keyword::PointLight),
            s             => TokenKind::Identifier(s.to_string()),
        };
        Ok(Token {
            kind, loc
        })
    }

    pub fn read_number(&mut self, ch: char) -> anyhow::Result<f32> {
        let mut new_ch: char = ch;
        let mut s : String = String::new();
        if ch == '-' {
            match self.read_char()? {
                None => return Err(anyhow!("Expected number is not a number: '-' found!")),
                Some('.') => return Err(anyhow!("Expected number is not a number: '-.' found!")),
                Some(a) => { s.push(ch); new_ch = a },
            }
        }
        let mut not_found_dot: bool = true;

        if new_ch.is_ascii_digit() {
            s.push(new_ch);
            loop {
                match self.read_char()? {
                    None => {
                        if s.ends_with('.') {
                            s.push('0');
                        }
                        return Ok(s.parse::<f32>()?);
                    }
                    Some(a) => {
                        if a.is_ascii_digit() {
                            s.push(a);
                        } else if a == '.' && not_found_dot {
                            s.push(a);
                            not_found_dot = false;
                        } else if SYMBOLS.contains(a) || WHITESPACE.contains(a) {
                            self.unread_char(a)?;
                            return Ok(s.parse::<f32>()?)
                        } else {
                            s.push(a);
                            return Err(anyhow!("Unexpected character in number: '{}'!", s.as_str()))
                        }
                    }
                }
            }
        } else {
            Err(anyhow!("Number not found: '{}' found!", s))
        }
    }

    pub fn read_token(&mut self) -> anyhow::Result<Token> {
        let ch: char;
        match self.skip_whitespace() {
            Ok(true) => Ok(Token {
                kind: StopToken,
                loc: self.source_location, // Check it
            }),
            Err(err) => Err(anyhow!("Error reading token: {}", err)),
            Ok(false) => {
                ch = self.read_char()?.unwrap();
                let loc = self.saved_location.unwrap_or(self.source_location);
                if ch.is_alphabetic() {
                    self.parse_string_token(ch, loc)
                } else if ch.is_ascii_digit() || ch == '-' {
                    let num = self.read_number(ch)?;
                    let kind = TokenKind::LiteralNumber(num);
                    let token = Token {
                        kind,
                        loc,
                    };
                    Ok(token)
                } else if SYMBOLS.contains(ch) {
                    let kind = TokenKind::Symbol(ch);
                    let token = Token { kind, loc, };
                    Ok(token)
                } else {
                    Err(anyhow!("NOT ALL POSSIBILITIES COVERED! The char is '{}'", ch))
                }
            }
        }
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

    #[test]
    fn test_skip_whitespace() {
        let text: String =
            String::from("# This is a comment\n# This is a comment too!\n\t   This must be read!");
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        assert_eq!(stream.skip_whitespace().unwrap(), false);
        assert_eq!(stream.source_location, SourceLocation::new(0, 2, 8));
        assert_eq!(stream.saved_char.unwrap(), 'T');
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 2, 8));

        for _ in String::from("This must be read!").chars() {
            let _ = stream.read_char();
        }

        assert_eq!(stream.skip_whitespace().unwrap(), true);
    }

    #[test]
    fn test_skip_whitespace_empty() {
        let text: String = String::from("# This is a comment\n# This is a comment too!\n\t   ");
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        assert_eq!(stream.skip_whitespace().unwrap(), true);
    }

    #[test]
    fn test_read_string() {
        let mut stream = setup1();
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_string(ch).unwrap();
        assert_eq!(s.as_str(), "float");
        assert_eq!(stream.skip_whitespace().unwrap(), false);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_string(ch).unwrap();
        assert_eq!(s.as_str(), "clock");
    }

    #[test]
    #[should_panic(expected = "Unexpected input character in string!")]
    fn test_read_string_err() {
        let mut stream = setup1();
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_string(ch).unwrap();
        let _ = stream.skip_whitespace().unwrap();
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_string(ch).unwrap();
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_string(ch).unwrap();
    }

    #[test]
    fn test_identifier_with_number() {
        let text: String = "a_b_8".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_string(ch).unwrap();
        assert_eq!(s.as_str(), "a_b_8");
    }

    #[test]
    fn test_read_number() {
        let text: String ="832\na".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let number = stream.read_number(ch).unwrap();
        assert_eq!(number, 832.0);
    }

    #[test]
    fn test_read_number_final_dot() {
        let text: String = "832.\t".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let number = stream.read_number(ch).unwrap();
        assert_eq!(number, 832.0);
    }

    #[test]
    fn test_negative_read_number() {
        let text: String ="-8322.3 ".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let number = stream.read_number(ch).unwrap();
        assert_eq!(number, -8322.3);
    }

    #[test]
    #[should_panic(expected = "Expected number is not a number: '-' found!")]
    fn test_read_number_minus_fail() {
        let text: String = "-".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_number(ch).unwrap();
    }

    #[test]
    #[should_panic(expected = "Expected number is not a number: '-.' found!")]
    fn test_read_number_minus_dot_fail() {
        let text: String = "-.".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_number(ch).unwrap();
    }

    #[test]
    #[should_panic(expected = "Unexpected character in number")]
    fn test_read_number_read_fail_character_in_number() {
        let text: String = "8322.3a".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_number(ch).unwrap();
    }

    #[test]
    #[should_panic(expected = "Number not found")]
    fn test_read_number_input_err() {
        let mut stream = setup1();
        let _ = stream.read_number('a').unwrap();
    }
}
