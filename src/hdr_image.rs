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
use anyhow::{Result, anyhow};
use endianness::{ByteOrder, EndiannessResult};
use std::fs::File;
use std::io;
use std::path::Path;
use std::io::BufRead;

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
    pub fn write_pfm(&self, filename: &str, endianness: &ByteOrder) -> Result<()> {
        // Create the new file with name 'filename'

        let path = Path::new(filename); //Create a path to make the new file
        let display = path.display(); //Create a variable to print the path
        let mut file = match File::create(filename) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        // Need to find a way to write in the line.
        // How can I write in bytes?

        // Later I will need this...
        // let ENDIAN = functions::endianness_number(endianness);

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
            return Err(anyhow!("average_luminosity():
            no pixel to compute average_luminosity!!!!!"));
        }

        let log_sum: f32 = self.pixels.iter()
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
            return Err(anyhow!("normalization(): no pixels to normalize!!!!"))
        }

        let a = wrapped_a.unwrap_or(0.18);
        if a <= 0.0 {
            return Err(anyhow!("normalization():\
             Cannot use a non-positive normalization factor: {a}!!!!"))
        }

        let avr = self.average_luminosity()?;
        if avr == 0.0 {
            return Err(anyhow!("normalization():
            Average luminosity is zero, cannot normalize."));
        }

        for color in self.pixels.iter_mut() {
            *color = (*color * a) / avr;
        }
        Ok(())
    }

    /// Applies Reinhard tone mapping to all pixels.
    ///
    /// # Errors
    ///
    /// Returns an error if the image is empty.
    ///
    /// # Description
    ///
    /// Uses a per-channel Reinhard operator:
    ///
    /// ```text
    /// c = c / (1 + c)
    /// ```
    pub fn sem_clamp_image(&mut self) -> Result<()> {
        if self.pixels.len() == 0 {
            return Err(anyhow!("sem_clamp_image(): no pixel to tone_map_reinhard!!!!!"));
        }
        for color in self.pixels.iter_mut() {
            color.tone_map_reinhard()?;
        }
        Ok(())
    }
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
        panic!("YOU NEED TO WRITE THE TEST!!!")
    }

    #[test]
    fn test_sem_clamp_image() {
        let mut hdr = HDR::new(1, 2);

        hdr.sem_clamp_image().unwrap();
        assert_eq!(hdr.get_pixel(0,0).unwrap().r, 0.0);

        hdr.set_pixel(0,0,Color{r: 1.0, g: 2.0e02, b: 3.0e03}).unwrap();
        hdr.sem_clamp_image().unwrap();

        assert_eq!(hdr.get_pixel(0,0).unwrap().r, 1.0 / 2.0);
        assert_eq!(hdr.get_pixel(0,0).unwrap().b, 3.0e3 / (1.0 + 3.0e3));
        assert_eq!(hdr.get_pixel(0,0).unwrap().g, 2.0e2 / (1.0 + 2.0e2));
        assert_eq!(hdr.get_pixel(0,1).unwrap().b, 0.0);
    }
}

     //    .collect::<Vec<u8>>();

     //// split_whitespace is implemented on str and returns a SplitWhitespace<'a str>

