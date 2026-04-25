//! This module contains utilities to manage the observer.
//!
//! This doc has to be written!!
//! Note: the canvas is put perpendicular to the X-Axis with width 2a and hight 2.0.
//! Note 2 : need to add all the validations!!!!!

use crate::functions::are_close;
use crate::geometry::Point;
use crate::geometry::X_AXIS;
use crate::ray::Ray;
use crate::transformations::IsHomogeneousMatrix;
use std::ops::Mul;

// =======================================================================
// CAMERA TRAIT
// =======================================================================

/// Marker trait for Camera classes
pub trait Camera {
    fn set_aspect_ratio(&mut self, aspect_ratio: f32);

    fn fire_ray(&self, u: f32, v: f32) -> Ray;
}

// =======================================================================
// ORTHOGONAL CAMERA
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OrthogonalCamera<T: IsHomogeneousMatrix> {
    pub transformation: T,
    pub aspect_ratio: f32,
}

impl<T: IsHomogeneousMatrix> OrthogonalCamera<T> {
    // Is it ok to give a Return<> type so we can handle
    // wrong aspect_ratios?
    pub fn new(transformation: T) -> OrthogonalCamera<T> {
        OrthogonalCamera {
            transformation,
            aspect_ratio: 1.0,
        }
    }
}

impl<T> Camera for OrthogonalCamera<T>
where
    T: IsHomogeneousMatrix + Mul<Ray, Output = Ray> + Copy,
{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        if aspect_ratio < 0.0 || are_close(aspect_ratio, 0.0) {
            panic!("invalid aspect ratio {}", aspect_ratio);
        }
        self.aspect_ratio = aspect_ratio;
    }

    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        // "Ugly but I hope fast" ~ Isacco.
        let point = Point {
            x: -1.0,
            y: - self.aspect_ratio * (2.0 * u - 1.0),
            z: 2.0 * v - 1.0,
        };
        let ray = Ray {
            origin: point,
            dir: X_AXIS,
            t_max: f32::INFINITY,
            t_min: 1e-5,
            depth: 0,
        };
        self.transformation * ray
    }
}

// =======================================================================
// PERSPECTIVE CAMERA
// =======================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerspectiveCamera<T: IsHomogeneousMatrix> {
    pub transformation: T,
    pub aspect_ratio: f32,
    pub distance: f32,
}

impl<T: IsHomogeneousMatrix> PerspectiveCamera<T> {
    pub fn new(transformation: T) -> PerspectiveCamera<T> {
        // Is it ok to give a Return<> type so we can handle
        // too small distances and wrong aspect_ratios?
        PerspectiveCamera {
            transformation,
            aspect_ratio: 1.0,
            distance: 1.0,
        }
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }
}

impl<T> Camera for PerspectiveCamera<T>
where
    T: IsHomogeneousMatrix + Mul<Ray, Output = Ray> + Copy,
{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        if aspect_ratio < 0.0 || are_close(aspect_ratio, 0.0) {
            panic!("invalid aspect ratio {}", aspect_ratio);
        }
        self.aspect_ratio = aspect_ratio
    }

    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        let point = Point {
            x: 0.0,
            y: - self.aspect_ratio * (2.0 * u - 1.0),
            z: 2.0 * v - 1.0,
        };

        let ray = Ray {
            origin: Point {
                x: -self.distance,
                y: 0.0,
                z: 0.0,
            },
            dir: point - Point::new(-self.distance, 0.0, 0.0),
            t_max: f32::INFINITY,
            t_min: 1e-5,
            depth: 0,
        };
        self.transformation * ray
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::{IDENTITY_4X4, equal_matrices};
    use crate::geometry::{Cross, Vector, is_close};
    use crate::transformations::{
        Scaling, Transformation, Translation, XRotation, YRotation, ZRotation,
    };
    #[test]
    fn test_orthogonal_camera() {
        let transformation = Scaling::new([1.0, 2.0, 3.0]);
        let camera = OrthogonalCamera::new(transformation);
        let mat: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(&mat, &camera.transformation.mat));
        assert_eq!(camera.aspect_ratio, 1.0);

        // Verify constructor compiles
        let _ = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let _ = OrthogonalCamera::new(XRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(YRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(ZRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(Translation::new(Vector::new(1.0, 2.0, 1.0)));
    }
    #[test]
    fn test_orthogonal_camera_transform() {
        let transformation = Translation::new(Vector::new(0.0, -2.0, 0.0))
            * ZRotation::new(std::f32::consts::FRAC_PI_2);

        let camera = OrthogonalCamera::new(transformation);
        let ray = camera.fire_ray(0.5, 0.5);

        assert!(is_close(ray.at(1.0), Point::new(0.0, -2.0, 0.0)));
    }

    #[test]
    #[should_panic(expected = "invalid aspect ratio")]
    fn test_oc_set_ar() {
        let mut orthogonal_camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        assert_eq!(orthogonal_camera.aspect_ratio, 1.0);
        orthogonal_camera.set_aspect_ratio(16.0 / 9.0);
        println!("not exploded\n");
        assert_eq!(orthogonal_camera.aspect_ratio, 16.0 / 9.0);
        orthogonal_camera.set_aspect_ratio(-9.0);
    }

    #[test]
    fn test_oc_fire_ray() {
        let mut orthogonal_camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let aspect_ratio = 16.0 / 9.0;
        orthogonal_camera.set_aspect_ratio(aspect_ratio);
        let ray1 = orthogonal_camera.fire_ray(0.0, 0.0);
        let ray2 = orthogonal_camera.fire_ray(0.0, 1.0);
        let ray3 = orthogonal_camera.fire_ray(1.0, 0.0);
        let ray4 = orthogonal_camera.fire_ray(1.0, 1.0);
        let vec = vec![ray1, ray2, ray3, ray4];

        for i in 0..3 {
            let cross_product = vec[i].dir.cross(&vec[i + 1].dir);
            assert!(is_close(cross_product, Vector::new(0.0, 0.0, 0.0)));
        }
        assert!(ray1.at(1.0).is_close(&Point::new(0.0, aspect_ratio, -1.0)));
        assert!(ray2.at(1.0).is_close(&Point::new(0.0, aspect_ratio, 1.0)));
        assert!(ray3.at(1.0).is_close(&Point::new(0.0, -aspect_ratio, -1.0)));
        assert!(ray4.at(1.0).is_close(&Point::new(0.0, -aspect_ratio, 1.0)));
    }

    #[test]
    fn test_perspective_camera_constructor() {
        let theta = std::f32::consts::PI / 4.0;
        let cos = theta.cos();
        let sin = theta.sin();
        let mat: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let transformation = XRotation::new(theta);
        let camera = PerspectiveCamera::new(transformation);
        assert!(equal_matrices(&mat, &camera.transformation.mat));
        assert_eq!(camera.aspect_ratio, 1.0);

        // Verify constructor compiles
        let _ = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        let _ = PerspectiveCamera::new(Scaling::new([1.0, 2.0, 3.0]));
        let _ = PerspectiveCamera::new(YRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = PerspectiveCamera::new(ZRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = PerspectiveCamera::new(Translation::new(Vector::new(1.0, 2.0, 1.0)));
    }

    #[test]
    fn test_perspective_camera_transformation() {
        let transformation = ZRotation::new(std::f32::consts::PI);
        let camera = PerspectiveCamera::new(transformation);
        let ray = camera.fire_ray(1.0, 0.0);

        // Default aspect_ration and distance
        println!("{:?}", ray);
        assert!(is_close(ray.at(1.0), Point::new(0.0, 1.0, -1.0)));
    }

    #[test]
    fn test_pc_set_distance() {
        let mut perspective_camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        perspective_camera.set_distance(16.0);
        assert_eq!(perspective_camera.distance, 16.0);
    }

    #[test]
    #[should_panic(expected = "invalid aspect ratio")]
    fn test_pc_set_ar() {
        let mut perspective_camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        perspective_camera.set_aspect_ratio(19.0);
        assert_eq!(perspective_camera.aspect_ratio, 19.0);
        println!("not exploded\n");
        perspective_camera.set_aspect_ratio(0.0000001);
    }

    #[test]
    fn test_pc_fire_ray() {
        let mut perspective_camera = PerspectiveCamera::new(
            Transformation::new(IDENTITY_4X4)
        );
        perspective_camera.set_aspect_ratio(2.0);
        perspective_camera.set_distance(1.0);

        let angles = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];

        let mut rays: Vec<Ray> = Vec::with_capacity(4);

        for i in 0..4 {
            let matrix = angles[i];
            let ray = perspective_camera.fire_ray(matrix[0], matrix[1]);
            let screen = Point {
                x: 0.0,
                y: - 2.0 * (2.0 * matrix[0] - 1.0),
                z: 2.0 * matrix[1] - 1.0,
            };
            let expected_vector = screen - Point::new(-1.0, 0.0, 0.0);
            assert!(is_close(expected_vector, ray.dir));
            rays.push(ray);
        }

        for i in 0..3 {
            assert!(
                is_close(rays[i].origin, rays[i + 1].origin),
                "ray{}:\n{}\nray{}:\n{}",
                i + 1,
                rays[i],
                i + 2,
                rays[i + 1]
            );

            assert!(!is_close(rays[i].dir, rays[i + 1].dir),);
        }
    }
}
