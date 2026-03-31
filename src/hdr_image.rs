use crate::color::Color;
use anyhow::{Result, anyhow};
use endianness::{ByteOrder, EndiannessResult};
use std::fs::File;
use std::io;
//use std::io::BufWriter;
use std::path::Path;
use std::io::BufRead;
//use std::num::ParseIntError;
//use endianness::ByteOrder::{BigEndian, LittleEndian};

#[derive(Clone, Debug, PartialEq)]
pub struct HDR {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

// ====================
//     Constructors
// ====================

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
}

// ====================
//   I/O operations
// ====================

impl HDR{
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
}

pub enum EndiannessError{
    InvalidValue
}

// reading and writing pfm files

//read_line already exists in Rust's standard library

//      USEREI UNA TUPLA, NON UN VETTORE!
//       Result<(u8,u8),EndiannessError>
pub fn _parse_img_size(filename: &str) -> Result<Vec<u8>, anyhow::Error> {

    let file = File::open(filename);
    let mut reader = io::BufReader::new(file.unwrap());
    let mut line: String = String::new();

    reader.read_line(&mut line).unwrap();
    line = line.trim().to_string();

    if line != "PF" && line != "Pf" {
        println!("NON è PFM RENDILO UN ERRORE");
        println!("{}", line);
    }

    //// checks the dimension of the image
    line.clear();

    reader.read_line(&mut line).unwrap();

    // turns the strings (created by split_whitespace into numbers (cols and rows)
    let   line_u8 = line.split_whitespace()
        .map(|x| x.parse::<u8>())
        .collect::<Result<Vec<u8>, _>>();

    match line_u8 {
        Ok(line) => {
            return Ok(line);
        }
        Err(e) => {
            println!("{:?}", e);
            return Err(anyhow::anyhow!("something's wrong with image dimensions declared in pfm"));
        }
    }
}

// read endianness takes the name of a file as an input
//Result<ByteOrder, &str>
pub fn _parse_endianness(filename: &str) -> Result<ByteOrder, EndiannessError> {
    let file = File::open(filename);
    let mut reader = io::BufReader::new(file.unwrap());
    let mut line: String = String::new();

    // reads PF line (read_line reads the lines in order,
    // to read the third I need to read the other two first
    reader.read_line(&mut line).unwrap();

    // reads line cols rows
    line.clear();
    reader.read_line(&mut line).unwrap();

    //reads endianness
    line.clear();
    reader.read_line(&mut line).unwrap();
    let endianness_number: f32 = line.trim().parse().unwrap();

    println!("{}", line.trim());

    if endianness_number > 0.0 {
        Ok(ByteOrder::BigEndian)
    } else if endianness_number < 0.0 {
        Ok(ByteOrder::LittleEndian)
    } else {
        Err(EndiannessError::InvalidValue)
    }
}

// ====================
//     Tone Mapping
// ====================

impl HDR {
    pub fn average_luminosity(&self) -> Result<f32> {
        let count = self.pixels.len() as f32;
        if count == 0.0 {
            return Err(anyhow!("\naverage_luminosity():\npixel.len() == 0!!!!!"));
        }

        let log_sum: f32 = self.pixels.iter()
            .map(|col| (col.sem_luminosity() + f32::EPSILON).log10())
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
            color.clamp();
        }

        Ok(())
    }
}

// ====================
//        Tests
// ====================

#[cfg(test)]
mod test {
    use super::*;
    use crate::functions;
    // ---------- Constructors ----------
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
    // ---------- I/O operators ----------
    #[test]
    fn test_write_pfm() {
        panic!("YOU NEED TO WRITE THE TEST!!!")
    }

    // ---------- Tone Mapping ----------
    #[test]
    fn test_average_luminosity() {

        let hdr = HDR::new(10, 55);
        assert!(functions::are_close(hdr.average_luminosity().unwrap(),0.0));

        let mut hdr = HDR::new(2, 3);
        for i in 0..3{
            hdr.set_pixel(0,i,Color{r:5.0,g:5.0,b:15.0}).unwrap();
            hdr.set_pixel(1,i,Color{r:1.0,g:2.0,b:3.0}).unwrap();
        }

        let mut log_avr : f32 = 0.0;
        let epsilon = f32::EPSILON;
        for i in 0..3{
            log_avr += (10.0 + epsilon).log10();
            log_avr += (2.0 + epsilon).log10();
        }
        log_avr /= hdr.pixels.len() as f32;

        assert!(functions::are_close(
            hdr.average_luminosity().unwrap(),
            10.0_f32.powf(log_avr)
        ));

    }

    #[test]
    fn test_normalization() {
        panic!("YOU NEED TO WRITE THE TEST!!!");
    }
}

     //    .collect::<Vec<u8>>();

     //// split_whitespace is implemented on str and returns a SplitWhitespace<'a str>

