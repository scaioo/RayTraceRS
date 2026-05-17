//! The BRDF encodes the Bidirectional Reflectance Distribution Function of the Rendering Equation.
//!

use crate::color::Color;
use crate::geometry::{Normal, Point, Vec2D, Vector, branchless_onb};
use crate::pcg::PCG;
use crate::pigments;
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
// BRDF
// ======================================================

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
