// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Utility types and operations for colors used in the ray tracing crate.
//!
//! This module provides the basic architecture to treat a single pixel color.
//!
//! The code in this module is written with two goals in mind:
//! - make invalid states easy to detect during development;
//! - allow validation checks to be removed later for performance.
//!
//! In the final version, most checks must be reduced or removed.
//! Arithmetic operations do not enforce validity, so callers are
//! responsible for preserving physically meaningful values.

use crate::functions::are_close;
use anyhow::{Result, anyhow};
use std::ops::{Add, AddAssign, Div, Mul};

/// RGB color stored as three linear floating-point components.
///
/// Arithmetic operations do not enforce validity, so intermediate values
/// may be outside the physically meaningful range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// =================================================================
//     Constructor
//     and methods
// =================================================================

impl Color {
    /// Creates a new `Color`.
    ///
    /// # Examples
    /// ```rust
    /// use rstrace::color::Color;
    ///
    /// let c = Color::new(0.2, 0.4, 0.6);
    /// assert_eq!(c.r, 0.2);
    /// ```
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Color {
            r: red,
            g: green,
            b: blue,
        }
    }

    /// Returns `true` if all components are approximately equal.
    ///
    /// Component-wise comparison is performed using [`are_close`].
    ///
    /// This method is mainly intended for testing floating-point computations.
    pub fn is_close(&self, other: &Color) -> bool {
        are_close(self.r, other.r) && are_close(self.g, other.g) && are_close(self.b, other.b)
    }

    /// Return false if any stored color is not a positive real number.
    fn is_valid(&self) -> bool {
        // Has this color all correct values?
        // Must be a Real, positive number!
        self.r.is_finite()
            && self.r.is_sign_positive()
            && self.g.is_finite()
            && self.g.is_sign_positive()
            && self.b.is_finite()
            && self.b.is_sign_positive()
    }

    /// Verifies that the color satisfies the validity invariants.
    ///
    /// # Errors
    /// Returns an error if any component is negative, `NaN`, or infinite (`INFINITY`).
    pub fn self_check(&self) -> Result<()> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(anyhow!(
                "invalid color: red({}), green({}), blue({})",
                self.r,
                self.g,
                self.b
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
    ///
    /// # Examples
    /// ```rust
    /// use rstrace::color::Color;
    ///
    /// let mut c = Color::new(2.0, 0.0, 0.0);
    /// c.tone_map().unwrap();
    ///
    /// assert!(c.r < 1.0);
    /// ```
    pub fn tone_map(&mut self) -> Result<()> {
        self.self_check()?;
        self.r = self.r / (self.r + 1.0);
        self.g = self.g / (self.g + 1.0);
        self.b = self.b / (self.b + 1.0);
        Ok(())
    }

    pub fn rescale(&mut self) -> Result<()> {
        self.self_check()?;
        let highest = self.r.max(self.g.max(self.b));
        if highest > 1.0 {
            self.r = self.r / highest;
            self.g = self.g / highest;
            self.b = self.b / highest;
        }
        Ok(())
    }

    /// Applies inverse gamma correction to convert an LDR pixel from gamma-encoded
    /// space back to a linear light value.
    ///
    /// Each channel is decoded as:
    ///
    /// ```text
    /// c_linear = (c_encoded / 256.0) ^ gamma
    /// ```
    ///
    /// # Arguments
    /// * `gamma` — The gamma exponent used during the original encoding (typically `2.2`).
    ///   Must be positive; validation is delegated to the caller ([`HDR::load_from_ldr`]).
    ///
    /// # Notes
    /// - The input channel values are assumed to be in the range `[0, 255]` (raw `u8`
    ///   cast to `f32`).
    /// - The divisor is `256.0` rather than `255.0`: this keeps pure white (`255`)
    ///   slightly below `1.0`, which avoids a division by zero in the subsequent
    ///   [`inverse_tone_mapping`] step (where `1 - c` appears in the denominator).
    ///   As a result, white is recovered as `≈ 0.996` rather than exactly `1.0`.
    pub fn inverse_gamma_correction(&mut self, gamma: f32) {
        // Validation must be done when application
        self.r = (self.r / 256.0_f32).powf(gamma);
        self.g = (self.g / 256.0_f32).powf(gamma);
        self.b = (self.b / 256.0_f32).powf(gamma);
    }

    /// Applies the inverse of the Reinhard tone mapping operator, recovering an
    /// approximate HDR luminance from a clamped LDR value.
    ///
    /// This is the algebraic inverse of the Reinhard operator `c_ldr = c_hdr / (1 + c_hdr)`,
    /// scaled by the normalization factor `a / avr_lum`. Each channel is recovered as:
    ///
    /// ```text
    /// c_hdr = (avr_lum * c_ldr) / ((1 - c_ldr) * a)
    /// ```
    ///
    /// # Arguments
    /// * `factor_a` — The exposure normalization factor used during the forward tone mapping
    ///   (typically `0.18`). Must be positive; validation is delegated to the caller.
    /// * `avr_lum` — The log-average luminance of the original HDR scene, used to
    ///   undo the normalization step. Must be positive; validation is delegated to the caller.
    ///
    /// # Notes
    /// - Channel values must lie strictly in `(0, 1)` before this step. Values at
    ///   exactly `0` produce `0.0` (safe); values at exactly `1` produce a division
    ///   by zero (`inf`). In practice this is avoided by using `256.0` instead of
    ///   `255.0` in [`inverse_gamma_correction`], which keeps `c_ldr < 1`.
    /// - Validation of `factor_a` and `avr_lum` is the responsibility of the calling function.
    pub fn inverse_tone_mapping(&mut self, factor_a: f32, avr_lum: f32) {
        // validation of `a` and `avr_lum` must be done in ldr_to_hdr function.
        self.r = avr_lum * self.r / ((1.0 - self.r) * factor_a);
        self.g = avr_lum * self.g / ((1.0 - self.g) * factor_a);
        self.b = avr_lum * self.b / ((1.0 - self.b) * factor_a);
    }
}

// =================================================================
// Trait implementation
// =================================================================

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

/// Component-wise addition of two colors.
///
/// # Examples
/// ```rust
/// use rstrace::color::Color;
///
/// let first_color = Color::new(1.0, 2.0, 3.0);
/// let second_color = Color::new(10.0, 20.0, 30.0);
///
/// let sum = first_color + second_color;
///
/// assert_eq!(sum.r, 11.0);
/// ```
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
///
/// # Examples
/// ```rust
/// use rstrace::color::Color;
///
/// let first_color = Color::new(1.0, 2.0, 3.0);
/// let second_color = Color::new(10.0, 20.0, 30.0);
///
/// let mul = first_color * second_color;
///
/// assert_eq!(mul.r, 10.0);
/// assert_eq!(mul.g, 40.0);
/// assert_eq!(mul.b, 90.0)
/// ```
///
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
            panic!("Cannot divide `Color` by zero-valued!");
        }

        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

/// Adds `rhs` component-wise in place: `self.r += rhs.r`, etc.
impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

// =========================================================
// Predefined colors and palettes
// =========================================================

/// Predefined rainbow-like palette used for testing and debugging.
pub static RAINBOW_COLORS: [Color; 8] = [
    Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    }, // White
    Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    }, // Red
    Color {
        r: 1.0,
        g: 0.5,
        b: 0.0,
    }, // Orange
    Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
    }, // Yellow
    Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    }, // Green
    Color {
        r: 0.0,
        g: 0.5,
        b: 1.0,
    }, // Blue
    Color {
        r: 0.3,
        g: 0.0,
        b: 0.6,
    }, // Indigo
    Color {
        r: 0.6,
        g: 0.0,
        b: 0.6,
    }, // Violet
];

/// Pure black color `(0.0, 0.0, 0.0)`.
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};

/// Pure white color `(1.0, 1.0, 1.0)`.
pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
};

// =========================================================
// Tests
// =========================================================

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
    fn test_add_assign() {
        let mut color1 = Color::new(1.0, 2.0, 3.0);

        color1 += Color::new(4.0, 5.0, 6.0);
        let expected = Color::new(5.0, 7.0, 9.0);
        assert!(
            color1.is_close(&expected),
            "expected : {expected:?}\ncolor : {color1:?}"
        );
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
    #[should_panic(expected = "Cannot divide `Color` by zero-valued!")]
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
        assert!(color.tone_map().is_err());
        color.r = 1.0;
        color.tone_map().unwrap();
        assert_eq!(color.r, 1.0 / (1.0 + 1.0));
        assert_eq!(color.g, 2.5 / (2.5 + 1.0));
        assert_eq!(color.b, 5.0 / (5.0 + 1.0));
    }

    #[test]
    fn test_rescale() {
        let mut color = Color::new(1.0, 2.0, 4.0);
        let expected = Color::new(0.25, 0.4, 1.0);
        color.rescale().unwrap();
        assert!(color.is_close(&expected), "color: {:?}\nexpected: {:?}", color, expected);
    }

    #[test]
    fn test_rescale_null() {
        let mut color = Color::new(0.25, 0.4, 1.0);
        let expected = color;
        color.rescale().unwrap();
        assert!(color.is_close(&expected), "color: {:?}\nexpected: {:?}", color, expected);
    }

    #[test]
    fn test_inverse_gamma_correction() {
        let mut color = Color::new(1.0, 2.0, 3.0);
        let gamma = 0.5;
        let expected_color = Color {
            r: (1.0f32 / 256.0).powf(0.5),
            g: (2.0f32 / 256.0).powf(0.5),
            b: (3.0f32 / 256.0).powf(0.5),
        };
        color.inverse_gamma_correction(gamma);
        assert!(
            expected_color.is_close(&color),
            "color obtained: {:?}",
            color
        );
    }

    #[test]
    fn test_inverse_gamma_correction_infinity() {
        let mut color = Color::new(1.0, 2.0, 3.0);
        let gamma = f32::INFINITY;

        color.inverse_gamma_correction(gamma);
        let expected_color = Color::new(0.0, 0.0, 0.0);
        assert!(color.is_close(&expected_color));
    }

    #[test]
    fn test_inverse_tone_mapping() {
        let mut color = Color::new(0.5, 0.0, 0.2);
        let a: f32 = 0.1;
        let avr_lum: f32 = 10.0;
        let expected_color = Color {
            // 100 * 0.5/0.5
            r: 100.0,
            // 100 * 0 / 1
            g: 0.0,
            // 100 * 0.2 / 0.8
            b: 25.0,
        };
        color.inverse_tone_mapping(a, avr_lum);
        assert!(
            color.is_close(&expected_color),
            "inverse_tone_mapping result: {:?}",
            color
        );
    }
}
