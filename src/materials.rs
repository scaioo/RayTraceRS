//! In the raytracing engine, the material type is stored in this module.
//! It is composed of a Pigment, representing the texture of the material,
//! and the BRDF which handles how the lights are reflected by the material.
//!
//! In this raytracing project only reflectance is considered. No transparent materials.

use crate::brdf::BRDF;
use crate::pigments::Pigment;

/// This struct stores the pigment and the light handling of a material.
pub struct Material<P, B> {
    pub pigment: P,
    pub brdf: B,
}

impl<P: Pigment, B: BRDF> Material<P, B> {
    fn new(pigment: P, brdf: B) -> Material<P, B> {
        Material { pigment, brdf }
    }
}
