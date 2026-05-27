//! The BRDF encodes the Bidirectional Reflectance Distribution Function of the Rendering Equation.
//!

use crate::geometry::{Dot, Normal, Point, Vector, branchless_onb};
use crate::pcg::PCG;
use crate::ray::Ray;

/// This trait collects the various BRDF types.
pub trait BRDF {
    fn scatter_ray(
        &self,
        pcg: &mut PCG,
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
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct DiffusiveBrdf {}
impl BRDF for DiffusiveBrdf {
    /// This function returns a random direction ray scattered from a point in the surface.
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
impl DiffusiveBrdf {
    pub fn new() -> Self {
        Self {}
    }
}

// ======================================================
// SpecularBrdf
// ======================================================

/// SpecularBrdf represent the totally reflective materials.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpecularBrdf {}
impl BRDF for SpecularBrdf {
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
        (PCG::new(), incoming_dir, interacting_point, normal)
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

        let result =
            specular_brdf.scatter_ray(&mut PCG::new(), incoming_dir, interacting_point, normal, 5);

        assert_eq!(result.dir, expected_dir);
    }
}
