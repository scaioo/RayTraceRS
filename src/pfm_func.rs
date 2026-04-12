//! PFM (Portable Float Map) image reading utilities.
//!
//! This module provides functions to parse and load `.pfm` files into the
//! [`HDR`](crate::hdr_image::HDR) structure used by the raytracer.
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
use std::arch::x86_64::_mm256_insert_epi16;
use std::char::from_u32;
use std::env;
use std::fs::File;

use std::io;
use std::io::Seek;
use std::io::{BufRead, BufReader, Read, stdin};
use std::ptr::addr_of_mut;
use std::string::ToString;

use std::io::Cursor;

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

/// Reads pixel data and constructs an [`HDR`] image.
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
fn _read_hdr(
    reader: &mut BufReader<File>,
    width: usize,
    height: usize,
    endianness: Endianness,
) -> anyhow::Result<HDR> {
    // Create an empty image
    let mut hdr_img: HDR = HDR::new(width, height);

    let mut buffer = [0; 4];

    //bytes to f32 is a closure that avoids code repetition, it takes an array of four bytes and,
    //matching the endianness, it turns it into f32
    //matching the endianness, it turns it into a f32
    let bytes_to_f32 = |buf: [u8; 4]| match endianness {
        Endianness::LittleEndian => f32::from_le_bytes(buf),
        Endianness::BigEndian => f32::from_be_bytes(buf),
    };

    // Color the empty image
    for i in (0..height).rev() {
        for j in 0..width {
            reader.read_exact(&mut buffer)?;
            let r = bytes_to_f32(buffer);
            reader.read_exact(&mut buffer)?;
            let g = bytes_to_f32(buffer);
            reader.read_exact(&mut buffer)?;
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

// read_pfm_image uses the functions defined above to read all the necessary information from a pfm file
// and returns an HDR type containing the datas in the pfm.
// row-major order is used to read pixels

// MISSING DOCUMENTATION!!!!
pub fn read_pfm_file(filename: &str) -> anyhow::Result<HDR, anyhow::Error> {
    let file = File::open(filename);
    let mut reader = BufReader::new(file?);
    let mut line: String = String::new();

    reader.read_line(&mut line)?;
    _read_magic(&mut line)?;

    //// checks the dimension of the image
    line.clear();

    reader.read_line(&mut line)?;
    let (width, height) = _parse_img_size(&mut line)?;

    println!("Pfm image size: {}x{}", width, height);
    line.clear();
    reader.read_line(&mut line)?;
    let endianness = _parse_endianness(&mut line);

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
    pub fn new(args: Vec<String>) -> anyhow::Result<Parameter> {
        if args.len() != 5 {
            return Err(anyhow!("wrong number of parameters"));
        }

        let input_temp: &String = &args[1];
        let input_pfm_file_name = input_temp.to_string();
        let mut factor_a: f32 = args[2].parse::<f32>().expect("invalid factor_ value");
        let mut gamma: f32 = args[3].parse::<f32>().expect("invalid gamma value");
        let output_temp: &String = &args[4];
        let output_file_name: String = output_temp.to_string();
        if factor_a <= 0.0 {
            println!("factor 'a' was automatically set to 0.18");
            factor_a = 0.18;
        }

        if gamma <= 0.0 {
            println!("gamma was automatically set to 1.0");
            gamma = 1.0;
        }

        Ok(Parameter {
            input_pfm_file_name,
            factor_a,
            gamma,
            output_file_name,
        })
    }
}

// parse_command_line takes input parameters,
// checks their number and format
// and returns a Parameter type containing all the information

// images are indexed with (0,0) at the top left corner.

// test parse endianness: verify endianness result is correct
// e che si arrabbi quando il numero è 0
#[cfg(test)]
mod test {
    use crate::color::Color;
    use crate::pfm_func::{_parse_endianness, _parse_img_size, _read_hdr, _read_magic, Endianness};

    const BE_ARRAY: &[u8] = &[
        0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42, 0xc8, 0x00, 0x00,
        0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43, 0xc8, 0x00, 0x00, 0x43, 0xfa, 0x00,
        0x00, 0x44, 0x16, 0x00, 0x00, 0x44, 0x2f, 0x00, 0x00, 0x44, 0x48, 0x00, 0x00, 0x44, 0x61,
        0x00, 0x00, 0x41, 0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41, 0xf0, 0x00, 0x00, 0x42,
        0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00, 0x00, 0x42, 0x8c, 0x00, 0x00,
        0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
    ];

    use anyhow::anyhow;

    const LE_ARRAY: &[u8] = &[
        0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00, 0xc8,
        0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43, 0x00, 0x00,
        0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00, 0x48, 0x44, 0x00,
        0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41,
        0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00, 0x70, 0x42, 0x00, 0x00, 0x8c,
        0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
    ];

    use super::*;
    use crate::functions::are_close;
    use std::io;
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

    /*#[test]
    #[should_panic(expected = "no more floats!")]
    fn test_read_4bytes_le() {
        let mut rdr = io::Cursor::new(LE_ARRAY);
        for _ in 0..3 {
            let mut line = String::new();
            let _ =rdr.read_line(& mut line).unwrap();
        }

        for i in 0..9 {
            let val = _read_4bytes(Endianness::LittleEndian, &mut rdr).unwrap();
            let expected = ((i + 1) * 100) as f32;
            assert!(are_close(val, expected));
        }
        for i in 0..9 {
            let val = _read_4bytes(Endianness::LittleEndian, &mut rdr).unwrap();
            let expected = ((i + 1) * 10) as f32;
            assert!(are_close(val, expected));
        }
        _read_4bytes(Endianness::LittleEndian,&mut rdr).expect("no more floats!");
    }*/

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



    /*    #[should_panic(expected = "no more floats!")]
        fn test_read_4bytes_be() {
            let mut rdr = io::Cursor::new(BE_ARRAY);
            for _ in 0..3 {
                let mut line = String::new();
                let _ =rdr.read_line(& mut line).unwrap();
            }
            for i in 0..9 {
                let val = _read_4bytes(Endianness::BigEndian, &mut rdr).unwrap();
                let expected = ((i + 1) * 100) as f32;
                assert!(are_close(val, expected));
            }
            for i in 0..9 {
                let val = _read_4bytes(Endianness::BigEndian, &mut rdr).unwrap();
                let expected = ((i + 1) * 10) as f32;
                assert!(are_close(val, expected));
            }
            _read_4bytes(Endianness::BigEndian,&mut rdr).expect("no more floats!");
        }
    */
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

    // test _read_hdr

    // DA FINIRE
    //#[test]
    /*fn test_read_hdr() -> anyhow::Result<()>{
            let mut le = [0x00, 0x00, 0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43];
            let mut vec_le = le.to_vec();
            let mut str_le = String::from_utf8(vec_le)?;
            let img = _read_hdr(&mut  str_le, 3, 2, Endianness::LittleEndian)?;

            //assert_eq!(img.get_pixel(0, 1), Color::new(1.0, 2.0, 3.0));

            let mut be = [0x42,
                0xc8, 0x00, 0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43,
                0xc8, 0x00, 0x00, 0x43, 0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44,
                0x2f, 0x00, 0x00, 0x44, 0x48, 0x00, 0x00, 0x44, 0x61, 0x00, 0x00, 0x41,
                0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41, 0xf0, 0x00, 0x00, 0x42,
                0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00, 0x00, 0x42,
                0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00
            ];

            Ok(())
        }

        // test read_pfm_file
        // TODO
    */
}
