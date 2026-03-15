use crate::color::Color;
use anyhow::{Result, anyhow};
use std::io::BufWriter;
use std::path::Path;
use std::fs::File;
use endianness::ByteOrder;
use crate::functions;


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

    pub fn write_pfm(&self, filename: &str, endianness : &ByteOrder) -> Result<()> {
        // Create the new file with name 'filename'

        let path = Path::new(filename); //Create a path to make the new file
        let display = path.display();  //Create a variable to print the path
        let mut file = match File::create(filename) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        // Need to find a way to write in the line.
        // How can I write in bytes?

        // Later I will need this...
        let ENDIAN = functions::endianness_number(endianness);

        Ok(())
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color)->Result<()> {
        self.check_position(x, y)?;
        self.pixels[y * self.width + x] = color;
        Ok(())
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        //Ok(self.pixels[y * self.width + x])  Is it better this previous version??
        Ok(self.pixels[self.vector_index(x,y)?])
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
        ).unwrap();
        let pixel = hdr.get_pixel(5, 1).unwrap();
        assert_eq!(pixel.r, 1.0);
        assert_eq!(pixel.g, 2.5);
        assert_eq!(pixel.b, 10.0);
    }
    #[test]
    fn test_get_pixel(){
        let mut hdr = HDR::new(10, 2);
        let color = Color{r: 1.0, g: 0.2, b: 30.0};
        hdr.set_pixel(1,1,color).unwrap();
        let pixel = hdr.get_pixel(1,1).unwrap();
        assert_eq!(pixel.r, color.r);
        assert_eq!(pixel.g, color.g);
        assert_eq!(pixel.b, color.b);
    }

    #[test]
    #[should_panic]
    fn test_get_pixel_panic() {
        let hdr = HDR::new(10, 2);
        let _ = hdr.get_pixel(11,1).unwrap();
    }

    #[test]
    fn test_vector_index() {
        let x = 9; let y = 1;
        let hdr = HDR::new(10, 10);
        assert_eq!(hdr.vector_index(x,y).unwrap(), y * hdr.width +x );
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

}