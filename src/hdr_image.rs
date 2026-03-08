use crate::color;
use crate::color::Color;

#[derive(Clone, Debug)]
pub struct HDR{
    pub width : usize,
    pub height : usize,
    pub pixels : Vec<Color>,
}

// Implement the full-black image
impl HDR{
    pub fn new(width : usize, height : usize) -> HDR{
        let pixels = vec![Color { r: 0.0, g: 0.0, b: 0.0 }; width * height];
        HDR {
            width,
            height,
            pixels,
        }
    }

    pub fn set_pixel(&mut self, x : usize, y : usize, color : Color){
        // Control on the position needed 
        self.pixels[y * self.width + x] = color;
    }

    pub fn get_pixel(&self, x : usize, y : usize) -> Color{
        // Control on the position needed 
        self.pixels[y * self.width + x]
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
}