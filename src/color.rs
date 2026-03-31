use std::ops::{Add, Div, Mul};
use anyhow::{Result, anyhow};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// ====================
//     Constructor
//     and methods
// ====================

// ----- Constructor ------
impl Color {
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        // Conviene mettere Result? Mi interessa bloccare tutto?
        // È un controllo troppo pesante?
        if !(red>=0.0 && green>=0.0 && blue>=0.0)
            || !(red.is_finite() && green.is_finite() && blue.is_finite()) {
            panic!("Color constructor:
            invalid color red({}), green({}), blue({})", red, green, blue);
        }
        Color {
            r: red.abs(),
            g: green.abs(),
            b: blue.abs(),
        } // The .abs() is to transform -0.0 -> +0.0
    }

    fn condition(&self)->bool{
        // Has this color all correct values?
        // Must be a Real, positive number!
        self.r.is_finite() && self.r.is_sign_positive() &&
            self.g.is_finite() && self.g.is_sign_positive() &&
            self.b.is_finite() && self.b.is_sign_positive()
    }

    pub fn self_check(&self) -> Result<()> {
        if self.condition() {
            Ok(())
        } else {
            Err(anyhow!("invalid color: red({}), green({}), blue({})",
                   self.r, self.g, self.b))
        }
    }
}
// ----- Empty Constructor ------
impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

// ----- Tone mapping methods ------
impl Color{
    pub fn sem_luminosity(&self) -> Result<f32> {
        self.self_check()?;
        // Shirley & Morley’s formula
        let max = self.r.max(self.g.max(self.b));
        let min = self.r.min(self.g.min(self.b));
        Ok((max + min) * 0.5)
    }

    pub fn clamp(& mut self) -> Result<()> {
        self.self_check()?;
        self.r = self.r / (self.r + 1.0);
        self.g = self.g / (self.g + 1.0);
        self.b = self.b / (self.b + 1.0);
        Ok(())
    }
}

// ====================
// Trait implementation
// ====================

// Color + Color
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

// Color * Color
impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

// Color * float
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: self.r * rhs,
            b: self.b * rhs,
            g: self.g * rhs,
        }
    }
}

// float * Color
impl Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self * rhs.r,
            b: self * rhs.b,
            g: self * rhs.g,
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

        Color {
            r: self.r / rhs,
            b: self.b / rhs,
            g: self.g / rhs,
        }
    }
}

// Test implementation
#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions;

    #[test]
    fn test_empty_constructor() {
        let c = Color::default();
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_constructor() {
        let c = Color::new(0.1, 0.2, 0.3);
        assert_eq!(c.r, 0.1);
        assert_eq!(c.g, 0.2);
        assert_eq!(c.b, 0.3);
    }

    #[test]
    #[should_panic]
    fn test_constructor_2(){ let _ = Color::new(-0.1, 0.2, 0.3);}

    #[test]
    fn test_self_check(){
        let mut color = Color::new(1.0, 0.2, 0.3);
        assert!(color.self_check().is_ok());
        color.b = -0.0;
        assert!(color.self_check().is_err());
        color.b = -1.0;
        assert!(color.self_check().is_err());
        color.b = f32::INFINITY;
        assert!(color.self_check().is_err());
        color.b = f32::NEG_INFINITY;
        assert!(color.self_check().is_err());
        color.b = f32::NAN;
        assert!(color.self_check().is_err());
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

        assert_eq!(c1 + c2, c3);
    }

    // test product Color-Color
    #[test]
    fn product_col_col() {
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
            r: 4.0,
            g: 10.0,
            b: 18.0,
        };

        assert_eq!(c1 * c2, c3);
    }

    // Test Color * scalar
    #[test]
    fn test_color_times_scalar() {
        let col: Color = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        let scalar: f32 = 2.5;
        let expected = Color {
            r: 2.5,
            g: 5.0,
            b: 7.5,
        };

        assert_eq!(col * scalar, expected);

        let scalar: f32 = -1.0 / 3.0;
        let expected = Color {
            r: -1.0 / 3.0,
            g: -2.0 / 3.0,
            b: -1.0,
        };

        assert_eq!(col * scalar, expected);
    }

    // Test scalar * Color
    #[test]
    fn test_scalar_times_colors() {
        let col: Color = Color {
            r: 1.0,
            g: 20.0,
            b: 35.0,
        };
        let scalar: f32 = 2.5;
        let expected = Color {
            r: 2.5,
            g: 50.0,
            b: 87.5,
        };
        assert_eq!(scalar * col, expected);

        let scalar: f32 = -10.1;
        let expected = Color {
            r: -10.1,
            g: -202.0,
            b: -353.5,
        };
        assert_eq!(scalar * col, expected);
    }

    // Test Color / scalar
    #[test]
    fn test_div() {
        let col = Color {
            r: 2.5,
            g: 50.0,
            b: 87.5,
        };
        let scalar: f32 = -2.5;
        let expected = Color {
            r: -1.0,
            g: -20.0,
            b: -35.0,
        };
        assert_eq!(col / scalar, expected);
    }

    // Test Color/0.0
    #[test]
    #[should_panic(expected = "Cannot divide by zero-valued `Color`!")]
    fn divide_by_zero() {
        let col = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        let scalar: f32 = 0.0;
        let _ = col / scalar;
    }


    #[test]
    fn test_sem_luminosity() {
        let color1 = Color::new(1.0, 2.0,3.0);
        assert!(
            functions::are_close(color1.sem_luminosity().unwrap(), 0.5 * (1.0 + 3.0)),
            "TEST_ERROR: sem_luminosity is incorrect!"
        );
        let color1 = Color::new(10.0, 2.0,12.0);
        assert!(
            functions::are_close(color1.sem_luminosity().unwrap(), 0.5 * (12.0 + 2.0)),
            "TEST_ERROR: sem_luminosity is incorrect!"
        );
    }

    #[test]
    fn test_clamp(){
        panic!("YOU NEED TO WRITE THE TEST!!!");
    }

    #[test]
    fn magic_test(){
        println!("Magic test {}", -0.0 == 0.0);
    }
}