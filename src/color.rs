//! Utility types and operations for colors used in the ray tracing crate.
//!
//! The code in this module is written with two goals in mind:
//! - make invalid states easy to detect during development;
//! - allow the validation checks to be removed later, once the renderer is
//!   finalized and performance becomes more important.
//!
//! In the final version of the ray tracer, some checks are expected to be
//! removed or reduced. For this reason, arithmetic operations do not enforce
//! validity automatically, and callers remain responsible for preserving
//! physically meaningful values when needed.

use std::ops::{Add, Div, Mul};
use anyhow::{Result, anyhow};

/// RGB color stored as three linear floating-point components.
///
/// # Invariants
/// A physically valid color should have:
/// - finite components
/// - non-negative components
///
/// # Validation policy
/// During development, [`Color::new`] and [`Color::self_check`] enforce these
/// invariants to help detect errors early.
///
/// In the future, these validation checks are expected to be removed or
/// reduced for performance reasons. As a consequence:
/// - arithmetic operations (`Add`, `Mul`, `Div`) do not enforce validity;
/// - intermediate colors may temporarily contain invalid values such as
///   negative numbers, `NaN`, or `∞`;
/// - preserving physically meaningful values becomes the responsibility of
///   the caller.
///
/// # Notes
/// This design is common in ray tracing pipelines, where intermediate values
/// may fall outside the physically meaningful range and are later corrected
/// by tone mapping or other post-processing steps.
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

impl Color {
    /// Creates a new validated color.
    ///
    /// # Panics
    /// Panics if any component is negative or not finite.
    ///
    /// # Notes
    /// `-0.0` is normalized to `+0.0`.
    ///
    /// This validation is intended to help during development. In the final
    /// optimized renderer, these checks may be removed for performance.
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        // Questi controlli saranno da eliminare non appena inizieremo
        // a implementare l'algoritmo di RayTracing
        if !(red >= 0.0 && green >= 0.0 && blue >= 0.0)
            || !(red.is_finite() && green.is_finite() && blue.is_finite()) {
            panic!(
                "Color constructor:\ninvalid color red({}), green({}), blue({})",
                red, green, blue
            );
        }

        Color {
            r: red.abs(),
            g: green.abs(),
            b: blue.abs(),
        } // The .abs() is to transform -0.0 -> +0.0
    }

    fn condition(&self) -> bool {
        // Has this color all correct values?
        // Must be a Real, positive number!
        self.r.is_finite() && self.r.is_sign_positive()
            && self.g.is_finite() && self.g.is_sign_positive()
            && self.b.is_finite() && self.b.is_sign_positive()
    }

    /// Verifies that the color satisfies the validity invariants.
    ///
    /// # Errors
    /// Returns an error if any component is negative or not finite.
    pub fn self_check(&self) -> Result<()> {
        if self.condition() {
            Ok(())
        } else {
            Err(anyhow!(
                "invalid color: red({}), green({}), blue({})",
                self.r, self.g, self.b
            ))
        }
    }

    /// Computes the semi-luminance of the color
    /// by using the Shirley & Morley’s formula.
    ///
    /// The formula is:
    /// `(max(r, g, b) + min(r, g, b)) / 2`
    ///
    /// # Errors
    /// Returns an error if the color is invalid.
    ///
    /// # Notes
    /// This is not a perceptual luminance measure.
    pub fn sem_luminosity(&self) -> Result<f32> {
        self.self_check()?;
        // Shirley & Morley’s formula
        let max = self.r.max(self.g.max(self.b));
        let min = self.r.min(self.g.min(self.b));
        Ok((max + min) * 0.5)
    }

    /// Applies a simple tone-mapping transform in place.
    ///
    /// Each component is mapped as:
    /// `c -> c / (c + 1)`
    ///
    /// This compresses high dynamic range values into the interval `[0, 1)`.
    ///
    /// # Errors
    /// Returns an error if the color is invalid.
    ///
    /// # Note
    /// Despite the name, this is not a traditional tone_map_reinhard operation.
    pub fn tone_map_reinhard(&mut self) -> Result<()> {
        self.self_check()?;
        self.r = self.r / (self.r + 1.0);
        self.g = self.g / (self.g + 1.0);
        self.b = self.b / (self.b + 1.0);
        Ok(())
    }
}

/// Returns the default black color `(0.0, 0.0, 0.0)`.
impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

// ====================
// Trait implementation
// ====================

/// Component-wise addition of two colors.
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

/// Component-wise multiplication of two colors.
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

/// Multiplies each component by a scalar.
impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

/// Multiplies each component by a scalar.
///
/// This implementation allows writing `scalar * color`.
impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self * rhs.r,
            g: self * rhs.g,
            b: self * rhs.b,
        }
    }
}

/// Divides each component by a scalar.
///
/// # Panics
/// Panics if `rhs == 0.0`.
impl Div<f32> for Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            panic!("Cannot divide by zero-valued `Color`!");
        }

        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
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
    fn test_constructor_2() {
        let _ = Color::new(-0.1, 0.2, 0.3);
    }

    #[test]
    fn test_self_check() {
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
        let color1 = Color::new(1.0, 2.0, 3.0);
        assert!(
            functions::are_close(color1.sem_luminosity().unwrap(), 0.5 * (1.0 + 3.0)),
            "TEST_ERROR: sem_luminosity is incorrect!"
        );
        let color1 = Color::new(10.0, 2.0, 12.0);
        assert!(
            functions::are_close(color1.sem_luminosity().unwrap(), 0.5 * (12.0 + 2.0)),
            "TEST_ERROR: sem_luminosity is incorrect!"
        );
    }

    #[test]
    fn test_clamp() {
        let mut color = Color::new(0.0, 2.5, 5.0);
        color.r = f32::NAN;
        assert!(color.tone_map_reinhard().is_err());
        color.r = 1.0;
        color.tone_map_reinhard().unwrap();
        assert_eq!(color.r, 1.0 / (1.0 + 1.0));
        assert_eq!(color.g, 2.5 / (2.5 + 1.0));
        assert_eq!(color.b, 5.0 / (5.0 + 1.0));
    }
}