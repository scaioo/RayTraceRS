//! Surface pigmentation system for the ray tracer.
//!
//! This module defines the [`Pigment`] trait and several implementations
//! used to describe how surface colors vary over a geometric object.
//!
//! A pigment maps 2D texture coordinates `(u,v)` into a [`Color`].
//!
//! Pigments may be:
//!
//! - uniform (single constant color)
//! - procedural (checkerboards, gradients, noise, etc.)
//! - image-based (texture mapping from HDR images)
//!
//! In the renderer architecture, pigments are responsible only for
//! determining the surface color. The reflection and scattering
//! behavior of light is instead handled by the BRDF system.
//!
//! # Available Pigments
//!
//! - [`UniformPigment`] : constant surface color
//! - [`CheckeredPigment`] : procedural checkerboard pattern
//! - [`ImagePigment`] : texture sampling from an HDR image
//! - [`GradientPigment`] : linear procedural gradient
//!
//! # Example
//!
//! ```rust
//! use rstrace::color::Color;
//! use rstrace::geometry::Vec2D;
//! use rstrace::pigments::{Pigment, UniformPigment};
//!
//! let pigment = UniformPigment::new(Color::new(1.0, 0.0, 0.0));
//!
//! let color = pigment.get_color(&Vec2D::new(0.5, 0.5)).unwrap();
//!
//! assert_eq!(color.r, 1.0);
//! ```
//!

use crate::color::{Color, WHITE};
use crate::geometry::Vec2D;
use crate::hdr_image::HDR;
use anyhow::Result;
// ===============================================
// Pigment type
// ==============================================

/// Describes the color distribution over a surface.
///
/// A `Pigment` maps UV texture coordinates to a [`Color`].
pub trait Pigment {
    /// Returns the `Color` of a certain point on the surface.
    fn get_color(&self, uv: &Vec2D) -> Result<Color>;
}

// ===============================================
// UniformPigment
// ==============================================

/// A pigment that returns the same color for every UV coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct UniformPigment {
    pub color: Color,
}
impl UniformPigment {
    /// Creates a pigment with a constant color.
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}
impl Default for UniformPigment {
    fn default() -> Self {
        Self::new(WHITE)
    }
}
impl Pigment for UniformPigment {
    fn get_color(&self, _uv: &Vec2D) -> Result<Color> {
        Ok(self.color)
    }
}

// ===============================================
// CheckeredPigment
// ==============================================
/// A procedural checkerboard pigment.
///
/// The UV domain `[0,1] × [0,1]` is subdivided into
/// `steps × steps` cells alternating between two colors.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckeredPigment {
    pub color1: Color,
    pub color2: Color,
    pub steps: u32,
}
impl CheckeredPigment {
    /// Creates a checkerboard pigment with the given colors and cell count.
    pub fn new(color1: Color, color2: Color, steps: u32) -> Self {
        CheckeredPigment {
            color1,
            color2,
            steps,
        }
    }
}
impl Pigment for CheckeredPigment {
    /// This function returns the color of a checkered surface for a given coordinate (u,v).
    ///
    /// # Warnings
    /// - If `CheckeredPigment` is initialized with `steps = 0` then the output color is always `color1`.
    /// - The border coordinate `1.0` for `u` or `v` returns the same result as `0.0`.
    /// - UV coordinates are assumed positive.
    fn get_color(&self, uv: &Vec2D) -> Result<Color> {
        let int_u = (uv.x * self.steps as f32).floor() as i32;
        let int_v = (uv.y * self.steps as f32).floor() as i32;

        if (int_u + int_v).rem_euclid(2) == 0 {
            Ok(self.color1)
        } else {
            Ok(self.color2)
        }
    }
}
// ===============================================
// ImagePigment
// ==============================================
/// A pigment defined by an HDR texture image.
///
/// Colors are sampled using bilinear interpolation
/// in normalized UV coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct ImagePigment {
    pub image: HDR,
}
impl ImagePigment {
    /// Creates an image pigment from an HDR texture.
    pub fn new(image: HDR) -> Self {
        Self { image }
    }
}
impl Pigment for ImagePigment {
    /// Returns the color of the texture at the given UV coordinates.
    ///
    /// Delegates to [`HDR::bilinear_interpolation`], which samples the image
    /// using bilinear interpolation between the four nearest pixels.
    ///
    /// # Errors
    /// Propagates error if the image contains no pixels.
    fn get_color(&self, uv: &Vec2D) -> Result<Color> {
        self.image.bilinear_interpolation(uv)
    }
}

// ===============================================
// Experimental procedural pigments
// ===============================================

/// A procedural linear gradient pigment.
///
/// The gradient interpolates linearly between `color1`
/// and `color2` along an axis rotated by `angle`.
///
/// The interpolation is unbounded, so colors may
/// extrapolate outside the `[color1, color2]` range.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientPigment {
    pub color1: Color,
    pub color2: Color,
    /// Rotation angle of the gradient axis, in radians.
    pub angle: f32,
}
impl GradientPigment {
    /// Creates a gradient pigment between two colors along a rotated axis.
    ///
    /// `angle` is expressed in radians and controls the direction of the
    /// gradient relative to the U axis.
    pub fn new(color1: Color, color2: Color, angle: f32) -> Self {
        Self {
            color1,
            color2,
            angle,
        }
    }
}
impl Pigment for GradientPigment {
    /// Returns a linear gradient along a rotated axis.
    ///
    /// The gradient is not clamped, so colors may extrapolate
    /// beyond `color1` and `color2`.
    fn get_color(&self, uv: &Vec2D) -> Result<Color> {
        let new_x = uv.x * self.angle.cos() + uv.y * self.angle.sin();
        Ok(self.color1 * (1.0 - new_x) + self.color2 * new_x)
    }
}

// **********************************************
// Tests
// **********************************************

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::RAINBOW_COLORS;
    use crate::pcg::PCG;

    // - - - - - - - - - - - - - - - - - - - - - -
    //              UniformPigment
    // - - - - - - - - - - - - - - - - - - - - - -
    #[test]
    fn test_uniform_pigment_constructor() {
        let color = Color::new(1.0, 2.0, 3.0);
        let pigment = UniformPigment::new(color);
        assert_eq!(color, pigment.color);
    }

    #[test]
    fn test_uniform_pigment_pigments() {
        let color = Color::new(1.0, 2.0, 3.0);
        let pigment = UniformPigment::new(color);
        assert_eq!(pigment.get_color(&Vec2D { x: 0.0, y: 0.0 }).unwrap(), color);
    }

    // - - - - - - - - - - - - - - - - - - - - - -
    //             CheckeredPigment
    // - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn test_checkered_pigment_constructor() {
        let color1 = Color::new(1.0, 2.0, 3.0);
        let color2 = Color::new(1.0, 2.0, 3.0);
        let pigment = CheckeredPigment::new(color1, color2, 3);
        assert_eq!(color1, pigment.color1);
        assert_eq!(color2, pigment.color2);
        assert_eq!(pigment.steps, 3);
    }

    #[test]
    fn test_checkered_pigment_get_color() {
        let green = Color::new(0.0, 1.0, 0.0);
        let blue = Color::new(0.0, 0.0, 1.0);
        let pigment = CheckeredPigment::new(green, blue, 3);

        //      (0,0)                                                   (1,0)
        //             * ----- * ----- * ----- * ----- * ----- * ----- *
        //             |       |       |       |       |       |       |
        //             * --- Green --- * --   Blue  -- * --- Green --- *
        //             |       |       |       |       |       |       |
        //             * ----- * ----- * ----- * ----- * ----- * ----- *
        //             |       |       |       |       |       |       |
        //             * --   Blue  -- * --- Green --- * --   Blue  -- *
        //             |       |       |       |       |       |       |
        //             * ----- * ----- * ----- * ----- * ----- * ----- *
        //             |       |       |       |       |       |       |
        //             * --- Green --- * --   Blue  -- * --- Green --- *
        //             |       |       |       |       |       |       |
        //             * ----- * ----- * ----- * ----- * ----- * ----- *
        //          (0,1)                                               (1,1)

        let expected_colors = vec![green, blue, green, blue, green, blue, green, blue, green];
        let mut expected = expected_colors.iter();

        // first row
        let u = 0.5 / 3.0;
        for i in 0..3 {
            let v = (i * 2 + 1) as f32 / 6.0;
            assert_eq!(
                &pigment.get_color(&Vec2D { x: u, y: v }).unwrap(),
                expected.next().unwrap()
            );
        }

        // second row
        let u = 0.5 / 3.0 + 1.0 / 3.0;
        for i in 0..3 {
            let v = (i * 2 + 1) as f32 / 6.0;
            assert_eq!(
                &pigment.get_color(&Vec2D { x: u, y: v }).unwrap(),
                expected.next().unwrap()
            );
        }

        // third row
        let u = 1.0 - 0.5 / 3.0;
        for i in 0..3 {
            let v = (i * 2 + 1) as f32 / 6.0;
            assert_eq!(
                &pigment.get_color(&Vec2D { x: u, y: v }).unwrap(),
                expected.next().unwrap()
            );
        }
    }

    #[test]
    fn test_checkered_pigment_get_color_zero_case() {
        let green = Color::new(0.0, 1.0, 0.0);
        let blue = Color::new(0.0, 0.0, 1.0);
        let pigment = CheckeredPigment::new(green, blue, 0);

        for _ in 0..10000 {
            let mut random_gen = PCG::new();
            let u = random_gen.random_float();
            let v = random_gen.random_float();

            assert_eq!(pigment.get_color(&Vec2D { x: u, y: v }).unwrap(), green);
        }
    }

    #[test]
    fn test_checkered_pigment_get_color_border_case() {
        let red = Color::new(3.0, 0.0, 0.0);
        let color = Color::new(0.0, 1.0, 2.0);
        let pigment = CheckeredPigment::new(red, color, 2);

        //   (0,0)                               (1,0)
        //      * ----- * ----- * ----- * ----- *
        //      |       |       |       |       |
        //      * ---- Red  --- * --- Color --- *
        //      |       |       |       |       |
        //      * ----- * ----- * ----- * ----- *
        //      |       |       |       |       |
        //      * --- Color --- * ---- Red  --- *
        //      |       |       |       |       |
        //      * ----- * ----- * ----- * ----- *
        //   (0,1)                               (1,1)

        // top left corner
        assert_eq!(pigment.get_color(&Vec2D { x: 0.0, y: 0.0 }).unwrap(), red);
        // top right corner
        assert_eq!(pigment.get_color(&Vec2D { x: 1.0, y: 1.0 }).unwrap(), red);
        // bottom left corner
        assert_eq!(pigment.get_color(&Vec2D { x: 0.0, y: 1.0 }).unwrap(), red);
        // bottom right corner
        assert_eq!(pigment.get_color(&Vec2D { x: 1.0, y: 0.0 }).unwrap(), red);
    }

    #[test]
    fn test_checkered_pigment_get_color_odd_steps() {
        let red = Color::new(1.0, 0.0, 0.0);
        let green = Color::new(0.0, 1.0, 0.0);
        let pigment = CheckeredPigment::new(red, green, 3);

        assert_eq!(
            pigment.get_color(&Vec2D { x: 1.1, y: 0.0 }).unwrap(),
            green,
            "Assert (1) failed"
        );
        assert_eq!(
            pigment.get_color(&Vec2D { x: 0.1, y: 1.1 }).unwrap(),
            green,
            "Assert (2) failed"
        );

        assert_eq!(
            pigment.get_color(&Vec2D { x: 1.1, y: 1.0 }).unwrap(),
            red,
            "Assert (3) failed"
        );
        assert_eq!(
            pigment.get_color(&Vec2D { x: 1.9, y: 1.9 }).unwrap(),
            red,
            "Assert (4) failed"
        );

        assert_eq!(
            pigment.get_color(&Vec2D { x: 0.5, y: 1.9 }).unwrap(),
            red,
            "Assert (5) failed"
        );
    }

    fn setup_test_rainbow() -> HDR {
        let mut img = HDR::new(4, 2);

        for (i, color) in RAINBOW_COLORS.iter().enumerate() {
            img.pixels[i] = *color;
        }

        img
    }

    #[test]
    fn test_image_pigments_constructor() {
        let image = setup_test_rainbow();
        let image_pigment = ImagePigment::new(image.clone());

        assert_eq!(image.width, image_pigment.image.width);
        assert_eq!(image.height, image_pigment.image.height);

        assert!(
            image
                .pixels
                .iter()
                .zip(image_pigment.image.pixels.iter())
                .all(|(a, b)| a.is_close(b))
        );
    }

    #[test]
    #[should_panic(expected = "No pigment image found")]
    fn test_image_pigment_constructor_fail() {
        let image = HDR::new(4, 0);
        let _ = ImagePigment::new(image);
    }

    #[test]
    fn test_image_pigments_get_color() {
        // basically the same test as in hdr_image ...
        let image = setup_test_rainbow();
        let image_pigment = ImagePigment::new(image.clone());

        let expected = Color {
            r: 0.5,
            g: 0.4,
            b: 0.5,
        };

        assert!(
            expected.is_close(
                &image_pigment
                    .image
                    .bilinear_interpolation(&Vec2D::new(0.2, 0.25))
                    .unwrap()
            )
        );

        let expected = Color {
            r: 0.74,
            g: 0.6,
            b: 0.34,
        };

        assert!(
            expected.is_close(
                &image_pigment
                    .image
                    .bilinear_interpolation(&Vec2D::new(0.8, 0.25))
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_gradient_pigments_constructor() {
        let color1 = Color::new(1.0, 2.0, 3.0);
        let color2 = Color::new(4.0, 5.0, 6.0);
        let angle = std::f32::consts::FRAC_PI_3;
        let gradient = GradientPigment::new(color1, color2, angle);

        assert_eq!(gradient.color1, color1);
        assert_eq!(gradient.color2, color2);
        assert_eq!(gradient.angle, angle);
    }

    fn setup_gradient() -> GradientPigment {
        let color1 = Color::new(1.0, 2.0, 3.0);
        let color2 = Color::new(4.0, 5.0, 6.0);
        let angle = std::f32::consts::FRAC_PI_3;
        GradientPigment::new(color1, color2, angle)
    }

    #[test]
    fn test_gradient_pigments_get_color() {
        let gradient = setup_gradient();

        assert_eq!(
            gradient.get_color(&Vec2D { x: 0.0, y: 0.0 }).unwrap(),
            gradient.color1,
            "Error in (0,0) check!"
        );
        assert_eq!(
            gradient
                .get_color(&Vec2D {
                    x: 0.5,
                    y: 3.0_f32.sqrt() / 2.0
                })
                .unwrap(),
            gradient.color2,
            "Error in new_x == 1 check!"
        );
        // for t = 0.5 we can compute the corresponding coordinates by:
        // u = l * cos(60) = 0.25,
        // v = l * sin(60) = sqrt(3) / 4.
        // The expected color is given by 0.5 * color1 + 0.5 * color2
        let mid_color = Color::new(2.5, 3.5, 4.5);
        assert_eq!(
            gradient
                .get_color(&Vec2D {
                    x: 0.25,
                    y: 3.0_f32.sqrt() / 4.0
                })
                .unwrap(),
            mid_color,
            "Error in mid-color check!"
        );
    }

    #[test]
    fn test_gradient_pigments_get_color_extrapolation() {
        let gradient = setup_gradient();
        let bottom_left_corner = Vec2D::new(0.9, 0.9);

        let expected_color = (1.0 - 1.2294228) * gradient.color1 + 1.2294228 * gradient.color2;

        assert!(expected_color.is_close(&gradient.get_color(&bottom_left_corner).unwrap()));
    }
}
