//! todo: this is just a draft!!
//!
//! This module contains the ray tracer description of a surface color.
//!
//! The material is described by `Pigment` and `BRDF`. The first gives the color and the second one
//! returns the reflected rays. This module focuses on the `Pigment`.

use crate::color::Color;
use crate::geometry::Vec2D;

// ===============================================
// Pigment type
// ==============================================

/// This is the marker trait for `Pigment` types.
pub trait Pigment {
    /// Returns the `Color` of a certain point on the surface.
    fn get_color(&self, uv: &Vec2D) -> Color;
}

// ===============================================
// UniformPigment
// ==============================================

/// The surface has only one color!
pub struct UniformPigment {
    pub color: Color,
}

impl UniformPigment {
    pub fn new(color: Color) -> Self {
        UniformPigment { color }
    }
}

impl Pigment for UniformPigment {
    fn get_color(&self, _uv: &Vec2D) -> Color {
        self.color
    }
}

// ===============================================
// CheckeredPigment
// ==============================================
/// The pigment is a checkered surface
pub struct CheckeredPigment {
    pub color1: Color,
    pub color2: Color,
    pub steps: u32,
}

impl CheckeredPigment {
    pub fn new(color1: Color, color2: Color, steps: u32) -> Self {
        CheckeredPigment {
            color1,
            color2,
            steps,
        }
    }
}

impl Pigment for CheckeredPigment {
    // Note: the function returns only one color if steps = 0.
    fn get_color(&self, uv: &Vec2D) -> Color {
        let int_u = (uv.x * self.steps as f32).floor() as u32;
        let int_v = (uv.y * self.steps as f32).floor() as u32;

        if int_u % 2 == int_v % 2 {
            self.color1
        } else {
            self.color2
        }
    }
}

// **********************************************
// Tests
// **********************************************

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(pigment.get_color(&Vec2D { x: 0.0, y: 0.0 }), color);
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

        //      (0,0)                                                   (0,1)
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
        //          (1,0)                                               (1,1)

        let expected_colors = vec![green, blue, green, blue, green, blue, green, blue, green];
        let mut expected = expected_colors.iter();

        // first row
        let u = 0.5 / 3.0;
        for i in 0..3 {
            let v = (i * 2 + 1) as f32 / 6.0;
            assert_eq!(
                &pigment.get_color(&Vec2D { x: u, y: v }),
                expected.next().unwrap()
            );
        }

        // second row
        let u = 0.5 / 3.0 + 1.0 / 3.0;
        for i in 0..3 {
            let v = (i * 2 + 1) as f32 / 6.0;
            assert_eq!(
                &pigment.get_color(&Vec2D { x: u, y: v }),
                expected.next().unwrap()
            );
        }

        // third row
        let u = 1.0 - 0.5 / 3.0;
        for i in 0..3 {
            let v = (i * 2 + 1) as f32 / 6.0;
            assert_eq!(
                &pigment.get_color(&Vec2D { x: u, y: v }),
                expected.next().unwrap()
            );
        }
    }
}
