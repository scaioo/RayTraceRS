use std::ops::{Add, Div, Mul};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// Constructor
impl Color {
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Color {
            r: red,
            g: green,
            b: blue,
        }
    }
}
// Empy constructor
impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

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
}
