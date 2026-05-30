//! In the raytracing engine, the material type is stored in this module.
//! It is composed of a Pigment, representing the texture of the material,
//! and the BRDF which handles how the lights are reflected by the material.
//!
//! In this raytracing project only reflectance is considered. No transparent materials.

use crate::brdf::{BRDF, DiffusiveBrdf};
use crate::color::BLACK;
use crate::pigments::{Pigment, UniformPigment};
use indicatif::style::ProgressTracker;

// ======================================================================
// Material struct
// ======================================================================

/// This struct stores the pigment and the light handling of a material.
#[derive(Clone)]
pub struct Material {
    pub pigment: Box<dyn Pigment>,
    pub brdf: Box<dyn BRDF>,
    pub emitted_radiance: Box<dyn Pigment>,
}
impl Material {
    pub fn new(
        pigment: impl Pigment + 'static,
        brdf: impl BRDF + 'static,
        emitted_radiance: impl Pigment + 'static,
    ) -> Self {
        Material {
            pigment: Box::new(pigment),
            brdf: Box::new(brdf),
            emitted_radiance: Box::new(emitted_radiance),
        }
    }
}
impl Default for Material {
    fn default() -> Self {
        Self {
            pigment: Box::new(UniformPigment::default()),
            brdf: Box::new(DiffusiveBrdf::default()),
            emitted_radiance: Box::new(UniformPigment::new(BLACK)),
        }
    }
}
