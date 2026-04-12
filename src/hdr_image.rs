//! HDR Image Module
//!
//! This module defines the [`HDR`] struct, which represents a High Dynamic Range
//! image using floating-point RGB pixels.
//!
//! It provides:
//! - Image creation and pixel manipulation
//! - Tone mapping utilities
//! - Export to `.pfm` (Portable Float Map) format
//!
//! ## Features
//!
//! - Linear RGB floating-point storage
//! - Safe pixel indexing with bounds checking
//! - Basic tone mapping (Reinhard operator)
//! - Log-average luminance computation
//!
//! ## Example
//!
//! ```rust
//! use crate::color::Color;
//! use crate::hdr::HDR;
//!
//! let mut img = HDR::new(512, 512);
//!
//! img.set_pixel(10, 10, Color { r: 1.0, g: 0.5, b: 0.2 }).unwrap();
//! let px = img.get_pixel(10, 10).unwrap();
//!
//! assert_eq!(px.r, 1.0);
//! ```
//!
use crate::color::Color;
use crate::functions::endianness_number;
use anyhow::{Result, anyhow};
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use endianness::ByteOrder;
use std::io::Write;

use crate::pfm_func::{Parameter, read_pfm_file};
use image::{Rgb, RgbImage};

/// Represents an HDR (High Dynamic Range) image.
///
/// Pixels are stored as a flat vector of [`Color`] in row-major order.
///
/// # Fields
///
/// - `width`: Image width in pixels
/// - `height`: Image height in pixels
/// - `pixels`: Flat vector of RGB colors
///
/// # Storage Layout
///
/// Pixels are stored row-by-row:
///
/// ```text
/// index = x + y * width
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct HDR {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl HDR {
    /// Creates a new HDR image filled with black pixels.
    ///
    /// # Arguments
    ///
    /// * `width` - Image width
    /// * `height` - Image height
    ///
    /// # Returns
    ///
    /// A new [`HDR`] image where all pixels are initialized to `Color::default()`.
    pub fn new(width: usize, height: usize) -> HDR {
        let pixels = vec![Color::default(); width * height];
        HDR {
            width,
            height,
            pixels,
        }
    }

    /// Writes the image to a `.pfm` (Portable Float Map) file.
    ///
    /// # Arguments
    /// * `filename` - Output file path
    /// * `endianness` - Byte order used for writing floats
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be created
    /// - Writing to the file fails
    ///
    /// # Notes
    /// - The image is written in **binary format**
    /// - Pixels are stored **bottom-to-top** as required by the PFM specification
    /// - The scale factor encodes endianness:
    ///   - Negative = little endian
    ///   - Positive = big endian
    pub fn write_pfm<W: Write>(&self, mut writer: W, endianness: &ByteOrder) -> Result<()> {
        write!(
            writer,
            "PF\n{} {}\n{:.1}\n",
            self.width,
            self.height,
            endianness_number(endianness)
        )?;

        match endianness {
            ByteOrder::LittleEndian => {
                for y in (0..self.height).rev() {
                    for x in 0..self.width {
                        let color = self.get_pixel(x, y)?;
                        writer.write_f32::<LittleEndian>(color.r)?;
                        writer.write_f32::<LittleEndian>(color.g)?;
                        writer.write_f32::<LittleEndian>(color.b)?;
                    }
                }
            }
            ByteOrder::BigEndian => {
                for y in (0..self.height).rev() {
                    for x in 0..self.width {
                        let color = self.get_pixel(x, y)?;
                        writer.write_f32::<BigEndian>(color.r)?;
                        writer.write_f32::<BigEndian>(color.g)?;
                        writer.write_f32::<BigEndian>(color.b)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Sets the color of a pixel at `(x, y)`.
    ///
    /// # Errors
    /// Returns an error if `(x, y)` is out of bounds.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<()> {
        self.check_position(x, y)?;
        self.pixels[y * self.width + x] = color;
        Ok(())
    }

    /// Returns the color of the pixel at `(x, y)`.
    ///
    /// # Errors
    /// Returns an error if `(x, y)` is out of bounds.
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        //Ok(self.pixels[y * self.width + x])  Is it better this previous version??
        Ok(self.pixels[self.vector_index(x, y)?])
    }

    /// Converts `(x, y)` coordinates into a linear index.
    ///
    /// # Errors
    /// Returns an error if `(x, y)` is out of bounds.
    pub fn vector_index(&self, x: usize, y: usize) -> Result<usize> {
        self.check_position(x, y)?;
        Ok(x + y * self.width)
    }

    /// Checks whether `(x, y)` is inside image bounds.
    ///
    /// # Errors
    /// Returns an error if the coordinates are out of bounds.
    fn check_position(&self, x: usize, y: usize) -> Result<()> {
        if x < self.width && y < self.height {
            Ok(())
        } else {
            Err(anyhow!("OUT OF BOUND PIXEL ({},{})!", x, y))
        }
    }
}

// ====================
//     Tone Mapping
// ====================

impl HDR {
    /// Computes the logarithmic average luminance of the image.
    ///
    /// # Returns
    /// The log-average luminance as `f32`.
    ///
    /// # Errors
    /// Returns an error if the image contains no pixels.
    ///
    /// # Notes
    /// Uses:
    /// ```text
    /// L_avg = 10^( (1/N) * Σ log10(L_i + ε) )
    /// ```
    ///
    /// where `ε` avoids log(0).
    pub fn average_luminosity(&self) -> Result<f32> {
        let count = self.pixels.len() as f32;
        if count == 0.0 {
            return Err(anyhow!(
                "average_luminosity():
            no pixel to compute average_luminosity!!!!!"
            ));
        }

        let log_sum: f32 = self
            .pixels
            .iter()
            .map(|col| (col.sem_luminosity().unwrap() + f32::EPSILON).log10())
            .sum();

        Ok(10.0_f32.powf(log_sum / count))
    }

    /// Normalizes the image luminance.
    ///
    /// # Arguments
    /// * `wrapped_a` - Optional exposure scaling factor (default: `0.18`)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The image is empty
    /// - `a <= 0`
    /// - Average luminance is zero
    ///
    /// # Description
    /// Each pixel is scaled by:
    ///
    /// ```text
    /// color = (color * a) / L_avg
    /// ```
    pub fn normalization(&mut self, wrapped_a: Option<f32>) -> Result<()> {
        if self.pixels.len() == 0 {
            return Err(anyhow!("normalization(): no pixels to normalize!!!!"));
        }

        let a = wrapped_a.unwrap_or(0.18);
        if a <= 0.0 {
            return Err(anyhow!(
                "normalization():\
             Cannot use a non-positive normalization factor: {a}!!!!"
            ));
        }

        let avr = self.average_luminosity()?;
        if avr == 0.0 {
            return Err(anyhow!(
                "normalization():
            Average luminosity is zero, cannot normalize."
            ));
        }

        for color in self.pixels.iter_mut() {
            *color = (*color * a) / avr;
        }
        Ok(())
    }

    /// Applies Tone mapping to all pixels.
    ///
    /// # Errors
    /// Returns an error if the image is empty.
    ///
    /// # Description
    /// Uses a per-channel Reinhard operator:
    ///
    /// ```text
    /// c = c / (1 + c)
    /// ```
    pub fn sem_clamp_image(&mut self) -> Result<()> {
        if self.pixels.len() == 0 {
            return Err(anyhow!(
                "sem_clamp_image(): no pixel to tone_map_reinhard!!!!!"
            ));
        }
        for color in self.pixels.iter_mut() {
            color.tone_map()?;
        }
        Ok(())
    }
}

// TODO MISSING DOCUMENTATION
// Note:
// After a little look to the code a couple of double-checks:
// - [ ] What does the Box<> do?
// - [ ] Why do you use .expect() instead of .unwrap() when treating the .pfm reading?
//       Isn't it better to keep the original Err message we designed?
// - [ ] What is the first loop for?
// - [ [ Watch out for reversed pixel writing in .pfm files!
pub fn hdr_to_ldr(img: &HDR, argv: &mut Parameter) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", &mut argv.input_pfm_file_name);
    // Creates HDR object and fill with the .pfm file
    let mut img = read_pfm_file(&mut argv.input_pfm_file_name)
        .expect("error reading input file");

    println!(
        "File {} has been opened and read",
        &mut argv.input_pfm_file_name
    );
    
    // Tone mapping of the HDR image
    img.normalization(Some(argv.factor_a))
        .expect("error during image normalization");
    img.sem_clamp_image().expect("error: sem_clamp_image");

    // Create RgbImage box and fill it with the image
    let mut new_img: RgbImage = RgbImage::new(img.width as u32, img.height as u32);
    
    // What is this loop for? Did you want to change [`img`]? 
    for y in 0..new_img.height() { // Shouldn't it .rev()?
        for x in 0..new_img.width() {
            let cur_color = new_img.get_pixel(x, y);

            let r = (cur_color[0].pow((1.0 / argv.gamma) as u32));
            let g = (cur_color[1].pow((1.0 / argv.gamma) as u32));
            let b = (cur_color[2].pow((1.0 / argv.gamma) as u32));
        }
    }
    let to_u8 = |x: f32| (x * 255.0).round() as u8;

    for y in 0..img.height { // What is LDR convention? Still .rev()? Another?
        for x in 0..img.width {
            let pixel = &img.pixels[img.width * (img.height - 1 - y) + x];

            let r = to_u8(pixel.r);
            let g = to_u8(pixel.g);
            let b = to_u8(pixel.b);

            new_img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }

    let out_file_name = &argv.output_file_name;
    let out_file_name_str = out_file_name.to_string();

    new_img.save(out_file_name_str)?;
    println!("all done");

    Ok(())
}
//                 tests

#[cfg(test)]
mod test {
    use super::*;
    // Test for
    #[test]
    fn test_new() {
        let hdr = HDR::new(10, 55);
        assert_eq!(hdr.width, 10);
        assert_eq!(hdr.height, 55);
        assert_eq!(hdr.pixels.len(), 550);
        let all_black = hdr
            .pixels
            .iter()
            .all(|p| p.r == 0.0 && p.g == 0.0 && p.b == 0.0);
        assert!(all_black, "Not all pixels were initialized to black!");
    }

    #[test]
    fn test_set_pixel() {
        let mut hdr = HDR::new(10, 2);
        hdr.set_pixel(
            5,
            1,
            Color {
                r: 1.0,
                g: 2.5,
                b: 10.0,
            },
        )
        .unwrap();
        let pixel = hdr.get_pixel(5, 1).unwrap();
        assert_eq!(pixel.r, 1.0);
        assert_eq!(pixel.g, 2.5);
        assert_eq!(pixel.b, 10.0);
    }
    #[test]
    fn test_get_pixel() {
        let mut hdr = HDR::new(10, 2);
        let color = Color {
            r: 1.0,
            g: 0.2,
            b: 30.0,
        };
        hdr.set_pixel(1, 1, color).unwrap();
        let pixel = hdr.get_pixel(1, 1).unwrap();
        assert_eq!(pixel.r, color.r);
        assert_eq!(pixel.g, color.g);
        assert_eq!(pixel.b, color.b);
    }

    #[test]
    #[should_panic]
    fn test_get_pixel_panic() {
        let hdr = HDR::new(10, 2);
        let _ = hdr.get_pixel(11, 1).unwrap();
    }

    #[test]
    fn test_vector_index() {
        let x = 9;
        let y = 1;
        let hdr = HDR::new(10, 10);
        assert_eq!(hdr.vector_index(x, y).unwrap(), y * hdr.width + x);
    }

    #[test]
    #[should_panic]
    fn test_check_position() {
        let hdr = HDR::new(10, 55);
        hdr.check_position(11, 2).unwrap();
    }

    #[test]
    fn test_write_pfm() {
        let reference_le_bytes = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00,
            0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43,
            0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00,
            0x48, 0x44, 0x00, 0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41,
            0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00,
            0x70, 0x42, 0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
        ];

        let reference_be_bytes = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42, 0xc8, 0x00,
            0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43, 0xc8, 0x00, 0x00, 0x43,
            0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44, 0x2f, 0x00, 0x00, 0x44, 0x48, 0x00,
            0x00, 0x44, 0x61, 0x00, 0x00, 0x41, 0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41,
            0xf0, 0x00, 0x00, 0x42, 0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00,
            0x00, 0x42, 0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00,
        ];
        let mut img = HDR::new(3, 2);

        img.set_pixel(0, 0, Color::new(1.0e1, 2.0e1, 3.0e1))
            .unwrap(); // Each component is
        img.set_pixel(1, 0, Color::new(4.0e1, 5.0e1, 6.0e1))
            .unwrap(); // different from any
        img.set_pixel(2, 0, Color::new(7.0e1, 8.0e1, 9.0e1))
            .unwrap(); // other: important in
        img.set_pixel(0, 1, Color::new(1.0e2, 2.0e2, 3.0e2))
            .unwrap(); // tests!
        img.set_pixel(1, 1, Color::new(4.0e2, 5.0e2, 6.0e2))
            .unwrap();
        img.set_pixel(2, 1, Color::new(7.0e2, 8.0e2, 9.0e2))
            .unwrap();
        let mut buffer: Vec<u8> = vec![];
        img.write_pfm(&mut buffer, &ByteOrder::LittleEndian)
            .unwrap();
        assert_eq!(buffer, reference_le_bytes);
        buffer = vec![];
        img.write_pfm(&mut buffer, &ByteOrder::BigEndian).unwrap();
        assert_eq!(buffer, reference_be_bytes);
    }

    #[test]
    fn test_sem_clamp_image() {
        let mut hdr = HDR::new(1, 2);

        hdr.sem_clamp_image().unwrap();
        assert_eq!(hdr.get_pixel(0, 0).unwrap().r, 0.0);

        hdr.set_pixel(
            0,
            0,
            Color {
                r: 1.0,
                g: 2.0e02,
                b: 3.0e03,
            },
        )
        .unwrap();
        hdr.sem_clamp_image().unwrap();

        assert_eq!(hdr.get_pixel(0, 0).unwrap().r, 1.0 / 2.0);
        assert_eq!(hdr.get_pixel(0, 0).unwrap().b, 3.0e3 / (1.0 + 3.0e3));
        assert_eq!(hdr.get_pixel(0, 0).unwrap().g, 2.0e2 / (1.0 + 2.0e2));
        assert_eq!(hdr.get_pixel(0, 1).unwrap().b, 0.0);
    }
}
