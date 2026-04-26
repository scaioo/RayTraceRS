//! Shapes module adds scene objects utilities.
//!
//! In this module we define the trait `RayIntersection` that collects all the shape the user
//! can put in the image tracer scene. Then follows the shape classes: `Sphere`, `Plane` and `Triangle`.
//!
//! All the documentation is a WIP - draft!

use std::ops::Mul;
use crate::camera::{Camera, PerspectiveCamera};
use crate::functions::{are_close, transpose_matrix};
use crate::hit_record::HitRecord;
use crate::ray::Ray;
use crate::geometry::{Dot, Normal, Point, Vec2D, Vector};
use crate::transformations::{IsHomogeneousMatrix, Transformation};

pub trait Shape {
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord>;

    fn normal_at(&self) -> Normal;

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
    + Mul<Ray, Output = Ray>
    + Copy
    + Mul<Point, Output = Point>
    + Mul<Normal, Output = Normal>
    + Mul<Vector, Output = Vector>
{
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let inverse_transformation =
            self.transformation.inverse_transformation();
        let transformed_ray = inverse_transformation * ray;

        let origin =
            transformed_ray.origin - Point::new(0.0, 0.0, 0.0);

        let module_of_d = transformed_ray.dir.norm();
        let o_times_d =  origin.dot(&transformed_ray.dir);
        let delta_fourth : f32 =
            o_times_d * o_times_d - module_of_d * module_of_d * (origin.norm() - 1.0);

        if delta_fourth < 0.0 || are_close(delta_fourth, 0.0) {
            return None;
        } else {
            let t_1 : f32 = (- o_times_d - delta_fourth.sqrt()) / module_of_d;
            let t_2 : f32 = (- o_times_d + delta_fourth.sqrt()) / module_of_d;

            let condition = |t:f32| t > transformed_ray.t_min && t < transformed_ray.t_max;
            if condition(t_1){
                let hit_point = transformed_ray.at(t_1);
                Some( HitRecord {
                    world_point : self.transformation * hit_point,
                    normal : self.transformation * self.normal_at(),
                    surface_normal : self.point_to_uv(&hit_point),
                    t : t_1,
                    ray
                })
            }else if condition(t_2){
                let hit_point = transformed_ray.at(t_2);
                Some(HitRecord{
                    world_point : self.transformation * hit_point,
                    normal : self.transformation * self.normal_at(),
                    surface_normal : self.point_to_uv(&hit_point),
                    t : t_2,
                    ray
                })
            } else { None } // Does it make sense?
        }
    }

    fn normal_at(&self) -> Normal {
        // TODO!!!!!
        Normal::new(0.0,0.0,0.0)
    }

    fn point_to_uv(&self, point: &Point) -> Vec2D {
        // TODO!!!!!
        Vec2D::new(point.x, point.y)
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