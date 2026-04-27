//! Shapes module adds scene objects utilities.
//!
//! In this module we define the trait `RayIntersection` that collects all the shape the user
//! can put in the image tracer scene. Then follows the shape classes: `Sphere`, `Plane` and `Triangle`.
//!
//! All the documentation is a WIP - draft!

use crate::functions::are_close;
use crate::geometry::{Dot, Normal, Point, Vec2D, Vector};
use crate::hit_record::HitRecord;
use crate::ray::Ray;
use crate::transformations::{IsHomogeneousMatrix, Transformation};
use std::ops::Mul;

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
pub struct Sphere<T: IsHomogeneousMatrix> {
    pub transformation: T, // Is it where the sphere is at?
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
        + Mul<Point, Output = Point>
        + Mul<Normal, Output = Normal>
        + Mul<Vector, Output = Vector>
        + Copy,
{
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let inverse_transformation = self.transformation.inverse_transformation();
        let transformed_ray = inverse_transformation * ray;

        let origin = transformed_ray.origin - Point::new(0.0, 0.0, 0.0);

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

        let condition = |t: f32| t > transformed_ray.t_min && t < transformed_ray.t_max;

        let t = if condition(t1) {
            t1
        } else if condition(t2) {
            t2
        } else {
            return None;
        };

        let hit_point = transformed_ray.at(t);

        Some(HitRecord {
            world_point: self.transformation * hit_point,
            normal: self.transformation * self.normal_at(hit_point, &transformed_ray),
            uv: self.point_to_uv(&hit_point),
            t,
            ray,
        })
    }

    fn normal_at(&self, point: Point, ray: &Ray) -> Normal {
        let result = Normal::new(point.x, point.y, point.z);
        let vector = point - Point::new(0.0, 0.0, 0.0);
        if (vector.dot(&ray.dir)) < 0.0 {
            result
        } else {
            -result
        }
    }

    fn point_to_uv(&self, point: &Point) -> Vec2D {
        let pi = std::f32::consts::PI;
        let mut u = point.y.atan2(point.x) / (2.0 * pi);
        if u < 0.0 {
            u += 1.0;
        }

        let v = point.z.acos() / pi;

        Vec2D { x: u, y: v }
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
pub struct Plane<T: IsHomogeneousMatrix> {
    pub transformation: T,
}
impl<T: IsHomogeneousMatrix> Plane<T> {
    pub fn new(transformation: T) -> Self {
        Self { transformation }
    }
}
impl<T> Shape for Plane<T>
where
    T: IsHomogeneousMatrix
        + Mul<Ray, Output = Ray>
        + Mul<Point, Output = Point>
        + Mul<Normal, Output = Normal>
        + Mul<Vector, Output = Vector>
        + Copy,
{
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let inverse_transformation = self.transformation.inverse_transformation();
        let transformed_ray = inverse_transformation * ray;

        if are_close(transformed_ray.dir.z, 0.0) {
            return None;
        }
        let t = -transformed_ray.origin.z / transformed_ray.dir.z;

        if t <= transformed_ray.t_min || t >= transformed_ray.t_max {
            return None;
        }
        let hit_point = transformed_ray.at(t);
        Some(HitRecord {
            world_point: self.transformation * hit_point,
            normal: self.transformation * self.normal_at(hit_point, &transformed_ray),
            uv: self.point_to_uv(&hit_point),
            t,
            ray,
        })
    }

    fn normal_at(&self, _point: Point, ray: &Ray) -> Normal {
        let result = Normal::new(0.0, 0.0, 1.0);
        if ray.dir.z > 0.0 { -result } else { result }
    }
    fn point_to_uv(&self, point: &Point) -> Vec2D {
        Vec2D {
            x: point.x - point.x.floor(),
            y: point.y - point.y.floor(),
        }
    }
}

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
pub struct Triangle {}

// =================================================================================
//
//                                    TESTS
//
// =================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::is_close;
    use crate::transformations::{Transformation, Translation};

    fn setup1() -> (Sphere<Transformation>, [Ray; 3]) {
        let rays = [
            Ray::new(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0)),
            Ray::new(Point::new(3.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0)),
            Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)),
        ];

        let transformation = Transformation::new(IDENTITY_4X4);
        let sphere = Sphere::new(transformation);
        (sphere, rays)
    }

    #[test]
    fn test_sphere_ray_point_intersection1() {
        let (sphere, rays) = setup1();

        let points: [Point; 3] = [
            Point::new(0.0, 0.0, 1.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ];

        for i in 0..3 {
            let hit_record = match sphere.ray_intersection(rays[i]) {
                None => panic!("ray_intersection IS WRONG!"),
                Some(h) => h,
            };
            assert!(
                is_close(hit_record.world_point, points[i]),
                "Error occurred: index {} is responsible.\n",
                i
            );
        }
    }

    #[test]
    fn test_sphere_uv_point_intersection1() {
        let (sphere, rays) = setup1();

        let uv_points: [Vec2D; 3] = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.0, 0.5),
            Vec2D::new(0.0, 0.5),
        ];

        for i in 0..3 {
            let hit_record = match sphere.ray_intersection(rays[i]) {
                None => panic!("ray_intersection IS WRONG!"),
                Some(h) => h,
            };
            assert!(
                hit_record.uv.is_close(&uv_points[i]),
                "Error occurred: index {} is responsible.\n",
                i
            );
        }
    }

    #[test]
    fn test_sphere_ray_normal_att1() {
        let (sphere, rays) = setup1();

        let normals: [Normal; 3] = [
            Normal::new(0.0, 0.0, 1.0),
            Normal::new(1.0, 0.0, 0.0),
            Normal::new(-1.0, 0.0, 0.0),
        ];

        for i in 0..3 {
            let hit_record = match sphere.ray_intersection(rays[i]) {
                None => panic!("ray_intersection IS WRONG!"),
                Some(h) => h,
            };
            assert!(
                is_close(hit_record.normal, normals[i]),
                "Error occurred: index {} is responsible.\n",
                i
            );
        }
    }

    fn setup2() -> (Sphere<Translation>, Ray, Ray) {
        let translation = Translation::new(Vector::new(10.0, 0.0, 0.0));
        let sphere = Sphere::new(translation);
        let ray = Ray::new(Point::new(10.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0));
        let ray2 = Ray::new(Point::new(13.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0));

        (sphere, ray, ray2)
    }

    #[test]
    fn test_sphere_ray_point_intersection2() {
        let (sphere, ray, ray2) = setup2();

        let point = Point::new(10.0, 0.0, 1.0);

        let hit_record = match sphere.ray_intersection(ray) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.world_point, point),
            "Error occurred (1): point : {}\nhit_record.world_point : {}\n",
            point,
            hit_record.world_point
        );

        let point = Point::new(11.0, 0.0, 0.0);

        let hit_record = match sphere.ray_intersection(ray2) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.world_point, point),
            "Error occurred (2): point : {}\nhit_record.world_point : {}\n",
            point,
            hit_record.world_point
        );
    }

    #[test]
    fn test_sphere_ray_normal_att2() {
        let (sphere, ray, ray2) = setup2();

        let normal = Normal::new(0.0, 0.0, 1.0);

        let hit_record = match sphere.ray_intersection(ray) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.normal, normal),
            "Error occurred (1): normal : {}\nhit_record.normal : {}\n",
            normal,
            hit_record.normal
        );

        let normal = Normal::new(1.0, 0.0, 0.0);

        let hit_record = match sphere.ray_intersection(ray2) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.normal, normal),
            "Error occurred (2): normal : {}\nhit_record.normal : {}\n",
            normal,
            hit_record.normal
        );
    }

    #[test]
    fn test_sphere_uv_point_intersection2() {
        let (sphere, ray, ray2) = setup2();

        let uv = Vec2D::new(0.0, 0.0);

        let hit_record = match sphere.ray_intersection(ray) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            uv.is_close(&hit_record.uv),
            "Error occurred (1): uv : {:?}\nhit_record.uv : {:?}\n",
            uv,
            hit_record.uv
        );

        let uv = Vec2D::new(0.0, 0.50);

        let hit_record = match sphere.ray_intersection(ray2) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            uv.is_close(&hit_record.uv),
            "Error occurred (2): uv : {:?}\nhit_record.uv : {:?}\n",
            uv,
            hit_record.uv
        );
    }

    #[test]
    fn test_sphere_ray_miss() {
        let (sphere, _, _) = setup2();
        let ray = Ray::new(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0));

        let hit_record = sphere.ray_intersection(ray);
        assert!(
            hit_record.is_none(),
            "Error occurred (1): there is intersection where shouldn't!\n{}",
            hit_record.unwrap().world_point
        );

        let ray = Ray::new(Point::new(-10.0, 0.0, 0.0), Vector::new(0.0, 0.0, -1.0));

        let hit_record = sphere.ray_intersection(ray);
        assert!(
            hit_record.is_none(),
            "Error occurred (2): there is intersection where shouldn't!\n{}",
            hit_record.unwrap().world_point
        );
    }
    fn setup_plane() -> (Plane<Transformation>, Ray, Ray, Ray) {
        let transformation = Transformation::new(IDENTITY_4X4);
        let plane = Plane::new(transformation);

        // Ray from top to bottom
        let ray_top = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, -1.0));
        // Ray from bottom to top
        let ray_bottom = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        // Ray parallel to the plane
        let ray_parallel = Ray::new(Point::new(0.0, 0.0, 2.0), Vector::new(1.0, 0.0, 0.0));

        (plane, ray_top, ray_bottom, ray_parallel)
    }

    #[test]
    #[test]
    fn test_plane_intersection_and_normal() {
        let (plane, ray_top, ray_bottom, ray_parallel) = setup_plane();

        // Test 1: Impact from top
        let hit_top = plane
            .ray_intersection(ray_top)
            .expect("Should hit the plane");
        assert!(are_close(hit_top.t, 5.0));
        assert!(is_close(hit_top.world_point, Point::new(0.0, 0.0, 0.0)));
        assert!(is_close(hit_top.normal, Normal::new(0.0, 0.0, 1.0)));
        assert!(hit_top.uv.is_close(&Vec2D::new(0.0, 0.0)));

        // Test 2: Impact from bottom (normal change sign)
        let hit_bottom = plane
            .ray_intersection(ray_bottom)
            .expect("Should hit the plane");
        assert!(are_close(hit_bottom.t, 5.0));
        assert!(is_close(hit_bottom.world_point, Point::new(0.0, 0.0, 0.0)));
        assert!(is_close(hit_bottom.normal, Normal::new(0.0, 0.0, -1.0)));
        assert!(hit_bottom.uv.is_close(&Vec2D::new(0.0, 0.0)));

        // Test 3: parallel impact (no impact)
        let hit_parallel = plane.ray_intersection(ray_parallel);
        assert!(
            hit_parallel.is_none(),
            "Parallel ray should not hit the plane"
        );
    }

    #[test]
    fn test_plane_uv_fractional_coordinates() {
        let transformation = Transformation::new(IDENTITY_4X4);
        let plane = Plane::new(transformation);

        // A ray hits the plane in x = 2.5, y = -1.3
        let ray = Ray::new(Point::new(2.5, -1.3, 5.0), Vector::new(0.0, 0.0, -1.0));

        let hit = plane.ray_intersection(ray).expect("Should hit the plane");

        //  x = 2.5: 2.5 - 2.0 = 0.5
        //  y = -1.3: -1.3 - floor(-1.3) = -1.3 - (-2.0) = 0.7
        assert!(hit.uv.is_close(&Vec2D::new(0.5, 0.7)));
    }
}
