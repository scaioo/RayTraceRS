//! todo: this is just a draft!!
//!
//! This module contains the ray tracer description of a surface color.
//!
//! The material is described by `Pigment` and `BRDF`. The first gives the color and the second one
//! returns the reflected rays. This module focuses on the `Pigment`.

use crate::color::Color;
use crate::geometry::Vec2D;
use crate::ray;


// ===============================================
// Pigment type
// ==============================================

/// This is the marker trait for `Pigment` types. 
pub trait Pigment {
    /// Returns the `Color` of a certain point on the surface.
    fn get_color(&self, uv: &Vec2D) -> Color;
}



