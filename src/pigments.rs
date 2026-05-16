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
pub struct UniformPigment{
    pub color : Color,
}

impl UniformPigment {
    pub fn new(color : Color) -> Self {
        UniformPigment { color }
    }
}

impl Pigment for UniformPigment {
    fn get_color(&self, _uv: &Vec2D) -> Color {
        self.color
    }
}







#[cfg(test)]
mod tests {
    use super::*;

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
}

