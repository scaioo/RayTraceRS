// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Lexer for the raytracer scene files.
//!
//! Defines the structures to read and tokenize a scene file,
//! converting raw text into a sequence of [`Token`]s ready
//! for the parser.

use crate::lexer::TokenKind::StopToken;
use anyhow::anyhow;
use std::io::BufRead;

// ==========================================
// SourceLocation
// ==========================================

/// Position of a character within the source files.
///
/// Used to produce precise error messages during parsing.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SourceLocation {
    /// Index of the source file.
    pub file_index: usize,
    /// Line number (starts at 1).
    pub line_number: usize,
    /// Column number (1-indexed: the first character of a line is column 1).
    /// Starts at 0 before any character on the line has been read.
    pub col_number: usize,
}

impl SourceLocation {
    pub fn new(file_index: usize, line_number: usize, col_number: usize) -> Self {
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

/// A source-location-aware input stream.
///
/// Reads characters from a [`BufRead`] one at a time, updating
/// [`source_location`](InputStream::source_location) on each read.
/// Supports a single character of lookahead via [`saved_char`](InputStream::saved_char).
pub struct InputStream<B: BufRead> {
    /// The bufreader.
    pub stream: B,
    /// The position of the last-read char.
    pub source_location: SourceLocation,
    /// The saved char.
    pub saved_char: Option<char>,
    /// The source location stored at the time of the last [`read_char`](InputStream::read_char)
    /// or [`unread_char`](InputStream::unread_char) call. Used by [`read_token`](InputStream::read_token)
    /// to assign the correct position to each token.
    pub saved_location: Option<SourceLocation>,
    /// The `\t` spaces.
    pub tabulation: usize,
    /// The token to look ahead
    pub saved_token: Option<Token>,
}

impl<B: BufRead> InputStream<B> {
    pub fn new(stream: B, file_index: usize, tabulation: usize) -> Self {
        // Might change saved_location definition: it depends on the usage of this struct.
        Self {
            stream,
            source_location: SourceLocation {
                file_index,
                line_number: 1, // Is the convention right? Check Tomasi's
                col_number: 0,
            },
            saved_char: None,
            saved_location: None,
            saved_token: None,
            tabulation,
        }
    }
}

impl<B: BufRead> InputStream<B> {
    /// Updates the current position based on the character just read.
    ///
    /// - `\n` → increments the line number, resets the column to 0
    /// - `\t` → advances the column by [`tabulation`](InputStream::tabulation) spaces
    /// - any other character → advances the column by 1
    ///
    /// This function does not handle `\r`.
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

    /// Reads the next character from the stream.
    ///
    /// If a character was pushed back via [`unread_char`](Self::unread_char),
    /// that character is returned without consuming the underlying stream.
    /// Updates [`source_location`] after each read from the stream.
    ///
    /// Returns `Ok(None)` at end of file.
    ///
    /// # Errors
    /// Propagates I/O errors from the underlying [`BufRead`].
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

    /// Pushes a character back onto the stream, so that the next call to
    /// [`read_char`](Self::read_char) returns it.
    ///
    /// # Errors
    /// Returns an error if a character is already saved
    /// (only one character of lookahead is supported).
    pub fn unread_char(&mut self, ch: char) -> anyhow::Result<()> {
        if self.saved_char.is_none() {
            self.saved_char = Some(ch);
            self.saved_location = Some(self.source_location);
            Ok(())
        } else {
            Err(anyhow!("Cannot unread more than one character!"))
        }
    }

    /// Advances the stream past whitespace and comments (`#` through end of line).
    ///
    /// After returning, the first non-whitespace character has been read
    /// and saved in [`saved_char`](InputStream::saved_char).
    ///
    /// Returns `Ok(true)` if end of file is reached, `Ok(false)` otherwise.
    ///
    /// # Errors
    /// Propagates I/O errors from [`read_char`](Self::read_char).
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

    /// Reads a quoted string literal from the stream.
    ///
    /// `ch` must be either `'` or `"` and acts as the opening delimiter;
    /// the function reads characters until the same delimiter is found again.
    /// The delimiter itself is not included in the returned string.
    ///
    /// # Errors
    /// - `ch` is not `'` or `"`
    /// - end of file is reached before the closing delimiter
    pub fn read_string_literal(&mut self, ch: char) -> anyhow::Result<String> {
        if ch != '\'' && ch != '"' {
            Err(anyhow!(
                "Wrong input character in read_string!\n Read char: '{}'",
                ch
            ))
        } else {
            let mut s = String::new();
            loop {
                match self.read_char()? {
                    None => return Err(anyhow!("Unexpected EOF!")),
                    Some(a) => {
                        if a == ch {
                            return Ok(s);
                        } else {
                            s.push(a);
                        }
                    }
                }
            }
        }
    }

    /// Reads a sequence of alphanumeric characters or `_`, starting from `ch`.
    ///
    /// `ch` must be alphabetic or `_`; subsequent characters may also be digits.
    /// Reading stops at the first invalid character, which is pushed back
    /// via [`unread_char`](Self::unread_char).
    ///
    /// # Errors
    /// Returns an error if `ch` is not alphabetic or `_`.
    pub fn read_identifier(&mut self, ch: char) -> anyhow::Result<String> {
        if ch.is_alphabetic() || ch == '_' {
            let mut s = String::from(ch);
            let mut new_ch: char;
            loop {
                match self.read_char()? {
                    None => return Ok(s),
                    Some(a) => new_ch = a,
                }
                if new_ch.is_alphanumeric() || new_ch == '_' {
                    s.push(new_ch);
                } else {
                    self.unread_char(new_ch)?;
                    break;
                }
            }
            Ok(s)
        } else {
            Err(anyhow!("Unexpected input character in string!"))
        }
    }

    /// Reads an identifier or keyword token starting from `ch`.
    ///
    /// Calls [`read_identifier`](Self::read_identifier) to collect the full word,
    /// then matches it against the reserved keywords of the scene language.
    /// Produces a [`TokenKind::Keyword`] if the word is reserved
    /// (e.g. `"new"`, `"sphere"`, `"camera"`), or a [`TokenKind::Identifier`]
    /// otherwise.
    ///
    /// `loc` is the position of the first character of the token.
    pub fn parse_identifier_token(
        &mut self,
        ch: char,
        loc: SourceLocation,
    ) -> anyhow::Result<Token> {
        let kind = match self.read_identifier(ch)?.as_str() {
            "new" => TokenKind::Keyword(Keyword::New),
            "material" => TokenKind::Keyword(Keyword::Material),
            "plane" => TokenKind::Keyword(Keyword::Plane),
            "sphere" => TokenKind::Keyword(Keyword::Sphere),
            "box" => TokenKind::Keyword(Keyword::Box),
            "cylinder" => TokenKind::Keyword(Keyword::Cylinder),
            "simple_mesh" => TokenKind::Keyword(Keyword::SimpleMesh),
            "diffuse" => TokenKind::Keyword(Keyword::Diffuse),
            "specular" => TokenKind::Keyword(Keyword::Specular),
            "uniform" => TokenKind::Keyword(Keyword::Uniform),
            "checkered" => TokenKind::Keyword(Keyword::Checkered),
            "image" => TokenKind::Keyword(Keyword::Image),
            "gradient" => TokenKind::Keyword(Keyword::Gradient),
            "identity" => TokenKind::Keyword(Keyword::Identity),
            "translation" => TokenKind::Keyword(Keyword::Translation),
            "rotation_x" => TokenKind::Keyword(Keyword::RotationX),
            "rotation_y" => TokenKind::Keyword(Keyword::RotationY),
            "rotation_z" => TokenKind::Keyword(Keyword::RotationZ),
            "scaling" => TokenKind::Keyword(Keyword::Scaling),
            "camera" => TokenKind::Keyword(Keyword::Camera),
            "orthogonal" => TokenKind::Keyword(Keyword::Orthogonal),
            "perspective" => TokenKind::Keyword(Keyword::Perspective),
            "float" => TokenKind::Keyword(Keyword::Float),
            "point" => TokenKind::Keyword(Keyword::Point),
            "point_light" => TokenKind::Keyword(Keyword::PtLightSource),
            "spherical_light" => TokenKind::Keyword(Keyword::SphLightSource),
            "true" | "True" => TokenKind::Keyword(Keyword::True),
            "false" | "False" => TokenKind::Keyword(Keyword::False),
            "black" | "Black" | "BLACK" => TokenKind::Keyword(Keyword::Black),
            "white" | "White" | "WHITE" => TokenKind::Keyword(Keyword::White),
            s => TokenKind::Identifier(s.to_string()),
        };
        Ok(Token { kind, loc })
    }

    /// Reads a floating-point number (including negative values) starting from `ch`.
    ///
    /// Handles integers, decimals, and the `-` sign. A trailing dot (e.g. `"832."`)
    /// is normalized to `832.0`. Reading stops at the first whitespace or symbol,
    /// which is pushed back onto the stream.
    ///
    /// # Errors
    /// - `ch` is `-` but is not followed by a digit
    /// - non-numeric characters are found within the number
    pub fn read_number(&mut self, ch: char) -> anyhow::Result<f32> {
        let mut new_ch: char = ch;
        let mut s: String = String::new();
        if ch == '-' {
            match self.read_char()? {
                None => return Err(anyhow!("Expected number is not a number: '-' found!")),
                Some('.') => return Err(anyhow!("Expected number is not a number: '-.' found!")),
                Some(a) => {
                    s.push(ch);
                    new_ch = a
                }
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
                            return Ok(s.parse::<f32>()?);
                        } else {
                            s.push(a);
                            return Err(anyhow!(
                                "Unexpected character in number: '{}'!",
                                s.as_str()
                            ));
                        }
                    }
                }
            }
        } else {
            Err(anyhow!("Number not found: '{}' found!", s))
        }
    }

    /// Saves a token that has just been read so that it can be returned on the next call
    pub fn unread_token(&mut self, token: Token) -> anyhow::Result<()> {
        if self.saved_token.is_none() {
            self.saved_token = Some(token);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "I can't mark more than one token as unread!"
            ))
        }
    }

    /// Reads the next token from the stream.
    ///
    /// Skips whitespace and comments, then classifies the current character:
    /// - letter or `_` → [`TokenKind::Keyword`] or [`TokenKind::Identifier`]
    /// - digit or `-` → [`TokenKind::LiteralNumber`]
    /// - single or double quotes -> [`TokenKind::LiteralString`]
    /// - symbol (`()[]<>,*`) → [`TokenKind::Symbol`]
    /// - end of file → [`TokenKind::StopToken`]
    ///
    /// # Errors
    /// Returns an error if the character does not belong to any of the
    /// categories above, or if an internal read fails.
    pub fn read_token(&mut self) -> anyhow::Result<Token> {
        if let Some(token) = self.saved_token.take() {
            return Ok(token);
        }
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
                if ch.is_alphabetic() || ch == '_' {
                    self.parse_identifier_token(ch, loc)
                } else if ch.is_ascii_digit() || ch == '-' {
                    let num = self.read_number(ch)?;
                    let kind = TokenKind::LiteralNumber(num);
                    Ok(Token { kind, loc })
                } else if SYMBOLS.contains(ch) {
                    let kind = TokenKind::Symbol(ch);
                    Ok(Token { kind, loc })
                } else if ch == '"' || ch == '\'' {
                    let s = self.read_string_literal(ch)?;
                    let kind = TokenKind::LiteralString(s);
                    Ok(Token { kind, loc })
                } else {
                    Err(anyhow!(
                        "Unexpected character '{}' at {}:{}",
                        ch,
                        self.source_location.line_number,
                        self.source_location.col_number
                    ))
                }
            }
        }
    }
}

// ==========================================
// Token and TokenKind
// ==========================================

/// The type of token recognized by the lexer.
#[derive(Debug, PartialEq)]
pub enum TokenKind {
    /// A reserved keyword of the scene language (e.g. `new`, `sphere`, `camera`).
    Keyword(Keyword),
    /// A user-defined identifier (e.g. a material name).
    Identifier(String),
    /// A quoted string literal, delimited by \'` or `"`.
    LiteralString(String),
    /// A floating-point literal (e.g. `3.14`, `-1.0`).
    LiteralNumber(f32),
    /// A single symbol belonging to `()[]<>,*`.
    Symbol(char),
    /// End-of-file sentinel.
    StopToken,
}

/// A token produced by the lexer, carrying its type and position in the source.
#[derive(Debug, PartialEq)]
pub struct Token {
    /// Type and value of the token.
    pub kind: TokenKind,
    /// Position of the token's first character in the source file.
    pub loc: SourceLocation,
}

/// Reserved keywords of the scene language.
///
/// Identify the constructs of a scene file:
/// object types, materials, transformations, and cameras.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Keyword {
    New,
    Material,
    Plane,
    Sphere,
    Box,
    SimpleMesh,
    Cylinder,
    Diffuse,
    Specular,
    Uniform,
    Checkered,
    Image,
    Gradient,
    Identity,
    Translation,
    RotationX,
    RotationY,
    RotationZ,
    Scaling,
    Camera,
    Orthogonal,
    Perspective,
    Float,
    Point,
    PtLightSource,
    SphLightSource,
    True,
    False,
    Black,
    White,
}

static SYMBOLS: &str = "()<>[],*";
static WHITESPACE: &str = " \t\r\n";

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Keyword::{
        Box, Cylinder, False, Float, Gradient, Material, Point, PtLightSource, SimpleMesh,
        SphLightSource, True,
    };
    use crate::lexer::TokenKind;
    use crate::lexer::TokenKind::Keyword;
    use std::io::Cursor;

    static TEST_FILE: &str = "float clock(150)

# This is the demo image from v0.3.0 - polish this file before merging PR

# Sky dome

material sky_material(
uniform(<0.5, 0.9, 1.0>),
diffuse(),
uniform(<0.5, 0.9, 1.0>)
)

sphere(
sky_material,
scaling([200, 200, 200]) * translation([0, 0, 0.4])
)

# Checkered floor

material floor_material(
checkered(
<0.3, 0.5, 0.1>,
<0.1, 0.2, 0.5>,
5
),
diffuse(),
uniform(<0, 0, 0>)
)

plane(
floor_material,
identity, True
)

# Diffuse sphere

material diffusive_sphere_material(
uniform(<0.3, 0.4, 0.8>),
diffuse(),
uniform(<0.0, 0.0, 0.0>)
)

sphere(
diffusive_sphere_material,
translation([0.0, 0.0, 1.0])
)

# Mirror sphere

material mirror_material(
uniform(<0.6, 0.2, 0.3>),
specular(),
uniform(<0, 0, 0>)
)

sphere(
mirror_material,
translation([1.0, 2.5, 0.0])
)

# Camera

camera(
perspective,
translation([-1, 0, 1]),
1.0
)

# End of world description!";

    fn setup1() -> InputStream<Cursor<&'static str>> {
        let stream = Cursor::new(TEST_FILE);
        InputStream::new(stream, 0, 8)
    }

    #[test]
    fn test_constructor() {
        let input_stream = setup1();
        assert_eq!(input_stream.saved_char, None);
        assert!(input_stream.saved_location.is_none());
        assert_eq!(input_stream.tabulation, 8);
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 1);
        assert_eq!(pos.col_number, 0);
        assert_eq!(pos.file_index, 0);
    }
    #[test]
    fn test_update_pos_n() {
        let mut input_stream = setup1();
        input_stream.update_pos('\n');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 2);
        assert_eq!(pos.col_number, 0);
        assert_eq!(pos.file_index, 0);
        assert_eq!(input_stream.source_location, pos);
    }
    #[test]
    fn test_update_pos_t() {
        let mut input_stream = setup1();
        input_stream.update_pos('\t');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 1);
        assert_eq!(pos.col_number, 8);
        assert_eq!(pos.file_index, 0);
    }
    #[test]
    fn test_update_pos_() {
        let mut input_stream = setup1();
        input_stream.update_pos('2');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 1);
        assert_eq!(pos.col_number, 1);
        assert_eq!(pos.file_index, 0);

        input_stream.update_pos('a');
        let pos = input_stream.source_location;
        assert_eq!(pos.line_number, 1);
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
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 1, 0));
        assert_eq!(stream.source_location, SourceLocation::new(0, 1, 1));

        let output = stream.read_char();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), Some('l'));
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 1, 1));
        assert_eq!(stream.source_location, SourceLocation::new(0, 1, 2));

        let output = stream.read_char();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), Some('o'));
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 1, 2));
        assert_eq!(stream.source_location, SourceLocation::new(0, 1, 3));
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
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 1, 1));
        assert_eq!(stream.source_location, SourceLocation::new(0, 1, 2));

        let str = String::from("oat clock(150)");
        for _ in str.chars() {
            stream.read_char().unwrap();
        }

        let ch = stream.read_char().unwrap();
        assert_eq!(ch, Some('\n'));
        assert_eq!(stream.saved_char, None);
        assert_eq!(
            stream.saved_location.unwrap(),
            SourceLocation::new(0, 1, 16)
        );
        assert_eq!(stream.source_location, SourceLocation::new(0, 2, 0));
    }

    #[test]
    fn test_skip_whitespace() {
        let text: String =
            String::from("# This is a comment\n# This is a comment too!\n\t   This must be read!");
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        assert_eq!(stream.skip_whitespace().unwrap(), false);
        assert_eq!(stream.source_location, SourceLocation::new(0, 3, 8));
        assert_eq!(stream.saved_char.unwrap(), 'T');
        assert_eq!(stream.saved_location.unwrap(), SourceLocation::new(0, 3, 8));

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
    fn test_read_identifier() {
        let mut stream = setup1();
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_identifier(ch).unwrap();
        assert_eq!(s.as_str(), "float");
        assert_eq!(stream.skip_whitespace().unwrap(), false);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_identifier(ch).unwrap();
        assert_eq!(s.as_str(), "clock");
    }

    #[test]
    #[should_panic(expected = "Unexpected input character in string!")]
    fn test_read_identifier_err() {
        let mut stream = setup1();
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_identifier(ch).unwrap();
        let _ = stream.skip_whitespace().unwrap();
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_identifier(ch).unwrap();
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_identifier(ch).unwrap();
    }

    #[test]
    fn test_read_identifier_with_number() {
        let text: String = "a_b_8".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_identifier(ch).unwrap();
        assert_eq!(s.as_str(), "a_b_8");
    }

    #[test]
    fn test_read_identifier_with_low_dash_first() {
        let text: String = "_a".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_identifier(ch).unwrap();
        assert_eq!(s.as_str(), "_a");
    }

    #[test]
    fn test_read_string_literal_double_quote() {
        let text = "\"file.txt\"".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_string_literal(ch).unwrap();
        assert_eq!(s.as_str(), "file.txt");
    }

    #[test]
    fn test_read_string_literal_single_quote() {
        let text = "'file.txt'".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let s = stream.read_string_literal(ch).unwrap();
        assert_eq!(s.as_str(), "file.txt");
    }

    #[test]
    #[should_panic(expected = "Unexpected EOF!")]
    fn test_read_string_literal_eof() {
        let text = "'file.txt".to_string();
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let ch = stream.read_char().unwrap().unwrap();
        let _ = stream.read_string_literal(ch).unwrap();
    }

    #[test]
    #[should_panic(expected = "Wrong input character in read_string!")]
    fn test_read_string_literal_err() {
        let mut stream = setup2();
        let _ = stream.read_string_literal('k').unwrap();
    }

    #[test]
    fn test_read_number() {
        let text: String = "832\na".to_string();
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
        let text: String = "-8322.3 ".to_string();
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

    #[test]
    fn test_read_token() {
        let mut stream = setup1();
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(Float),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 1, 1),
            "token.loc = {:?}",
            token.loc
        );

        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Identifier("clock".to_string()),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 1, 7),
            "token.loc = {:?}",
            token.loc
        );

        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Symbol('('),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 1, 12),
            "token.loc = {:?}",
            token.loc
        );

        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::LiteralNumber(150.0),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 1, 13),
            "token.loc = {:?}",
            token.loc
        );

        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Symbol(')'),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 1, 16),
            "token.loc = {:?}",
            token.loc
        );

        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(Material),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 7, 1),
            "token.loc = {:?}",
            token.loc
        );

        for _ in 0..4 {
            let _ = stream.read_token();
        }

        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Symbol('<'),
            "token.kind = {:?}",
            token.kind
        );
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 8, 9),
            "token.loc = {:?}",
            token.loc
        );
    }

    #[test]
    fn test_read_token_string_literal() {
        let text = String::from("\"hello_world\"\n\"");
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::LiteralString("hello_world".to_string())
        );
        assert_eq!(token.loc, SourceLocation::new(0, 1, 1));
    }

    #[test]
    fn test_read_token_identifier_with_low_dash() {
        let text = String::from("_identifier()");
        let cursor = Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(token.kind, TokenKind::Identifier("_identifier".to_string()));
        assert_eq!(token.loc, SourceLocation::new(0, 1, 1));
    }

    #[test]
    fn test_read_token_eof() {
        let s = String::from("\n");
        let cursor = Cursor::new(s);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(token.kind, StopToken, "token.kind = {:?}", token.kind);
        assert_eq!(
            token.loc,
            SourceLocation::new(0, 2, 0),
            "token.loc = {:?}",
            token.loc
        );
    }

    #[test]
    fn test_gradient_reader() {
        let text = r#"
        gradient("#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            Keyword(Gradient),
            "token.kind = {:?}",
            token.kind
        );
    }

    #[test]
    fn test_box_reader() {
        let text = r#"
        box( #things...
        "#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(token.kind, Keyword(Box), "token.kind = {:?}", token.kind);
    }

    #[test]
    fn test_point_reader() {
        let text = r#"point([0.0, 0.0, 1.0])"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(token.kind, Keyword(Point), "token.kind = {:?}", token.kind);
    }

    #[test]
    fn test_mesh_reader() {
        let text = r#"simple_mesh("#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(SimpleMesh),
            "token.kind = {:?}",
            token.kind
        );
    }

    #[test]
    fn test_point_light_source_reader() {
        let text = r#"point_light("#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(PtLightSource),
            "token.kind = {:?}",
            token.kind
        );
    }

    #[test]
    fn test_spherical_light_source_reader() {
        let text = r#"spherical_light("#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(SphLightSource),
            "token.kind = {:?}",
            token.kind
        );
    }

    #[test]
    fn test_boolean_reader() {
        let text = r#"true,True,false,False,"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(True),
            "token.kind = {:?}",
            token.kind
        );
        stream.read_token().unwrap();
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(True),
            "token.kind = {:?}",
            token.kind
        );
        stream.read_token().unwrap();
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(False),
            "token.kind = {:?}",
            token.kind
        );
        stream.read_token().unwrap();
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(False),
            "token.kind = {:?}",
            token.kind
        );
    }

    #[test]
    fn test_color_keyword_reader() {
        use crate::lexer::Keyword::{Black, White};

        let cases = [
            ("black", TokenKind::Keyword(Black)),
            ("Black", TokenKind::Keyword(Black)),
            ("BLACK", TokenKind::Keyword(Black)),
            ("white", TokenKind::Keyword(White)),
            ("White", TokenKind::Keyword(White)),
            ("WHITE", TokenKind::Keyword(White)),
        ];

        for (input, expected) in cases {
            let cursor = Cursor::new(input);
            let mut stream = InputStream::new(cursor, 0, 4);
            let token = stream.read_token().unwrap();
            assert_eq!(
                token.kind, expected,
                "input '{}': token.kind = {:?}",
                input, token.kind
            );
        }
    }

    #[test]
    fn test_cylinder_keyword_reader() {
        let text = r#"cylinder("#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let token = stream.read_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Keyword(Cylinder),
            "token.kind = {:?}",
            token.kind
        );
    }
}
