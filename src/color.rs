use std::ops::{Add, Div, Mul};

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// Empty constructor
impl Color {
    pub fn new() -> Self {
        Color{r: 0.0, g: 0.0, b: 0.0}
    }
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

// Scalar multiplication and division
// Color * float
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Color{
            r : self.r * rhs,
            b : self.b * rhs,
            g : self.g * rhs,
        }
    }
}

// float * Color
impl Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color{
            r : self * rhs.r,
            b : self * rhs.b,
            g : self * rhs.g,
        }
    }
}


// Division by a scalar
impl Div<f32> for Color {
    type Output = Color;
    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            panic!("Cannot divide by zero-valued `Color`!");
        }
        
        Color{
            r: self.r / rhs,
            b: self.b / rhs,
            g : self.g / rhs,
        }
    }
}

// Test implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_constructor() {
        let c = Color::new();
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

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

    #[test]
    fn test_color_times_scalar(){
        let col: Color = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        let scalar:f32 = 2.5;
        let expected = Color{
            r : 2.5,
            g : 5.0,
            b : 7.5,
        };
        let result = col * scalar;
        assert_eq!(result.r, expected.r);
        assert_eq!(result.b, expected.b);
        assert_eq!(result.g, expected.g);

        let scalar:f32 = - 1.0 / 3.0;
        let expected = Color{
            r : -1.0/3.0,
            g : -2.0/3.0,
            b : -1.0,
        };
        let result = col * scalar;
        assert_eq!(result.r, expected.r);
        assert_eq!(result.g, expected.g);
        assert_eq!(result.b, expected.b);
    }

    #[test]
    fn test_scalar_times_colors(){
        let col: Color = Color {
            r: 1.0,
            g: 20.0,
            b: 35.0,
        };
        let scalar:f32 = 2.5;
        let result = scalar * col;
        let expected = Color{
            r: 2.5,
            g: 50.0,
            b: 87.5,
        };
        assert_eq!(result.r, expected.r);
        assert_eq!(result.g, expected.g);
        assert_eq!(result.b, expected.b);

        let scalar:f32 = - 10.1;
        let result = scalar * col;
        let expected = Color{
            r: -10.1,
            g: -202.0,
            b: -353.5,
        };
        assert_eq!(result.r, expected.r);
        assert_eq!(result.g, expected.g);
        assert_eq!(result.b, expected.b);
    }
    
    #[test]
    fn test_div(){
        let col = Color{
            r: 2.5,
            g: 50.0,
            b: 87.5,
        };
        let scalar:f32 = - 2.5;
        let result = col / scalar;
        let expected = Color{
            r: -  1.0,
            g: - 20.0,
            b: - 35.0,
        };
        assert_eq!(result.r, expected.r);
        assert_eq!(result.g, expected.g);
        assert_eq!(result.b, expected.b);
    }
}