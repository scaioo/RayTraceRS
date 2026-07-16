// This file is licensed under the EUPL-1.2. See LICENSE.md.

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
use anyhow::{Result, anyhow};

// ===============================================
// Pigment Cloning supertrait
// ==============================================
/// Helper supertrait that makes `Box<dyn Pigment>` cloneable.
///
/// You never need to implement this manually. The blanket `impl` below
/// provides it automatically for any type that implements `Pigment + Clone`.
pub trait ClonePigment {
    /// Clones `self` into a new boxed [`Pigment`] trait object.
    fn clone_pigment(&self) -> Box<dyn Pigment>;
}

/// Blanket implementation: any `T: Pigment + Clone + 'static` gets
/// [`ClonePigment`] for free by boxing a normal `.clone()` call.
impl<T> ClonePigment for T
where
    T: Pigment + Clone + 'static,
{
    fn clone_pigment(&self) -> Box<dyn Pigment> {
        Box::new(self.clone())
    }
}

// ===============================================
// Pigment type
// ==============================================

/// Describes the color distribution over a surface.
///
/// A `Pigment` maps UV texture coordinates to a [`Color`].
pub trait Pigment: ClonePigment + Send + Sync {
    /// Returns the `Color` of a certain point on the surface.
    fn get_color(&self, uv: &Vec2D) -> Result<Color>;

    /// Checks that every color this pigment can produce is a physically
    /// valid reflectance, i.e. within `[0,1]` per channel (see
    /// [`Color::validate_reflectance`]).
    ///
    /// This is a one-time, whole-pigment check — not run per-sample during
    /// rendering. Implementations only need to check the finite set of
    /// colors they store, not every possible `get_color` output; this is
    /// only sound for pigments that interpolate as a convex combination of
    /// stored colors (uniform, checkered, bilinear image sampling, and
    /// [`GradientPigment`] once its projection is normalized to `[0,1]`
    /// over the unit square), since a convex combination of valid colors is
    /// itself always valid.
    ///
    /// # Errors
    /// Returns an error describing which stored color(s) are out of range.
    fn validate_reflectance(&self) -> Result<()>;
}

impl Clone for Box<dyn Pigment> {
    fn clone(&self) -> Box<dyn Pigment> {
        self.clone_pigment()
    }
}

// ===============================================
// UniformPigment
// ==============================================

/// A pigment that returns the same color for every UV coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct UniformPigment {
    /// The constant color returned for every UV coordinate.
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

    /// Checks the single stored color, since it's the only color this
    /// pigment can ever produce.
    fn validate_reflectance(&self) -> Result<()> {
        if self.color.validate_reflectance() {
            Ok(())
        } else {
            Err(anyhow!(
                "UniformPigment has invalid reflection: {:?}",
                self.color
            ))
        }
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
    /// Color of the even cells.
    pub color1: Color,
    /// Color of the odd cells.
    pub color2: Color,
    /// Number of cells per axis over the UV domain.
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

    /// Checks both stored colors: `get_color` only ever returns `color1` or
    /// `color2` verbatim, never a blend, so checking these two is exhaustive.
    fn validate_reflectance(&self) -> Result<()> {
        if self.color1.validate_reflectance() && self.color2.validate_reflectance() {
            Ok(())
        } else {
            Err(anyhow!(
                "CheckeredPigment has invalid reflection:\n color1 {:?}, color2 {:?}",
                self.color1,
                self.color2
            ))
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
    /// The HDR texture sampled by this pigment.
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

    /// Checks every stored pixel. Since [`get_color`](Pigment::get_color)
    /// bilinearly interpolates between the four nearest pixels — a convex
    /// combination — the sampled color can never exceed the range of the
    /// pixels it's blended from, so a full pixel scan is exhaustive without
    /// having to sample the whole UV domain.
    fn validate_reflectance(&self) -> Result<()> {
        for pixel in &self.image.pixels {
            if !pixel.validate_reflectance() {
                return Err(anyhow!("ImagePigment has invalid reflection: {:?}", pixel));
            }
        }
        Ok(())
    }
}

// ===============================================
// Procedural pigments
// ===============================================

/// A procedural linear gradient pigment.
///
/// The gradient interpolates linearly between `color1`
/// and `color2` along an axis rotated by `angle`.
///
/// The projection is normalized against the four corners of the unit
/// square, so for `uv` within `[0,1] x [0,1]` the interpolation is
/// bounded: whichever corner has the smallest projection is always
/// exactly `color1` and whichever has the largest is always exactly
/// `color2`, regardless of `angle`. Outside the unit square, colors
/// saturate to the nearest endpoint (`color1` or `color2`).
#[derive(Clone, Debug, PartialEq)]
pub struct GradientPigment {
    /// Color at the start of the gradient (smallest projection).
    pub color1: Color,
    /// Color at the end of the gradient (largest projection).
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
    /// The projection of `uv` onto the gradient axis is rescaled so that
    /// the unit square's extreme corners map exactly to `color1` and
    /// `color2`. The parameter `t` is clamped to `[0,1]`, so `uv` outside
    /// `[0,1] x [0,1]` saturates to the endpoints instead of extrapolating
    /// past them (which could produce colors with negative channels).
    fn get_color(&self, uv: &Vec2D) -> Result<Color> {
        let (c, s) = (self.angle.cos(), self.angle.sin());
        let t_min = [0.0f32, c, s, c + s]
            .into_iter()
            .fold(f32::INFINITY, f32::min);
        let t_max = [0.0f32, c, s, c + s]
            .into_iter()
            .fold(f32::NEG_INFINITY, f32::max);
        let t = ((uv.x * c + uv.y * s - t_min) / (t_max - t_min)).clamp(0.0, 1.0);
        Ok(self.color1 * (1.0 - t) + self.color2 * t)
    }

    /// Checks both endpoint colors. This is exhaustive because `get_color`
    /// clamps `t` to `[0,1]`, so every output is a convex combination of
    /// `color1` and `color2` for any `uv`: if both endpoints are valid,
    /// every output is too.
    fn validate_reflectance(&self) -> Result<()> {
        if !self.color1.validate_reflectance() || !self.color2.validate_reflectance() {
            Err(anyhow!(
                "GradientPigment has invalid reflection: \ncolor1 {:?}\n color2 {:?}",
                self.color1,
                self.color2
            ))
        } else {
            Ok(())
        }
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

    #[test]
    fn test_uniform_pigment_validate_reflectance() {
        let color = Color::new(1.0, 0.0, 1.9 / 7.2);
        let pigment = UniformPigment::new(color);
        assert!(pigment.validate_reflectance().is_ok());
    }

    #[test]
    fn test_uniform_pigment_validate_reflectance_err() {
        let color = Color::new(-1.0, 2.0, 3.0);
        let pigment = UniformPigment::new(color);
        assert!(pigment.validate_reflectance().is_err());
    }

    #[test]
    fn test_uniform_pigment_negative_uv() {
        let color = Color::new(1.0, 2.0, 3.0);
        let pigment = UniformPigment::new(color);
        let result = pigment.get_color(&Vec2D { x: -0.1, y: 0.0 }).unwrap();

        assert!(
            color.is_close(&result),
            "expected: {:?}\nfound: {:?}",
            color,
            result
        );
    }

    #[test]
    fn test_uniform_pigment_out_of_bound_uv() {
        let color = Color::new(1.0, 2.0, 3.0);
        let pigment = UniformPigment::new(color);
        let result = pigment.get_color(&Vec2D { x: 0.1, y: 10.0 }).unwrap();

        assert!(
            color.is_close(&result),
            "expected: {:?}\nfound: {:?}",
            color,
            result
        );
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
        let mut random_gen = PCG::default();

        for _ in 0..10000 {
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

    #[test]
    fn test_checkered_pigment_get_color_round_to_one() {
        let red = Color::new(1.0, 0.0, 0.0);
        let green = Color::new(0.0, 1.0, 0.0);
        let pigment = CheckeredPigment::new(red, green, 2);
        let uv = Vec2D {
            x: 0.4,
            y: 1.0 - 1e-9,
        };

        let color = pigment.get_color(&uv).unwrap();

        assert!(
            color.is_close(&red),
            "expected: {:?}\nactual: {:?}",
            color,
            red
        );
    }

    #[test]
    fn test_checkered_pigment_get_color_big_negative_uv() {
        let red = Color::new(1.0, 0.0, 0.0);
        let green = Color::new(0.0, 1.0, 0.0);
        let pigment = CheckeredPigment::new(red, green, 2);
        let uv = Vec2D { x: -0.9, y: 0.05 };

        let color = pigment.get_color(&uv).unwrap();

        assert!(
            color.is_close(&red),
            "expected: {:?}\nactual: {:?}",
            color,
            red
        );
    }

    #[test]
    fn test_checkered_pigment_validate_reflectance() {
        let red = Color::new(1.0, 0.5, 0.0);
        let green = Color::new(0.01, 1.0, 0.33);
        let pigment = CheckeredPigment::new(red, green, 3);
        assert!(pigment.validate_reflectance().is_ok());
    }

    #[test]
    fn test_checkered_pigment_validate_reflectance_fail_high() {
        let red = Color::new(1.0, 0.5, 0.0);
        let green = Color::new(0.01, 1.01, 0.33);
        let pigment = CheckeredPigment::new(red, green, 3);
        assert!(pigment.validate_reflectance().is_err());
    }

    #[test]
    fn test_checkered_pigment_validate_reflectance_fail_low() {
        let red = Color::new(1.0, 0.5, 0.0);
        let green = Color::new(-0.01, 1.00, 0.33);
        let pigment = CheckeredPigment::new(red, green, 3);
        assert!(pigment.validate_reflectance().is_err());
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
    fn test_image_pigments_get_color_rounded_to_one() {
        let pigment = ImagePigment::new(setup_test_rainbow());
        let result = pigment.get_color(&Vec2D::new(0.2, 1.0 - 1e-9)).unwrap();
        let expected = pigment.get_color(&Vec2D::new(0.2, 0.0)).unwrap();

        assert!(
            expected.is_close(&result),
            "expected {:?}, got {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_image_pigments_get_color_big_negative() {
        let pigment = ImagePigment::new(setup_test_rainbow());
        let result = pigment.get_color(&Vec2D::new(0.2, -0.9)).unwrap();
        let expected = pigment.get_color(&Vec2D::new(0.2, 0.1)).unwrap();

        assert!(
            expected.is_close(&result),
            "expected {:?}, got {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_image_pigments_get_color_rounded_to_one_u_coordinate() {
        let pigment = ImagePigment::new(setup_test_rainbow());
        let result = pigment.get_color(&Vec2D::new(1.0 - 1e-9, 0.6)).unwrap();
        let expected = pigment.get_color(&Vec2D::new(0.0, 0.6)).unwrap();

        assert!(
            expected.is_close(&result),
            "expected {:?}, got {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_image_pigments_get_color_small_negative() {
        let pigment = ImagePigment::new(setup_test_rainbow());
        let result = pigment.get_color(&Vec2D::new(0.2, -1e-9)).unwrap();
        let expected = pigment.get_color(&Vec2D::new(0.2, 0.0)).unwrap();

        assert!(
            expected.is_close(&result),
            "expected {:?}, got {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_image_pigments_get_color_small_negative_u() {
        let pigment = ImagePigment::new(setup_test_rainbow());
        let result = pigment.get_color(&Vec2D::new(-1e-9, 0.6)).unwrap();
        let expected = pigment.get_color(&Vec2D::new(0.0, 0.6)).unwrap();

        assert!(
            expected.is_close(&result),
            "expected {:?}, got {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_image_pigments_validate_reflection() {
        let image = setup_test_rainbow();
        let image_pigment = ImagePigment::new(image);
        assert!(image_pigment.validate_reflectance().is_ok());
    }

    #[test]
    fn test_image_pigments_validate_reflection_fail_low() {
        let color = Color::new(-1.0, 0.0, 0.0);
        let mut image = HDR::new(1, 1);
        image.set_pixel(0, 0, color).unwrap();
        let image_pigment = ImagePigment::new(image);
        assert!(image_pigment.validate_reflectance().is_err());
    }

    #[test]
    fn test_image_pigments_validate_reflection_fail_high() {
        let color = Color::new(1.0001, 0.0, 0.0);
        let mut image = HDR::new(1, 1);
        image.set_pixel(0, 0, color).unwrap();
        let image_pigment = ImagePigment::new(image);
        assert!(image_pigment.validate_reflectance().is_err());
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
        // angle = 60 deg: both cos and sin are positive, so the extreme
        // corners of the unit square are (0,0) -> color1 and (1,1) -> color2,
        // same as the un-rotated case.
        let gradient = setup_gradient();

        assert_eq!(
            gradient.get_color(&Vec2D { x: 0.0, y: 0.0 }).unwrap(),
            gradient.color1,
            "Error in (0,0) check!"
        );
        assert_eq!(
            gradient.get_color(&Vec2D { x: 1.0, y: 1.0 }).unwrap(),
            gradient.color2,
            "Error in (1,1) check!"
        );
        // (0.5, 0.5) lies halfway between the two extreme corners along the
        // gradient axis, so it must be the exact average of color1 and color2.
        let mid_color = Color::new(2.5, 3.5, 4.5);
        assert_eq!(
            gradient.get_color(&Vec2D { x: 0.5, y: 0.5 }).unwrap(),
            mid_color,
            "Error in mid-color check!"
        );
    }

    #[test]
    fn test_gradient_pigments_get_color_horizontal() {
        // angle = 0: gradient runs purely along x, independent of y.
        let gradient =
            GradientPigment::new(Color::new(1.0, 2.0, 3.0), Color::new(4.0, 5.0, 6.0), 0.0);

        assert!(
            gradient
                .color1
                .is_close(&gradient.get_color(&Vec2D::new(0.0, 0.0)).unwrap())
        );
        assert!(
            gradient
                .color1
                .is_close(&gradient.get_color(&Vec2D::new(0.0, 0.7)).unwrap())
        );
        assert!(
            gradient
                .color2
                .is_close(&gradient.get_color(&Vec2D::new(1.0, 0.0)).unwrap())
        );
        assert!(
            gradient
                .color2
                .is_close(&gradient.get_color(&Vec2D::new(1.0, 1.0)).unwrap())
        );

        let mid_color = Color::new(2.5, 3.5, 4.5);
        assert!(mid_color.is_close(&gradient.get_color(&Vec2D::new(0.5, 0.3)).unwrap()));
    }

    #[test]
    fn test_gradient_pigments_get_color_vertical() {
        // angle = 90 deg: gradient runs purely along y, independent of x.
        let gradient = GradientPigment::new(
            Color::new(1.0, 2.0, 3.0),
            Color::new(4.0, 5.0, 6.0),
            std::f32::consts::FRAC_PI_2,
        );

        assert!(
            gradient
                .color1
                .is_close(&gradient.get_color(&Vec2D::new(0.0, 0.0)).unwrap())
        );
        assert!(
            gradient
                .color1
                .is_close(&gradient.get_color(&Vec2D::new(0.6, 0.0)).unwrap())
        );
        assert!(
            gradient
                .color2
                .is_close(&gradient.get_color(&Vec2D::new(0.0, 1.0)).unwrap())
        );
        assert!(
            gradient
                .color2
                .is_close(&gradient.get_color(&Vec2D::new(1.0, 1.0)).unwrap())
        );

        let mid_color = Color::new(2.5, 3.5, 4.5);
        assert!(mid_color.is_close(&gradient.get_color(&Vec2D::new(0.4, 0.5)).unwrap()));
    }

    #[test]
    fn test_gradient_pigments_get_color_negative_angle_off_diagonal_extremes() {
        // angle = -45 deg: cos > 0, sin < 0, so the extreme corners are the
        // *off-diagonal* pair (1,0) -> color2 and (0,1) -> color1, while the
        // main diagonal corners (0,0) and (1,1) both land exactly on the midpoint.
        let color1 = Color::new(1.0, 2.0, 3.0);
        let color2 = Color::new(4.0, 5.0, 6.0);
        let gradient = GradientPigment::new(color1, color2, -std::f32::consts::FRAC_PI_4);

        assert!(color1.is_close(&gradient.get_color(&Vec2D::new(0.0, 1.0)).unwrap()));
        assert!(color2.is_close(&gradient.get_color(&Vec2D::new(1.0, 0.0)).unwrap()));

        let mid_color = Color::new(2.5, 3.5, 4.5);
        assert!(mid_color.is_close(&gradient.get_color(&Vec2D::new(0.0, 0.0)).unwrap()));
        assert!(mid_color.is_close(&gradient.get_color(&Vec2D::new(1.0, 1.0)).unwrap()));
    }

    #[test]
    fn test_gradient_pigments_get_color_saturation_plateau() {
        let gradient = setup_gradient();
        let result = gradient.get_color(&Vec2D::new(1.5, 1.5)).unwrap();
        let expected = gradient.get_color(&Vec2D::new(2.0, 2.0)).unwrap();

        assert!(
            expected.is_close(&result),
            "expected: {:?}\n found: {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_gradient_pigments_get_color_saturates_below() {
        let gradient = setup_gradient();
        let uv = Vec2D::new(-0.2, -0.2);
        let result = gradient.get_color(&uv).unwrap();

        assert!(
            gradient.color1.is_close(&result),
            "expected: {:?}\n found: {:?}",
            gradient.color1,
            result
        );
    }

    #[test]
    fn test_gradient_pigments_get_color_saturates_vertical() {
        let gradient = GradientPigment::new(
            Color::new(1.0, 2.0, 3.0),
            Color::new(4.0, 5.0, 6.0),
            std::f32::consts::FRAC_PI_2,
        );

        assert!(
            gradient
                .color2
                .is_close(&gradient.get_color(&Vec2D::new(0.3, 1.5)).unwrap())
        );
        assert!(
            gradient
                .color1
                .is_close(&gradient.get_color(&Vec2D::new(0.3, -0.5)).unwrap())
        );
    }

    #[test]
    fn test_gradient_pigment_get_color_negative_uv_small() {
        let gradient = setup_gradient();
        let color = gradient.get_color(&Vec2D { x: -1e-9, y: 0.101 }).unwrap();
        let expected_color = gradient.get_color(&Vec2D { x: 0.0, y: 0.101 }).unwrap();

        assert!(
            expected_color.is_close(&color),
            "expected: {:?}\nfound: {:?}",
            expected_color,
            color
        );
    }

    #[test]
    fn test_gradient_pigment_get_color_saturation_angle() {
        let gradient = GradientPigment::new(
            Color::new(0.0, 0.7, 0.0),
            Color::new(0.0, 0.0, 0.5),
            std::f32::consts::FRAC_PI_3,
        );
        let color = gradient.get_color(&Vec2D::new(1.5, 1.5)).unwrap();

        assert!(
            color.r >= 0.0 && color.g >= 0.0 && color.b >= 0.0,
            "gradient produced a negative channel: {:?}",
            color
        );
    }

    #[test]
    fn test_gradient_pigments_validate_reflection() {
        let color1 = Color::new(1.0, 0.1, 0.3);
        let color2 = Color::new(0.11, 0.31, 0.63);
        let pigment = GradientPigment::new(color1, color2, std::f32::consts::FRAC_PI_3);
        assert!(pigment.validate_reflectance().is_ok());
    }

    #[test]
    fn test_gradient_pigments_validate_reflection_fail_high() {
        let color1 = Color::new(1.01, 0.1, 0.3);
        let color2 = Color::new(0.11, 0.31, 0.63);
        let pigment = GradientPigment::new(color1, color2, std::f32::consts::PI);
        assert!(pigment.validate_reflectance().is_err());
    }

    #[test]
    fn test_gradient_pigments_validate_reflection_fail_low() {
        let color1 = Color::new(1.0, 0.0, 0.0);
        let color2 = Color::new(-0.1, 0.0, 0.0);
        let pigment = GradientPigment::new(color1, color2, 0.0);
        assert!(pigment.validate_reflectance().is_err());
    }
}
