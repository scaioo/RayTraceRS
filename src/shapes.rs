//! Shapes module adds scene objects utilities.
//!
//! In this module we define the trait `RayIntersection` that collects all the shape the user
//! can put in the image tracer scene. Then follows the shape classes: `Sphere`, `Plane` and `Triangle`.
//!
//! All the documentation is a WIP - draft!

use std::ops::{Mul, Sub};
use crate::camera::{Camera, PerspectiveCamera};
use crate::functions::{are_close, transpose_matrix};
use crate::hit_record::HitRecord;
use crate::ray::Ray;
use crate::geometry::{Dot, Normal, Point, Vec2D, Vector};
use crate::transformations::{IsHomogeneousMatrix, Transformation};

pub trait Shape {
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord>;

    fn normal_at(&self, point: Point, ray: &Ray) -> Normal;

    fn point_to_uv(&self, point: &Point) -> Vec2D;
}

/// The class Sphere adds the possibility to represent spherical objects in images
///
/// Draft:
/// Sphere implements:
/// 1. `RayIntersection` trait that determines the point of intersection between
/// the ray and the sphere
/// 2. A method that returns the normal of the sphere
/// 3. A method that returns the $(u,v)$ coordinates given the point of intersection
///
/// # Note:
///
/// All of this is for the unit sphere. To obtain other pseudo-spherical objects
/// we use transformations.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere<T: IsHomogeneousMatrix>{
    pub transformation: T // Is it where the sphere is at?
}

impl<T: IsHomogeneousMatrix> Sphere<T> {
    pub fn new(transformation: T) -> Self {
        Self { transformation }
    }
}

impl<T> Shape for Sphere<T>
where
    T: IsHomogeneousMatrix
    + Sub<Point, Output=Vector>
    + Mul<Ray, Output = Ray>
    + Mul<Point, Output = Point>
    + Mul<Normal, Output = Normal>
    + Mul<Vector, Output = Vector>
    + Copy
{
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let inverse_transformation =
            self.transformation.inverse_transformation();
        let transformed_ray = inverse_transformation * ray;

        let origin =
            transformed_ray.origin - Point::new(0.0, 0.0, 0.0);

        let a = transformed_ray.dir.squared_norm();
        let half_b = origin.dot(&transformed_ray.dir);
        let c = origin.squared_norm() - 1.0;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 || are_close(discriminant, 0.0) {
           return None;
        }

        let sqrt_d = discriminant.sqrt();
        let t1 = (-half_b - sqrt_d) / a;
        let t2 = (-half_b + sqrt_d) / a;

        let condition = |t:f32| t > transformed_ray.t_min && t < transformed_ray.t_max;

        let t = if condition(t1) {
            t1
        }else if condition(t2) {
            t2
        }else { return None };

        let hit_point = transformed_ray.at(t);

        Some(
            HitRecord{
                world_point : self.transformation * hit_point,
                normal: self.transformation * self.normal_at(hit_point, &transformed_ray),
                uv : self.point_to_uv(&hit_point),
                t,
                ray
            }
        )
    }


    fn normal_at(&self, point: Point, ray: &Ray) -> Normal {
        let result = Normal::new(point.x, point.y, point.z);
        let vector = point - Point::new(0.0, 0.0, 0.0);
        if (vector.dot(&ray.dir)) < 0.0 {
            result
        } else {- result}
    }

    fn point_to_uv(&self, point: &Point) -> Vec2D {
        let pi = std::f32::consts::PI;
        let mut u = point.y.atan2(point.x) / (2.0 * pi);
        if u < 0.0 {
            u += 1.0;
        }

        let v = point.z.acos() / pi;

        Vec2D { x: u, y : v }
    }
}

/// The class Plane adds the possibility to represent the plane in an image
///
/// Draft:
/// Plane implements:
/// 1. `RayIntersection` trait that determines the point of intersection between
/// the ray and the plane
/// 2. A method that returns the normal of the plane
/// 3. A method that returns the $(u,v)$ coordinates given the point of intersection
///
/// # Note:
///
/// All of this is for the x-y plane. To obtain other pseudo-spherical objects
/// we use transformations.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane{}



/// The class Triangle adds the possibility to represent a triangle in an image
///
/// Draft:
/// Triangle implements:
/// 1. `RayIntersection` trait that determines the point of intersection between
/// the ray and the triangle
/// 2. A method that returns the normal of the triangle
/// 3. A method that returns the $(u,v)$ coordinates given the point of intersection
///
/// # Note:
///
/// Understand where to put the triangle properly for then further transformations!
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Triangle{}