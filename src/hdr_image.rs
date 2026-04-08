use crate::color::Color;
use anyhow::{Result, anyhow};
use endianness::{ByteOrder};
use std::fs::File;
use std::path::Path;
use std::io::{Write,BufReader};
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use crate::functions::endianness_number;

#[derive(Clone, Debug, PartialEq)]
pub struct HDR {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl HDR {
    // Implement the full-black image
    pub fn new(width: usize, height: usize) -> HDR {
        let pixels = vec![Color::default(); width * height];
        HDR {
            width,
            height,
            pixels,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<()> {
        self.check_position(x, y)?;
        self.pixels[y * self.width + x] = color;
        Ok(())
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        //Ok(self.pixels[y * self.width + x])  Is it better this previous version??
        Ok(self.pixels[self.vector_index(x, y)?])
    }

    pub fn vector_index(&self, x: usize, y: usize) -> Result<usize> {
        self.check_position(x, y)?;
        Ok(x + y * self.width)
    }

    fn check_position(&self, x: usize, y: usize) -> Result<()> {
        if x < self.width && y < self.height {
            Ok(())
        } else {
            Err(anyhow!("OUT OF BOUND PIXEL ({},{})!", x, y))
        }
    }

    pub fn create_stream<P: AsRef<Path>>(path: P) -> Result<BufReader<File>>{
        let file = File::open(path)?;
        let stream = BufReader::new(file);
        Ok(stream)
    }
    pub fn write_pfm<W: Write>(&self, mut writer: W, endianness: &ByteOrder) -> anyhow::Result<()> {

        write!(writer, "PF\n{} {}\n{:.1}\n", self.width, self.height, endianness_number(endianness))?;

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
}

// ====================
//     Tone Mapping
// ====================

impl HDR {
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

    pub fn sem_clamp_image(&mut self) -> Result<()> {
        if self.pixels.len() == 0 {
            return Err(anyhow!("clamp_image(): no pixel to clamp!!!!!"));
        }
        for color in self.pixels.iter_mut() {
            color.clamp()?;
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
        let reference_le_bytes = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a,
            0x00, 0x00, 0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43,
            0x00, 0x00, 0xc8, 0x43, 0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44,
            0x00, 0x00, 0x2f, 0x44, 0x00, 0x00, 0x48, 0x44, 0x00, 0x00, 0x61, 0x44,
            0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41,
            0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00, 0x70, 0x42,
            0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42
        ];

        let reference_be_bytes = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42,
            0xc8, 0x00, 0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43,
            0xc8, 0x00, 0x00, 0x43, 0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44,
            0x2f, 0x00, 0x00, 0x44, 0x48, 0x00, 0x00, 0x44, 0x61, 0x00, 0x00, 0x41,
            0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41, 0xf0, 0x00, 0x00, 0x42,
            0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00, 0x00, 0x42,
            0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00
        ];
        let mut img = HDR::new(3,2);

        img.set_pixel(0, 0, Color::new(1.0e1, 2.0e1, 3.0e1)).unwrap(); // Each component is
        img.set_pixel(1, 0, Color::new(4.0e1, 5.0e1, 6.0e1)).unwrap(); // different from any
        img.set_pixel(2, 0, Color::new(7.0e1, 8.0e1, 9.0e1)).unwrap(); // other: important in
        img.set_pixel(0, 1, Color::new(1.0e2, 2.0e2, 3.0e2)).unwrap(); // tests!
        img.set_pixel(1, 1, Color::new(4.0e2, 5.0e2, 6.0e2)).unwrap();
        img.set_pixel(2, 1, Color::new(7.0e2, 8.0e2, 9.0e2)).unwrap();
        let mut buffer: Vec<u8> = vec![];
        img.write_pfm(&mut buffer,&ByteOrder::LittleEndian).unwrap();
        assert_eq!(buffer,reference_le_bytes);
        buffer = vec![];
        img.write_pfm(&mut buffer,&ByteOrder::BigEndian).unwrap();
        assert_eq!(buffer,reference_be_bytes);
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

