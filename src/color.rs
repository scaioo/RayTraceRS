use std::ops::{Add, Mul};

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// Implementation of the sum (+) for type Color
impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}


// implementing Color-Color product (*)

impl Mul for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

// Test Add implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let c1: Color = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        let c2: Color = Color {
            r: 4.0,
            g: 5.0,
            b: 6.0,
        };
        let c3: Color = Color {
            r: 5.0,
            g: 7.0,
            b: 9.0,
        };
        let sum: Color = c1 + c2;

        assert_eq!(sum.r, c3.r);
        assert_eq!(sum.g, c3.g);
        assert_eq!(sum.b, c3.b);
    }

    // test product Color-Color
    #[test]
    fn product_Col_Col(){
        let c1: Color = Color{
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };

        let c2: Color = Color{
            r: 4.0,
            g: 5.0,
            b: 6.0,
        };

        let c3: Color = Color{
            r: 4.0,
            g: 10.0,
            b: 18.0,
        };

        let prod_c1_c2: Color = c1*c2;

        assert_eq!(prod_c1_c2.r, c3.r);
        assert_eq!(prod_c1_c2.g, c3.g);
        assert_eq!(prod_c1_c2.b, c3.b);
    }
}
