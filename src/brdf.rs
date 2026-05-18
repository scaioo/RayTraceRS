//! The BRDF encodes the Bidirectional Reflectance Distribution Function of the Rendering Equation.
//!

use crate::geometry::{Dot, Normal, Point, Vec2D, Vector, branchless_onb};
use crate::pcg::PCG;
use crate::ray::Ray;

/// This trait collects the various BRDF types.
pub trait BRDF {
    fn scatter_ray(
        &self,
        pcg: PCG,
        incoming_dir: Vector,
        interacting_point: Point,
        normal: Normal,
        depth: usize,
    ) -> Ray;
}

// ======================================================
// DiffusiveBrdf
// ======================================================

/// DiffusiveBrdf is used for the material that emit light rays for 2π hemisphere.
pub struct DiffusiveBrdf {}
impl BRDF for DiffusiveBrdf {
    /// This function returns a random direction ray scattered from a point in the surface.
    fn scatter_ray(
        &self,
        mut pcg: PCG,
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
            t_max: std::f32::INFINITY,
            t_min: 1e-3,
            depth,
        }
    }
}

// ======================================================
// DiffusiveBrdf
// ======================================================

/// SpecularBrdf represent the totally reflective materials.
pub struct SpecularBrdf {}

impl BRDF for SpecularBrdf {
    fn scatter_ray(
        &self,
        pcg: PCG,
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
            t_max: std::f32::INFINITY,
            depth,
        }
    }
}
