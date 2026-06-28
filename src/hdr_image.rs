// This file is licensed under the EUPL-1.2. See LICENSE.md.

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
//! - Basic tone mappings
//! - Log-average luminance computation
//!
//! ## Example
//!
//! ```rust, no_run
//! use rstrace::color::Color;
//! use rstrace::hdr_image::HDR;
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
use crate::functions::{are_close, endianness_number};
use anyhow::{Result, anyhow};
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{BufReader, Write};

use crate::geometry::Vec2D;
use crate::pfm_func::{Endianness, Parameter, read_pfm};
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

// =================================================================
// PFM Writing
// =================================================================

impl HDR {
    /// Writes the image to a `.pfm` (Portable Float Map) file.
    ///
    /// # Arguments
    /// * `writer` - The output destination (e.g., a file, a memory buffer, or a network socket).
    ///   It accepts any type that implements the [`Write`] trait.
    /// * `endianness` - Byte order used for writing floats
    ///
    /// # Errors
    /// Returns an error if:
    /// - The underlying stream (`writer`) cannot be written to.
    /// - An I/O error occurs while formatting the header or encoding the pixels.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rstrace::hdr_image::HDR;
    /// use std::fs::File;
    /// use std::io::BufWriter;
    ///
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// use rstrace::pfm_func::Endianness;
    /// let img = HDR::new(1920, 1080);
    /// let endianness = Endianness::LittleEndian;
    ///
    /// // Example 1: Writing to a physical file on disk (using BufWriter for optimal performance)
    /// let file = File::create("render.pfm")?;
    /// let mut disk_writer = BufWriter::new(file);
    /// img.write_pfm(&mut disk_writer, &endianness)?;
    ///
    /// // Example 2: Writing directly to RAM (useful for automated tests or passing data to other APIs)
    /// let mut memory_buffer: Vec<u8> = Vec::new();
    /// img.write_pfm(&mut memory_buffer, &endianness)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    /// - The image is written in **binary format**
    /// - Pixels are stored **bottom-to-top** as required by the PFM specification
    /// - The scale factor encodes endianness:
    ///   - Negative = little endian
    ///   - Positive = big endian
    pub fn write_pfm<W: Write>(&self, mut writer: W, endianness: &Endianness) -> Result<()> {
        write!(
            writer,
            "PF\n{} {}\n{:.1}\n",
            self.width,
            self.height,
            endianness_number(endianness)
        )?;

        match endianness {
            Endianness::LittleEndian => {
                for y in (0..self.height).rev() {
                    for x in 0..self.width {
                        let color = self.get_pixel(x, y)?;
                        writer.write_f32::<LittleEndian>(color.r)?;
                        writer.write_f32::<LittleEndian>(color.g)?;
                        writer.write_f32::<LittleEndian>(color.b)?;
                    }
                }
            }
            Endianness::BigEndian => {
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
}

// =================================================================
// Tone Mapping
// =================================================================

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
            no pixel to compute average_luminosity!"
            ));
        }

        let log_sum: f32 = self
            .pixels
            .iter()
            .map(|col| {
                let lum = col.sem_luminosity()?;

                Ok((lum + f32::EPSILON).log10())
            })
            .sum::<Result<f32>>()?;

        let res = 10.0_f32.powf(log_sum / count);
        Ok(res)
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
    pub fn normalization(&mut self, wrapped_a: Option<&f32>) -> Result<()> {
        if self.pixels.is_empty() {
            return Err(anyhow!("normalization(): no pixels to normalize!!!!"));
        }

        let a = *wrapped_a.unwrap_or(&0.18);
        if a <= 0.0 {
            return Err(anyhow!(
                "normalization():\
              Cannot use a non-positive normalization factor such as {a}!!!!"
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

    /// Applies tone mapping to all pixels.
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
        if self.pixels.is_empty() {
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

// ==========================================
// Linear interpolation
// ==========================================
impl HDR {
    /// Samples the image using bilinear interpolation.
    ///
    /// The input coordinates are interpreted in normalized UV space,
    /// where `(0,0)` corresponds to the top-left texel and values
    /// are wrapped periodically into the `[0,1)` range.
    ///
    /// The interpolation is performed using the four neighboring texels.
    ///
    /// # Warning
    ///
    /// UV coordinate are assumed to be positive. No validation is implemented.
    ///
    /// # Errors
    ///
    /// Returns an error if the image contains no pixels.
    pub fn bilinear_interpolation(&self, uv: &Vec2D) -> Result<Color> {
        if self.pixels.is_empty() {
            Err(anyhow!(
                "bilinear_interpolation(): cannot interpolate an empty image"
            ))
        } else {
            // Bilinear interpolation is performed over the surrounding texel cell:
            // (i0,j0) ---- (i1,j0)
            //    |             |
            //    |    (x,y)    |
            //    |             |
            // (i0,j1) ---- (i1,j1)
            let u_wrapped = uv.x - uv.x.floor();
            let v_wrapped = uv.y - uv.y.floor();

            let x = u_wrapped * self.width as f32;
            let y = v_wrapped * self.height as f32;

            let i0 = x.floor() as usize;
            let j0 = y.floor() as usize;
            let i1 = (i0 + 1) % self.width;
            let j1 = (j0 + 1) % self.height;

            let tx = x - i0 as f32;
            let ty = y - j0 as f32;

            let top = (1.0 - tx) * self.pixels[i0 + j0 * self.width]
                + tx * self.pixels[i1 + j0 * self.width];

            let bottom = (1.0 - tx) * self.pixels[i0 + j1 * self.width]
                + tx * self.pixels[i1 + j1 * self.width];

            Ok((1.0 - ty) * top + ty * bottom)
        }
    }
}

// ==========================================
// HDR to LDR
// ==========================================

/// Converts an HDR image stored in `.pfm` (Portable Float Map) format into an LDR image.
///
/// The function performs the following steps:
/// - Reads the input `.pfm` file into an HDR image structure
/// - Applies tone mapping via normalization and clamping
/// - Converts floating-point pixel values to 8-bit RGB using gamma correction
/// - Writes the resulting LDR image to disk
///
/// # Parameters
/// - `argv`: Configuration parameters including:
///   - `input_pfm_file_name`: Path to the input `.pfm` file
///   - `output_file_name`: Path where the LDR image will be saved
///   - `factor_a`: Normalization factor used during tone mapping
///   - `gamma`: Gamma value used for gamma correction (typically ~2.2)
///
/// # Errors
/// Returns an error if:
/// - The input file cannot be read or parsed
/// - Tone mapping operations fail
/// - The output image cannot be written to disk
///
/// # Notes
/// - Pixel values are assumed to be in row-major order
/// - Gamma correction is applied as `x^(1/gamma)`
/// - Output values are clamped to `[0, 1]` before conversion to `[0, 255]`
/// - Negative values stored in HDR will produce `NaN`.
///
/// # Example
/// ```rust, no_run
/// use rstrace::pfm_func::Parameter;
/// use rstrace::hdr_image::hdr_to_ldr;
/// let mut params = Parameter {
///     input_pfm_file_name: "input.pfm".into(),
///     output_file_name: "output.png".into(),
///     factor_a: 1.0,
///     gamma: 2.2,
/// };
///
/// hdr_to_ldr(&mut params).unwrap();
/// ```
pub fn hdr_to_ldr(argv: &mut Parameter) -> Result<()> {
    // Creates HDR object and fill with the .pfm file

    let file = File::open(&argv.input_pfm_file_name);
    let mut reader: BufReader<File> = BufReader::new(file?);

    let mut img = read_pfm(&mut reader)?;

    println!("File {} has been opened and read", argv.input_pfm_file_name);

    // Tone mapping of the HDR image
    img.normalization(Some(&argv.factor_a))?;
    img.sem_clamp_image()?;

    // Create RgbImage box and fill it with the image
    let mut new_img: RgbImage = RgbImage::new(img.width as u32, img.height as u32);

    // Pixel by pixel mapping to LDR

    let to_u8 = |x: f32| {
        let corrected = x.powf(1.0 / argv.gamma);
        (corrected.clamp(0.0, 1.0) * 255.0).round() as u8
    };

    for y in 0..img.height {
        for x in 0..img.width {
            let pixel = &img.pixels[img.width * y + x];

            let r = to_u8(pixel.r);
            let g = to_u8(pixel.g);
            let b = to_u8(pixel.b);

            new_img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }

    // Saving the LDR
    let out_file_name = &argv.output_file_name;
    new_img.save(out_file_name)?;
    println!("File {} has been created", out_file_name);

    Ok(())
}

// ==========================================
// HDR from LDR
// ==========================================

impl HDR {
    /// Converts a flat byte buffer of raw RGB pixel data into a [`Vec<Color>`].
    ///
    /// Each pixel is expected to occupy exactly 3 consecutive bytes in RGB order,
    /// with each channel value in the range `[0, 255]`. The values are cast to
    /// `f32` without any normalization — downstream callers are responsible for
    /// applying gamma correction and tone mapping.
    ///
    /// # Arguments
    /// * `vec` — Raw byte buffer, as returned by [`image::ImageBuffer::into_raw`].
    /// * `width` — Image width in pixels.
    /// * `height` — Image height in pixels.
    ///
    /// # Errors
    /// Returns an error if `vec.len() != width * height * 3` or if `vec` is empty.
    pub fn get_pixels_vector(vec: &[u8], width: usize, height: usize) -> Result<Vec<Color>> {
        let num_pixels = width * height;
        // Avoids creating the vector and mid-operation breaks, not essential
        if vec.len() != num_pixels * 3 || vec.is_empty() {
            return Err(anyhow!(
                "get_pixels_vector(): vector length does not match! \
             expected: {}, got: {}",
                num_pixels * 3,
                vec.len()
            ));
        }

        let pixels = vec
            .chunks_exact(3)
            .map(|chunk| Color::new(chunk[0] as f32, chunk[1] as f32, chunk[2] as f32))
            .collect();

        Ok(pixels)
    }

    /// Loads a PNG or JPEG image from disk and converts it into an [`HDR`] image
    /// by inverting the LDR pipeline.
    ///
    /// The conversion applies two successive per-pixel operations:
    /// 1. **Inverse gamma correction** — decodes the gamma-encoded LDR values back
    ///    to a linear light space (see [`Color::inverse_gamma_correction`]).
    /// 2. **Inverse tone mapping** — recovers approximate HDR luminances by
    ///    inverting the Reinhard operator (see [`Color::inverse_tone_mapping`]).
    ///
    /// # Arguments
    /// * `path` — Path to the input LDR image (PNG or JPEG).
    /// * `factor_a` — The exposure normalization factor assumed to have been used in the
    ///   forward tone mapping. Must be strictly positive and greater than `1e-5`.
    /// * `avr_lum` — The log-average luminance of the original HDR scene, used to
    ///   undo the normalization step. Must be strictly positive and greater than `1e-5`.
    /// * `gamma` — The gamma exponent assumed to have been used during LDR encoding.
    ///   Must be strictly positive and greater than `1e-5`.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `factor_a`, `avr_lum`, or `gamma` are negative or smaller than `1e-5`
    /// - The file at `path` cannot be opened or decoded
    /// - The decoded pixel buffer has an unexpected size
    ///
    /// # Notes
    /// - The recovered HDR values are **approximate**: the inverse pipeline can only
    ///   reconstruct the HDR image up to the precision of the LDR quantization.
    /// - Very dark images (luminance near `0`) will not be recovered accurately, as
    ///   the `1e-5` validation threshold may reject physically valid but extremely
    ///   small parameter values.
    /// - The function accepts any image format supported by the [`image`] crate.
    pub fn load_from_ldr(path: &str, factor_a: f32, avr_lum: f32, gamma: f32) -> Result<Self> {
        if factor_a < 0.0 || are_close(factor_a, 0.0) {
            return Err(anyhow!(
                "load_from_ldr: invalid `a` factor: {factor_a}\na must be positive"
            ));
        }
        if avr_lum < 0.0 || are_close(avr_lum, 0.0) {
            return Err(anyhow!(
                "load_from_ldr: invalid `avr_lum` factor: {avr_lum}\navr_lum must be positive"
            ));
        }
        if gamma < 0.0 || are_close(gamma, 0.0) {
            return Err(anyhow!(
                "load_from_ldr: invalid `gamma` parameter: {gamma}\ngamma must be positive"
            ));
        }

        let img = image::open(path)?.into_rgb8();
        let (width, height) = img.dimensions();
        let (width, height) = (width as usize, height as usize);
        let rgb: Vec<u8> = img.into_raw();
        let mut pixels: Vec<Color> = HDR::get_pixels_vector(&rgb, width, height)?;
        for pixel in &mut pixels {
            pixel.inverse_gamma_correction(gamma);
            pixel.inverse_tone_mapping(factor_a, avr_lum);
        }

        Ok(HDR {
            width,
            height,
            pixels,
        })
    }
}
// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod test {
    use super::*;
    use crate::color::RAINBOW_COLORS;
    use crate::functions::are_close;
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
    fn test_get_and_set_pixel() -> Result<()> {
        let mut hdr = HDR::new(10, 2);
        hdr.set_pixel(
            5,
            1,
            Color {
                r: 1.0,
                g: 2.5,
                b: 10.0,
            },
        )?;
        let pixel = hdr.get_pixel(5, 1)?;
        assert_eq!(pixel.r, 1.0);
        assert_eq!(pixel.g, 2.5);
        assert_eq!(pixel.b, 10.0);
        Ok(())
    }

    #[test]
    fn test_get_pixel_error() {
        let hdr = HDR::new(10, 2);
        assert!(hdr.get_pixel(11, 1).is_err());
    }

    #[test]
    fn test_vector_index() -> Result<()> {
        let x = 9;
        let y = 1;
        let hdr = HDR::new(10, 10);
        assert_eq!(hdr.vector_index(x, y)?, y * hdr.width + x);
        Ok(())
    }

    #[test]
    fn test_check_position() {
        let hdr = HDR::new(10, 55);
        assert!(hdr.check_position(11, 2).is_err());
    }

    #[test]
    fn test_write_pfm() -> Result<()> {
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

        img.set_pixel(0, 0, Color::new(1.0e1, 2.0e1, 3.0e1))?;
        img.set_pixel(1, 0, Color::new(4.0e1, 5.0e1, 6.0e1))?;
        img.set_pixel(2, 0, Color::new(7.0e1, 8.0e1, 9.0e1))?;
        img.set_pixel(0, 1, Color::new(1.0e2, 2.0e2, 3.0e2))?;
        img.set_pixel(1, 1, Color::new(4.0e2, 5.0e2, 6.0e2))?;
        img.set_pixel(2, 1, Color::new(7.0e2, 8.0e2, 9.0e2))?;
        let mut buffer: Vec<u8> = vec![];
        img.write_pfm(&mut buffer, &Endianness::LittleEndian)?;
        assert_eq!(buffer, reference_le_bytes);
        buffer = vec![];
        img.write_pfm(&mut buffer, &Endianness::BigEndian)?;
        assert_eq!(buffer, reference_be_bytes);
        Ok(())
    }

    #[test]
    fn test_sem_clamp_image() -> Result<()> {
        let mut hdr = HDR::new(1, 2);

        hdr.sem_clamp_image()?;
        assert_eq!(hdr.get_pixel(0, 0)?.r, 0.0);

        hdr.set_pixel(
            0,
            0,
            Color {
                r: 1.0,
                g: 2.0e02,
                b: 3.0e03,
            },
        )?;
        hdr.sem_clamp_image()?;

        assert_eq!(hdr.get_pixel(0, 0)?.r, 1.0 / 2.0);
        assert_eq!(hdr.get_pixel(0, 0)?.b, 3.0e3 / (1.0 + 3.0e3));
        assert_eq!(hdr.get_pixel(0, 0)?.g, 2.0e2 / (1.0 + 2.0e2));
        assert_eq!(hdr.get_pixel(0, 1)?.b, 0.0);

        Ok(())
    }

    #[test]
    fn test_average_luminosity() -> Result<()> {
        let img = HDR::new(0, 0);
        assert!(img.average_luminosity().is_err());

        let mut img = HDR::new(1, 4);
        assert!(img.average_luminosity().is_ok());

        assert!(are_close(img.average_luminosity()?, f32::EPSILON));

        let mut sum = 0.0;
        for i in 0..4 {
            let mut color = Color::new(1.0, 20.0, 300.0);
            color = 10.0_f32.powi(i) * color;
            img.set_pixel(0, i as usize, color)?;
            sum += (color.sem_luminosity()? + f32::EPSILON).log10() / 4.0;
        }
        assert_eq!(img.average_luminosity()?, 10.0_f32.powf(sum));

        Ok(())
    }

    #[test]
    fn test_normalization() -> Result<()> {
        // Test the empty image
        let mut img1 = HDR::new(0, 0);
        assert!(img1.normalization(Some(&1.0)).is_err());

        // MODIFICATO: Rimossi i match prolissi. Ora testiamo il risultato direttamente.
        let mut img = HDR::new(1, 4);
        let mut img1 = HDR::new(1, 4);
        let mut img2 = HDR::new(1, 4);

        assert!(img1.normalization(Some(&-1.0)).is_err());
        assert!(img1.normalization(Some(&0.0)).is_err());

        // Fill the HDR image and get the average
        for i in 0..4 {
            let mut color = Color::new(1.0, 20.0, 300.0);
            color = 10.0_f32.powi(i) * color;
            img.set_pixel(0, i as usize, color)?;
            img1.set_pixel(0, i as usize, color)?;
            img2.set_pixel(0, i as usize, color)?;
        }
        let log_average = img.average_luminosity()?;

        // Test the None option
        img1.normalization(None)?;
        assert_eq!(
            img1.get_pixel(0, 0)?.r,
            img.get_pixel(0, 0)?.r * 0.18 / log_average
        );

        // Test the input value option
        img2.normalization(Some(&5.0))?;
        assert_eq!(
            img2.get_pixel(0, 0)?.r,
            img.get_pixel(0, 0)?.r * 5.0 / log_average
        );

        Ok(())
    }

    fn setup_test_rainbow() -> HDR {
        let mut img = HDR::new(4, 2);

        for (i, color) in RAINBOW_COLORS.iter().enumerate() {
            img.pixels[i] = *color;
        }

        img
    }

    #[test]
    fn test_bilinear_interpolation() {
        let hdr_image = setup_test_rainbow();

        let expected = Color {
            r: 0.5,
            g: 0.4,
            b: 0.5,
        };
        assert!(
            expected.is_close(
                &hdr_image
                    .bilinear_interpolation(&Vec2D::new(0.2, 0.25))
                    .unwrap(),
            )
        );
    }

    #[test]
    fn test_bilinear_interpolation_top_border_pixel() {
        let hdr_image = setup_test_rainbow();

        let expected = Color {
            r: 0.74,
            g: 0.6,
            b: 0.34,
        };
        assert!(
            expected.is_close(
                &hdr_image
                    .bilinear_interpolation(&Vec2D::new(0.8, 0.25))
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_bilinear_interpolation_00_case() {
        let hdr_image = setup_test_rainbow();
        let expected = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        assert!(
            expected.is_close(
                &hdr_image
                    .bilinear_interpolation(&Vec2D::new(0.0, 0.0))
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_bilinear_interpolation_wrapping() {
        let hdr_image = setup_test_rainbow();

        let expected = Color {
            r: 0.5,
            g: 0.4,
            b: 0.5,
        };
        assert!(
            expected.is_close(
                &hdr_image
                    .bilinear_interpolation(&Vec2D::new(1.2, 3.25))
                    .unwrap(),
            )
        );
    }

    #[test]
    #[should_panic(expected = "bilinear_interpolation(): cannot interpolate an empty image")]
    fn test_bilinear_interpolation_fail() {
        let hdr_image = HDR::new(0, 0);
        assert!(
            hdr_image
                .bilinear_interpolation(&Vec2D::new(0.1, 0.4))
                .is_err()
        );

        let hdr_image = HDR::new(1, 0);
        assert!(
            hdr_image
                .bilinear_interpolation(&Vec2D::new(0.2, 0.42))
                .is_err()
        );

        let hdr_image = HDR::new(0, 9);
        hdr_image
            .bilinear_interpolation(&Vec2D::new(0.3, 1.5))
            .unwrap();
    }

    #[test]
    fn test_get_pixels_vector_success() {
        let vec: Vec<u8> = vec![100, 0, 20, 32, 3, 9, 10, 11, 255];
        let expected_vec = vec![
            Color::new(100.0, 0.0, 20.0),
            Color::new(32.0, 3.0, 9.0),
            Color::new(10.0, 11.0, 255.0),
        ];
        let result_vec = HDR::get_pixels_vector(&vec, 3, 1).unwrap();
        assert_eq!(
            result_vec.len(),
            3,
            "Expected vec len: 3, vec len: {}",
            result_vec.len()
        );
        for i in 0..3 {
            assert!(
                result_vec[i].is_close(&expected_vec[i]),
                "index: {i}, result_color[i] = {:?}",
                result_vec[i]
            );
        }
    }

    #[test]
    #[should_panic(expected = "get_pixels_vector(): vector length does not match!")]
    fn test_get_pixels_vector_fail1() {
        let vec: Vec<u8> = vec![100, 0, 20, 32, 3, 9, 10, 11];
        let _ = HDR::get_pixels_vector(&vec, 3, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "get_pixels_vector(): vector length does not match!")]
    fn test_get_pixels_vector_fail2() {
        let vec: Vec<u8> = vec![];
        let _ = HDR::get_pixels_vector(&vec, 3000, 10).unwrap();
    }

    #[test]
    #[should_panic(expected = "get_pixels_vector(): vector length does not match!")]
    fn test_get_pixels_vector_fail3() {
        let vec: Vec<u8> = vec![1, 0, 1];
        let _ = HDR::get_pixels_vector(&vec, 3000, 10).unwrap();
    }

    #[test]
    #[should_panic(expected = "load_from_ldr: invalid `a` factor:")]
    fn test_load_from_ldr_invalid_factor_a() {
        let _ = HDR::load_from_ldr("tests/assets/pixar_ball.png", -10.0, 1.0, 1.0).unwrap();
    }

    #[test]
    #[should_panic(expected = "load_from_ldr: invalid `a` factor:")]
    fn test_load_from_ldr_invalid_factor_a_null() {
        let _ = HDR::load_from_ldr("tests/assets/pixar_ball.png", 0.0000001, 1.0, 1.0).unwrap();
    }

    #[test]
    #[should_panic(expected = "load_from_ldr: invalid `avr_lum` factor:")]
    fn test_load_from_ldr_invalid_factor_lum() {
        let _ = HDR::load_from_ldr("tests/assets/pixar_ball.png", 10.0, -1.0, 1.0).unwrap();
    }

    #[test]
    #[should_panic(expected = "load_from_ldr: invalid `avr_lum` factor:")]
    fn test_load_from_ldr_invalid_factor_lum_null() {
        let _ = HDR::load_from_ldr("tests/assets/pixar_ball.png", 10.0, 0.0000001, 1.0).unwrap();
    }

    #[test]
    #[should_panic(expected = "load_from_ldr: invalid `gamma` parameter:")]
    fn test_load_from_ldr_invalid_factor_gamma_null() {
        let _ = HDR::load_from_ldr("tests/assets/pixar_ball.png", 10.0, 0.1, 0.0).unwrap();
    }

    #[test]
    #[should_panic(expected = "load_from_ldr: invalid `gamma` parameter:")]
    fn test_load_from_ldr_invalid_factor_gamma() {
        let _ = HDR::load_from_ldr("tests/assets/pixar_ball.png", 10.0, 0.1, -1.0).unwrap();
    }
}
