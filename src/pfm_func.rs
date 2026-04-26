//! PFM (Portable Float Map) image reading utilities.
//!
//! This module provides functions to parse and load `.pfm` files into the
//! [`HDR`] structure used by the raytracer.
//!
//! ## Supported features
//! - Grayscale (`Pf`) and RGB (`PF`) formats
//! - Little-endian and big-endian float encoding
//! - Row-major pixel layout
//!
//! ## PFM format overview
//! A `.pfm` file is structured as:
//!
//! ```text
//! PF or Pf        # magic number (RGB or grayscale)
//! <width> <height>
//! <scale>         # sign indicates endianness
//! <binary data>   # 32-bit floats (RGB RGB...)
//! ```
//!
//! - Positive scale → big endian
//! - Negative scale → little endian
//!
//! ## Notes
//! - This implementation expects well-formed files.
//! - Extra trailing bytes are treated as an error.
//! - Pixel data is stored in row-major order.
use crate::color::Color;
use crate::hdr_image::HDR;
use anyhow::anyhow;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::string::ToString;

/// Byte order used in the PFM file.
#[derive(Debug, PartialEq)]
pub enum Endianness {
    /// Least significant byte first
    LittleEndian,
    /// Most significant byte first
    BigEndian,
}
pub enum EndiannessError {
    /// Scale value is not a number
    InvalidValue,
}

// reading and writing pfm files

/// Validates the PFM magic number (`PF` or `Pf`).
///
/// # Errors
/// Returns an error if the provided line is not a valid PFM magic header.
///
/// # Notes
/// The input is expected to be the first line of a PFM file, already read as text.
pub fn _read_magic(line: &str) -> anyhow::Result<()> {
    let trimmed = line.trim();
    if trimmed != "PF" && trimmed != "Pf" {
        return Err(anyhow!("magic is not PF nor Pf! file is not PFM"));
    }
    Ok(())
}

/// Parses the image dimensions from a PFM header line.
///
/// The input is expected to contain two whitespace-separated integers:
/// `<width> <height>`.
///
/// # Errors
/// Returns an error if:
/// - the width or height is missing
/// - parsing fails
/// - more than two values are provided
pub fn _parse_img_size(line: &str) -> anyhow::Result<(usize, usize)> {
    // Note: no control is made to verify the input is a single line
    // i.e. : "3\n2" is valid.
    let mut parts = line.split_whitespace();

    let width = parts
        .next()
        .ok_or_else(|| anyhow!("missing width"))?
        .parse::<usize>()?;

    let height = parts
        .next()
        .ok_or_else(|| anyhow!("missing height"))?
        .parse::<usize>()?;

    if parts.next().is_some() {
        return Err(anyhow!("too many values for image size"));
    }

    Ok((width, height))
}

/// Parses the PFM scale factor to determine endianness.
///
/// # Returns
/// - `BigEndian` if the value is positive
/// - `LittleEndian` if the value is negative
///
/// # Errors
/// Returns an error if:
/// - the value is zero
/// - parsing as `f32` fails
pub fn _parse_endianness(line: &str) -> anyhow::Result<Endianness> {
    let scale: f32 = line.trim().parse::<f32>()?;

    if scale > 0.0 {
        Ok(Endianness::BigEndian)
    } else if scale < 0.0 {
        Ok(Endianness::LittleEndian)
    } else {
        Err(anyhow::anyhow!("PFM scale factor cannot be zero"))
    }
}

/// Reads pixel data and constructs an [`HDR`] image **top to bottom**.
///
/// # Arguments
/// * `line`- Pixels string from file
/// * `width`, `height` - Image dimensions
/// * `endianness` - Byte order of the pixel data
///
/// # Behavior
/// - Reads 3 `f32` values per pixel (R, G, B)
/// - Assumes row-major order
/// - Converts bytes based on endianness
///
/// # Errors
/// Returns an error if:
/// - The buffer does not contain enough data
/// - Extra bytes remain after reading all pixels
///
/// # Notes
/// The function assumes that the cursor is positioned at the start
/// of the binary pixel data.
fn _read_hdr<R: Read>(
    reader: &mut R,
    width: usize,
    height: usize,
    endianness: Endianness,
) -> anyhow::Result<HDR> {
    // Create an empty image
    let mut hdr_img: HDR = HDR::new(width, height);

    let mut buffer = [0; 4];

    //bytes to f32 is a closure that avoids code repetition, it takes an array of four bytes and,
    //matching the endianness, it turns it into f32
    let bytes_to_f32 = |buf: [u8; 4]| match endianness {
        Endianness::LittleEndian => f32::from_le_bytes(buf),
        Endianness::BigEndian => f32::from_be_bytes(buf),
    };

    // Color the empty image
    for i in (0..height).rev() {
        for j in 0..width {
            reader.read_exact(&mut buffer).expect("unexpected eof");
            let r = bytes_to_f32(buffer);
            reader.read_exact(&mut buffer).expect("unexpected eof");
            let g = bytes_to_f32(buffer);
            reader.read_exact(&mut buffer).expect("unexpected eof");
            let b = bytes_to_f32(buffer);
            hdr_img.pixels[width * i + j] = Color::new(r, g, b);
        }
    }

    let mut check_extra_bytes = Vec::new();
    reader.read_to_end(&mut check_extra_bytes)?;
    if !check_extra_bytes.is_empty() {
        return Err(anyhow!(
            "extra bytes at end of file! (incorrect image dimensions or bytes stored)"
        ));
    }

    Ok(hdr_img)
}

/// A convenience function to read a `.pfm` (Portable FloatMap) directly from a file.
///
/// This function opens the file at the given path, wraps it in a [`BufReader`]
/// for performance, and delegates the parsing to the generic [`read_pfm`] engine.
///
/// # Arguments
///
/// * `filename` - The path to the `.pfm` file.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened (e.g., file not found, permission denied).
/// - The file content is not a valid PFM format (delegated to [`read_pfm`]).
///
/// # Examples
///
/// ```rust,no_run
/// use rstrace::pfm_func::read_pfm_file;
/// use rstrace::hdr_image::HDR;
///
/// # fn main() -> anyhow::Result<()> {
/// let image: HDR = read_pfm_file("image.pfm")?;
/// assert!(image.width > 0);
/// assert!(image.height > 0);
/// # Ok(())
/// # }
/// ```
/// # Notes
/// The function expects a well-formed PFM file. Validation is performed
/// during parsing, and any inconsistency results in an error.
pub fn read_pfm_file(filename: &str) -> anyhow::Result<HDR> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    read_pfm(reader)
}

/// Reads a `.pfm` (Portable FloatMap) image from a generic buffered stream.
///
/// This function parses the PFM header (magic number, dimensions, and scale)
/// and reads the binary pixel data into an [`HDR`] structure **TOP TO BOTTOM**.
///
/// Both RGB (`PF`) and grayscale (`Pf`) formats are supported. Pixel data
/// is interpreted according to the endianness specified by the scale factor.
///
/// # Arguments
///
/// * `reader` - The input stream (e.g., a file, a memory buffer, or network socket).
///   It accepts any type that implements the [`BufRead`] trait.
///
/// # Errors
///
/// Returns an error if:
/// - An I/O error occurs while reading the stream.
/// - The magic number is invalid or missing.
/// - The image dimensions cannot be parsed.
/// - The scale factor is invalid or zero.
/// - The binary data is incomplete or malformed.
/// - Extra bytes are found after the pixel data.
///
/// # Examples
///
/// ```rust
/// use std::io::Cursor;
/// use rstrace::pfm_func::read_pfm;
/// use rstrace::hdr_image::HDR;
///
/// # fn main() -> anyhow::Result<()> {
/// // A tiny, fake PFM file in memory (RAM)
/// let pfm_data = b"PF\n1 1\n-1.0\n\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
/// let mut stream = Cursor::new(pfm_data);
///
/// let image = read_pfm(&mut stream)?;
/// assert_eq!(image.width, 1);
/// assert_eq!(image.height, 1);
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// The function expects a well-formed PFM stream. Validation is performed
/// during parsing, and any inconsistency results in an error.
pub fn read_pfm<R: BufRead>(mut reader: R) -> anyhow::Result<HDR> {
    let mut line: String = String::new();
    reader.read_line(&mut line)?;
    _read_magic(&line)?;

    //// checks the dimension of the image
    line.clear();

    reader.read_line(&mut line)?;
    let (width, height) = _parse_img_size(&line)?;

    println!("Pfm image size: {}x{}", width, height);
    line.clear();
    reader.read_line(&mut line)?;
    let endianness = _parse_endianness(&line);

    println!("endianness: {}", line.trim());

    let hdr_img = _read_hdr(&mut reader, width, height, endianness?)?;
    Ok(hdr_img)
}

/// converting from pfm to jpeg
///
pub struct Parameter {
    pub input_pfm_file_name: String,
    pub factor_a: f32,
    pub gamma: f32,
    pub output_file_name: String,
}

impl Parameter {
    /// Constructs a [`Parameter`] instance from command-line arguments.
    ///
    /// The expected argument format is:
    /// ```text
    /// <program> <input_file_name> <factor_a> <gamma> <output_file_name>
    /// ```
    ///
    /// # Parameters
    /// - `args`: Vector of command-line arguments (typically from `std::env::args()`)
    ///
    /// # Behavior
    /// - Parses `factor_a` and `gamma` as `f32` values
    /// - If `factor_a <= 0.0`, it is replaced with a default value of `0.18`
    /// - If `gamma <= 0.0`, it is replaced with a default value of `2.2`
    ///
    /// # Errors
    /// Returns an error if:
    /// - The number of arguments is not exactly 5
    ///
    /// # Panics
    /// This function will panic if:
    /// - `factor_a` or `gamma` cannot be parsed as `f32`
    ///
    /// # Notes
    /// - Argument validation is minimal: only the argument count is checked
    /// - Invalid numeric values are handled via `expect`, which will terminate the program
    ///
    /// # Example
    /// ```rust, no_run
    /// use rstrace::pfm_func::Parameter;
    /// let args = vec![
    ///     "program".into(),
    ///     "input.pfm".into(),
    ///     "0.18".into(),
    ///     "2.2".into(),
    ///     "output.png".into(),
    /// ];
    ///
    /// let params = Parameter::new(args).unwrap();
    /// ```
    pub fn new(args: Vec<String>) -> anyhow::Result<Parameter> {
        if args.len() != 5 {
            return Err(anyhow!(
                "wrong number of parameters: expected\n\
            <input_file_name> <factor_a> <gamma> <output_file_name>"
            ));
        }

        let input_temp: &String = &args[1];
        let input_pfm_file_name = input_temp.to_string();
        let mut factor_a: f32 = args[2].parse::<f32>().expect("invalid factor_a value");
        let mut gamma: f32 = args[3].parse::<f32>().expect("invalid gamma value");
        let output_temp: &String = &args[4];
        let output_file_name: String = output_temp.to_string();
        if factor_a <= 0.0 {
            println!("factor 'a' was automatically set to 0.18");
            factor_a = 0.18;
        }

        if gamma <= 0.0 {
            println!("gamma was automatically set to 2.2");
            gamma = 2.2;
        }

        Ok(Parameter {
            input_pfm_file_name,
            factor_a,
            gamma,
            output_file_name,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::color::Color;
    use crate::pfm_func::{_parse_endianness, _parse_img_size, _read_magic, Endianness};

    const BE_ARRAY: &[u8] = &[
        0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42, 0xc8, 0x00, 0x00,
        0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43, 0xc8, 0x00, 0x00, 0x43, 0xfa, 0x00,
        0x00, 0x44, 0x16, 0x00, 0x00, 0x44, 0x2f, 0x00, 0x00, 0x44, 0x48, 0x00, 0x00, 0x44, 0x61,
        0x00, 0x00, 0x41, 0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41, 0xf0, 0x00, 0x00, 0x42,
        0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00, 0x00, 0x42, 0x8c, 0x00, 0x00,
        0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
    ];

    const LE_ARRAY: &[u8] = &[
        0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00, 0xc8,
        0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43, 0x00, 0x00,
        0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00, 0x48, 0x44, 0x00,
        0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41,
        0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00, 0x70, 0x42, 0x00, 0x00, 0x8c,
        0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
    ];

    use super::*;
    use std::io::{BufRead, Cursor};

    #[test]
    fn test_read_magic() {
        let mut pf: String = String::from("pf");
        assert!(_read_magic(&mut pf).is_err());
        assert!(_read_magic("PF\nERROR!!").is_err());

        pf = String::from("\nPf\n");
        assert!(_read_magic(&mut pf).is_ok());
        assert!(_read_magic("PF\n\n\n\n\n").is_ok());
    }

    //test _parse_img_size
    #[test]
    fn test_parse_img_size() -> anyhow::Result<()> {
        let mut img_dim = String::from("3 2");
        assert_eq!(_parse_img_size(&mut img_dim)?, (3, 2));
        assert!(_parse_img_size("   3 ").is_err());
        assert!(_parse_img_size("3   2  ").is_ok());
        assert!(_parse_img_size("3 2 3").is_err());
        Ok(())
    }

    // test _parse_endianness
    #[test]
    fn test_parse_endianness() -> anyhow::Result<()> {
        let mut minus_one = String::from("-1.0");
        let mut plus_one = String::from("+1.0");
        let mut zero = String::from("0.0");
        let mut minus_zero = String::from("-0.0");
        let mut test_char = String::from("a");

        assert_eq!(_parse_endianness(&mut minus_one)?, Endianness::LittleEndian);
        assert_eq!(_parse_endianness(&mut plus_one)?, Endianness::BigEndian);
        assert!(_parse_endianness(&mut zero).is_err());
        assert!(_parse_endianness(&mut minus_zero).is_err());
        assert!(_parse_endianness(&mut test_char).is_err());
        Ok(())
    }

    // test read_hdr
    #[test]
    #[should_panic]
    // tests that _read_hdr correctly panics when buffer is too short
    fn test_1_read_hdr() {
        let file = File::open("reference_be.pfm");
        let mut reader = BufReader::new(file.unwrap());
        let mut line: String = String::new();
        reader.read_line(&mut line).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        line.clear();

        let _hdr = _read_hdr(&mut reader, 2, 4, Endianness::BigEndian);
    }

    #[test]
    #[should_panic]
    // tests that _read_hdr correctly panics when buffer is too long
    fn test_2_read_hdr() {
        let file = File::open("reference_be.pfm");
        let mut reader = BufReader::new(file.unwrap());
        let mut line: String = String::new();
        reader.read_line(&mut line).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        line.clear();

        let _hdr = _read_hdr(&mut reader, 2, 2, Endianness::BigEndian).unwrap();
    }

    // test for new created for Parameter
    #[test]
    #[should_panic]
    //should panic if factor_a is not a number
    fn test_1_new_parameter() {
        let strings: Vec<String> = ["exe", "filename_in", "a", "2.2", "filename_out"]
            .map(String::from)
            .to_vec();
        let _par = Parameter::new(strings);
    }

    #[test]
    #[should_panic]
    //should panic if gamma is not a number
    fn test_2_new_parameter() {
        let strings: Vec<String> = ["exe", "filename_in", "0.18", "a", "filename_out"]
            .map(String::from)
            .to_vec();
        let _par = Parameter::new(strings);
    }

    #[test]
    //sets factor_a to 0.18 when a < 0
    fn test_3_new_parameter() {
        let strings: Vec<String> = ["exe", "filename_in", "-1", "2.2", "filename_out"]
            .map(String::from)
            .to_vec();
        let par = Parameter::new(strings).unwrap();
        assert_eq!(0.18, par.factor_a);
    }

    #[test]
    //sets gamma to 2.2 when gamma < 0
    fn test_4_new_parameter() {
        let strings: Vec<String> = ["exe", "filename_in", "0.18", "-1", "filename_out"]
            .map(String::from)
            .to_vec();
        let par = Parameter::new(strings).unwrap();
        assert_eq!(2.2, par.gamma);
    }

    #[test]
    #[should_panic]
    //should panic if incorrect number of input parameters
    fn test_5_new_parameter() {
        let strings: Vec<String> = [
            "added string",
            "exe",
            "filename_in",
            "0.18",
            "a",
            "filename_out",
        ]
        .map(String::from)
        .to_vec();
        let _par = Parameter::new(strings).unwrap();
    }

    #[test]
    fn test_read_pfm() -> anyhow::Result<()> {
        for _reference_bytes in [BE_ARRAY, LE_ARRAY] {
            let mut stream = Cursor::new(_reference_bytes);
            let img = read_pfm(&mut stream)?;
            assert_eq!(img.width, 3);
            assert_eq!(img.height, 2);
            assert_eq!(img.get_pixel(0, 0)?, Color::new(1.0e1, 2.0e1, 3.0e1));
            assert_eq!(img.get_pixel(1, 0)?, Color::new(4.0e1, 5.0e1, 6.0e1));
            assert_eq!(img.get_pixel(2, 0)?, Color::new(7.0e1, 8.0e1, 9.0e1));
            assert_eq!(img.get_pixel(0, 1)?, Color::new(1.0e2, 2.0e2, 3.0e2));
            assert_eq!(img.get_pixel(1, 1)?, Color::new(4.0e2, 5.0e2, 6.0e2));
            assert_eq!(img.get_pixel(2, 1)?, Color::new(7.0e2, 8.0e2, 9.0e2));
        }
        Ok(())
    }
}
