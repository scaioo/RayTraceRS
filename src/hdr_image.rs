use crate::color;
use crate::color::Color;
use anyhow::{anyhow, Result};

#[derive(Clone, Debug)]
pub struct HDR{
    pub width : usize,
    pub height : usize,
    pub pixels : Vec<Color>,
}

impl HDR{
    // Implement the full-black image
    pub fn new(width : usize, height : usize) -> HDR{
        let pixels = vec![Color { r: 0.0, g: 0.0, b: 0.0 }; width * height];
        HDR {
            width,
            height,
            pixels,
        }
    }
    
    /*===============    NOTE    ================
          CHECK AND DUBLECHECK WITH TOMASI!!!
                  is it the best way?
                 better to propagate?
     ============================================*/

    pub fn set_pixel(&mut self, x : usize, y : usize, color : Color){
        match self.check_position(x, y){
            Ok(_) => self.pixels[y * self.width + x] = color,
            Err(_) => panic!("Pixel Error: pixel {},{} out of bound!", x,y)
        }
    }

    pub fn get_pixel(&self, x : usize, y : usize) -> Color{
        match self.check_position(x, y){
            Ok(_) => self.pixels[y * self.width + x],
            Err(_) => panic!("Pixel Error: pixel {},{} out of bound!", x,y)
        }

        /*               AN ALTERNATIVE:
         in this case fn -> Result<&HDR>: we propagate the Err

        self.check_position(x, y)?;
        Ok(self.pixels[y * self.width + x])
         */

    }
    
    fn check_position(&self, x : usize, y : usize) -> Result<&HDR>{
        if x < self.width && y < self.height{
            Ok(&self)
        } else { Err(anyhow!("NON-EXISTING PIXEL ({},{})", x, y))}
    }
}


mod test{
    use super::*;

    // Test for
    #[test]
    fn test_new(){
        let hdr = HDR::new(10, 55);
        assert_eq!(hdr.width, 10);
        assert_eq!(hdr.height, 55);
        assert_eq!(hdr.pixels.len(), 550);
        let all_black = hdr.pixels
            .iter()
            .all(|p| p.r == 0.0 && p.g == 0.0 && p.b == 0.0);
        assert!(all_black,"Not all pixels were initialized to black!");
    }

    #[test]
    fn test_set_pixel(){
        let mut hdr = HDR::new(10, 2);
        hdr.set_pixel(5, 1, Color { r: 1.0, g: 2.5, b: 10.0 });
        let pixel = hdr.get_pixel(5, 1);
        assert_eq!(pixel.r, 1.0);
        assert_eq!(pixel.g, 2.5);
        assert_eq!(pixel.b, 10.0);
    }

    #[test]
    #[should_panic]
    fn test_check_position(){
        let hdr = HDR::new(10, 55);
        hdr.check_position(11, 2).unwrap();
    }// NOTE : it would be best to integrate this test with Result options!!!!
}