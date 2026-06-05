// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! # Materials
//!
//! This module defines [`Material`], the surface descriptor attached to every
//! shape in the scene.
//!
//! A material bundles three independent components:
//!
//! | Field              | Trait      | Role                                        |
//! |--------------------|------------|---------------------------------------------|
//! | `pigment`          | [`Pigment`]| Colour or texture at a surface point        |
//! | `brdf`             | [`BRDF`]   | Angular distribution of reflected light     |
//! | `emitted_radiance` | [`Pigment`]| Self-emission (black for non-emissive surfaces) |
//!
//! All components are stored as trait objects (`Box<dyn …>`), so any combination
//! of concrete pigment and BRDF types can be assembled at runtime without making
//! `Material` itself generic.

use crate::brdf::{BRDF, DiffusiveBrdf};
use crate::color::BLACK;
use crate::pigments::{Pigment, UniformPigment};

// ======================================================================
// Material struct
// ======================================================================

/// A complete surface description: texture, reflectance model, and self-emission.
///
/// `Material` is referenced by [`HitRecord`](crate::hit_record::HitRecord) after a
/// ray–shape intersection and consumed by the renderer to compute outgoing radiance.
///
/// # Cloning
///
/// `Material` derives [`Clone`]. This works because [`Box<dyn Pigment>`] and
/// [`Box<dyn BRDF>`] implement `Clone` via the `ClonePigment` / `CloneBrdf`
/// supertrait mechanism defined in their respective modules.
///
/// # Note on `emitted_radiance`
///
/// This field reuses the [`Pigment`] trait as a convenient UV-indexed colour
/// source. It does not represent the same physical quantity as `pigment`:
/// it is the spectral radiance emitted by the surface regardless of incoming
/// illumination. Use [`UniformPigment::new(BLACK)`] for non-luminous surfaces.

#[derive(Clone)]
pub struct Material {
    /// Colour or texture of the surface, evaluated at UV coordinates.
    pub pigment: Box<dyn Pigment>,
    /// Reflectance model: determines how incoming light scatters off the surface.
    pub brdf: Box<dyn BRDF>,
    /// Self-emitted radiance. Set to [`BLACK`] for non-luminous surfaces.
    pub emitted_radiance: Box<dyn Pigment>,
}
impl Material {
    /// Creates a new `Material` from concrete pigment, BRDF, and emission components.
    ///
    /// Each argument is accepted as an owned concrete type and boxed internally,
    /// so callers do not need to wrap values in `Box`.
    ///
    /// The `+ 'static` bound requires that concrete types contain no borrowed
    /// references. All standard pigment and BRDF types satisfy this automatically.
    ///
    /// # Example
    ///
    /// ```
    /// use rstrace::brdf::DiffusiveBrdf;
    /// use rstrace::pigments::UniformPigment;
    /// use rstrace::color::{BLACK, Color};
    /// use rstrace::materials::Material;
    ///
    /// let mat = Material::new(
    ///     UniformPigment::new(Color::new(0.8, 0.3, 0.3)),
    ///     DiffusiveBrdf::default(),
    ///     UniformPigment::new(BLACK), // non-emissive
    /// );
    /// ```
    ///
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
    /// Returns a default matte white material with no self-emission.
    ///
    /// Equivalent to:
    /// ```rust
    /// use rstrace::brdf::DiffusiveBrdf;
    /// use rstrace::pigments::UniformPigment;
    /// use rstrace::color::BLACK;
    /// use rstrace::materials::Material;
    ///
    /// Material::new(
    ///     UniformPigment::default(), // white
    ///     DiffusiveBrdf::default(),
    ///     UniformPigment::new(BLACK),
    /// );
    /// ```
    fn default() -> Self {
        Self {
            pigment: Box::new(UniformPigment::default()),
            brdf: Box::new(DiffusiveBrdf::default()),
            emitted_radiance: Box::new(UniformPigment::new(BLACK)),
        }
    }
}
