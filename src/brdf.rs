// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! # BRDF — Bidirectional Reflectance Distribution Functions
//!
//! This module defines the [`BRDF`] trait and its implementations:
//!
//! - [`DiffusiveBrdf`] — perfectly diffuse scattering over the
//!   cosine-weighted hemisphere above the surface normal.
//! - [`SpecularBrdf`] — perfect mirror reflection.
//!
//! Both are used by the renderer to determine how a ray continues after
//! hitting a surface at the interaction point.
use crate::geometry::{Dot, Normal, Point, Vector, branchless_onb};
use crate::pcg::PCG;
use crate::ray::Ray;

// ======================================================================
// Cloning supertraits
// ======================================================================
/// Helper supertrait that makes `Box<dyn BRDF>` cloneable.
///
/// You never need to implement this manually. The blanket `impl` below
/// provides it automatically for any type that implements `BRDF + Clone`.
pub trait CloneBrdf {
    fn clone_brdf(&self) -> Box<dyn BRDF>;
}

/// Blanket implementation: any `T: BRDF + Clone + 'static` gets
/// [`CloneBrdf`] for free by boxing a normal `.clone()` call.
impl<T> CloneBrdf for T
where
    T: BRDF + Clone + 'static,
{
    fn clone_brdf(&self) -> Box<dyn BRDF> {
        Box::new(self.clone())
    }
}

// ======================================================================
// BRDF trait
// ======================================================================

/// Bidirectional Reflectance Distribution Function.
///
/// Implementors describe how a surface scatters an incoming ray into an
/// outgoing ray
pub trait BRDF: CloneBrdf + Send + Sync {
    /// Computes the scattered [`Ray`] produced when `incoming_dir` hits a
    /// surface at `interacting_point`.
    ///
    /// # Arguments
    ///
    /// * `pcg`               — mutable RNG used for stochastic sampling.
    /// * `incoming_dir`      — direction of the ray arriving at the surface.
    /// * `interacting_point` — world-space point of intersection.
    /// * `normal`            — surface normal at the interaction point
    /// * `depth`             — current ray recursion depth, forwarded to the new [`Ray`] unchanged.
    fn scatter_ray(
        &self,
        pcg: &mut PCG,
        incoming_dir: Vector,
        interacting_point: Point,
        normal: Normal,
        depth: usize,
    ) -> Ray;
}

impl Clone for Box<dyn BRDF> {
    fn clone(&self) -> Box<dyn BRDF> {
        self.clone_brdf()
    }
}

/// A lightweight tag enum enumerating the available BRDF variants.
///
/// Useful for serialisation, scene-description parsing, or any context where
/// a concrete BRDF type must be named without holding a `Box<dyn BRDF>`.
/// Convert to a real BRDF by constructing the corresponding struct
/// ([`DiffusiveBrdf`] or [`SpecularBrdf`]) directly.
pub enum BRDFs {
    /// Perfectly diffuse (Lambertian) reflectance. See [`DiffusiveBrdf`].
    Diffuse,
    /// Perfect mirror reflectance. See [`SpecularBrdf`].
    Specular,
}

// ======================================================
// DiffusiveBrdf
// ======================================================

/// A perfectly diffuse material.
///
/// Scatters incoming rays with a cosine-weighted distribution over the
/// hemisphere above the surface normal, independent of the incoming direction.
/// This models rough, matter surfaces such as chalk or unfinished plaster.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct DiffusiveBrdf {}
impl BRDF for DiffusiveBrdf {
    /// Scatters the ray in a random direction in the hemisphere above `normal`.
    ///
    /// The resulting direction always satisfies `dir · normal > 0`.
    fn scatter_ray(
        &self,
        pcg: &mut PCG,
        _incoming_dir: Vector,
        interacting_point: Point,
        normal: Normal,
        depth: usize,
    ) -> Ray {
        let (e1, e2, e3) = branchless_onb(normal);
        let cos_theta_sq = pcg.random_float();
        let cos_theta = cos_theta_sq.sqrt();
        let sin_theta = (1.0 - cos_theta_sq).sqrt();
        let phi = 2.0 * std::f32::consts::PI * pcg.random_float();
        let dir = e1 * phi.cos() * sin_theta + e2 * phi.sin() * sin_theta + e3 * cos_theta;

        Ray {
            origin: interacting_point,
            dir,
            t_max: f32::INFINITY,
            t_min: 1e-3,
            depth,
        }
    }
}

// ======================================================
// SpecularBrdf
// ======================================================

/// A perfectly specular material.
///
/// Reflects incoming rays about the surface normal with no scattering.
/// Models polished metals, mirrors, and other near-perfect reflectors.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpecularBrdf {}
impl BRDF for SpecularBrdf {
    /// Reflects `incoming_dir` about `normal` using the mirror reflection law.
    ///
    /// The reflected direction is `d - 2(d · n̂)n̂`, where `d` and `n̂` are
    /// the normalised incoming direction and surface normal respectively.
    /// The result is deterministic — `pcg` is not used.
    fn scatter_ray(
        &self,
        _pcg: &mut PCG,
        incoming_dir: Vector,
        interacting_point: Point,
        normal: Normal,
        depth: usize,
    ) -> Ray {
        let ray_dir = Vector::new(incoming_dir.x, incoming_dir.y, incoming_dir.z).normalize();
        let normal = Vector::from(normal).normalize();
        let dot_product = ray_dir.dot(&normal);

        Ray {
            origin: interacting_point,
            dir: ray_dir - normal * 2.0 * dot_product,
            t_min: 1e-5,
            t_max: f32::INFINITY,
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::are_close;

    fn scatter_inputs() -> (PCG, Vector, Point, Normal) {
        let incoming_dir = Vector::new(0.0, -1.0, -1.0);
        let interacting_point = Point::new(0.0, 0.0, 0.0);
        let normal = Normal::new(0.0, 0.0, 1.0);
        (PCG::default(), incoming_dir, interacting_point, normal)
    }

    #[test]
    fn test_diffusive_brdf_hemisphere() {
        let diffusive_brdf = DiffusiveBrdf {};
        let (mut pcg, incoming_dir, interacting_point, normal) = scatter_inputs();

        for _ in 0..1000 {
            let ray =
                diffusive_brdf.scatter_ray(&mut pcg, incoming_dir, interacting_point, normal, 2);
            assert!(
                ray.dir.dot(&normal) > 0.0,
                "ray.dir.dot(&normal) < 0!\nray.dir: {}",
                ray.dir
            );
        }
    }

    #[test]
    fn test_diffusive_brdf_z_check() {
        let diffusive_brdf = DiffusiveBrdf {};
        let (mut pcg, incoming_dir, interacting_point, normal) = scatter_inputs();

        for _ in 0..1000 {
            let ray =
                diffusive_brdf.scatter_ray(&mut pcg, incoming_dir, interacting_point, normal, 2);
            assert!(
                ray.dir.z < 1.0 || are_close(1.0, ray.dir.z),
                "Diffused ray has inconsistent z coordinate!\nray.dir: {}",
                ray.dir
            );
        }
    }

    #[test]
    fn test_specular_brdf() {
        let specular_brdf = SpecularBrdf {};
        let (_, incoming_dir, interacting_point, normal) = scatter_inputs();

        let expected_dir = Vector::new(0.0, -1.0, 1.0).normalize();

        let result = specular_brdf.scatter_ray(
            &mut PCG::default(),
            incoming_dir,
            interacting_point,
            normal,
            5,
        );

        assert_eq!(result.dir, expected_dir);
    }
}
