//! This module contains utilities to manage the observer.
//!
//! This doc has to be written!!

use std::ops::Mul;
use crate::geometry::{Point, Vector};
use crate::geometry::{X_AXIS};
use crate::ray::Ray;
use crate::transformations;
use crate::transformations::IsHomogeneousMatrix;

// =======================================================================
// CAMERA TRAIT
// =======================================================================

/// Marker trait for Camera classes
pub trait Camera{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32);

    fn fire_ray(&mut self, u : f32, v : f32) -> Ray;
}

// =======================================================================
// ORTHOGONAL CAMERA
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OrthogonalCamera<T : IsHomogeneousMatrix>{
    pub transformation : T,
    pub aspect_ratio : f32,
}

impl<T : IsHomogeneousMatrix> OrthogonalCamera<T>{
    pub fn new(transformation : T) -> OrthogonalCamera<T>{
        OrthogonalCamera{transformation, aspect_ratio : 1.0}
    }
}

impl<T> Camera for OrthogonalCamera<T>
where
    T: IsHomogeneousMatrix + Mul<Ray, Output = Ray> + Copy
{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32){
        self.aspect_ratio = aspect_ratio;
    }

    fn fire_ray(&mut self, u: f32, v : f32)-> Ray{
        // "Ugly but I hope fast" ~ Isacco.
        let point = Point {
            x: 0.0,
            y: (u + 1.0)/ (2.0 * self.aspect_ratio),
            z: (v +1.0)/ 2.0
        };
        let ray = Ray {
            origin: point, 
            dir : X_AXIS,
            t_max : f32::INFINITY,
            t_min : 1e-5,
            depth: 0
        };
        self.transformation * ray
    }
}

// =======================================================================
// PERSPECTIVE CAMERA
// =======================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerspectiveCamera<T : IsHomogeneousMatrix>{
    pub transformation : T,
    pub aspect_ratio : f32,
}

impl<T : IsHomogeneousMatrix> PerspectiveCamera<T>{
    pub fn new(transformation : T) -> PerspectiveCamera<T>{
        PerspectiveCamera{transformation, aspect_ratio : 1.0}
    }
}

impl<T> Camera for PerspectiveCamera<T>
where
    T: IsHomogeneousMatrix + Mul<Ray, Output = Ray> + Copy
{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32){
        self.aspect_ratio = aspect_ratio
    }

    fn fire_ray(&mut self, u: f32, v : f32)-> Ray {
        // TODO!!!!!
        Ray::new(Point::new(0.0,0.0,0.0), Vector::new(0.0,0.0, 0.0))
    }
}

#[cfg(test)]
mod tests{
    use crate::transformations::{Scaling, Transformation, Translation, XRotation, YRotation, ZRotation};
    use crate::functions::{equal_matrices, IDENTITY_4X4};
    use crate::geometry::Vector;
    use super::*;
    #[test]
    fn test_orthogonal_camera(){
        let transformation = Scaling::new([1.0, 2.0, 3.0]);
        let camera = OrthogonalCamera::new(transformation);
        let mat :[f32;16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 2.0, 0.0, 0.0,
            0.0, 0.0, 3.0, 0.0,
            0.0, 0.0, 0.0, 1.0
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
    fn test_perspective_camera(){
        let theta = std::f32::consts::PI / 4.0;
        let cos = theta.cos();
        let sin = theta.sin();
        let mat : [f32;16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, cos, -sin, 0.0,
            0.0, sin, cos, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];
        let transformation = XRotation::new(theta);
        let camera = PerspectiveCamera::new(transformation);
        assert!(equal_matrices(&mat, &camera.transformation.mat));
        assert_eq!(camera.aspect_ratio, 1.0);

        // Verify constructor compiles
        let _ = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let _ = OrthogonalCamera::new(Scaling::new([1.0, 2.0, 3.0]));
        let _ = OrthogonalCamera::new(YRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(ZRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(Translation::new(Vector::new(1.0, 2.0, 1.0)));
    }
}