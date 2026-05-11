//! A scene is represented as a collection of geometric objects.
//! This module implements a list of shapes: the `World` type.
//!
//! - It maintains a list of `Shape` objects.
//! - It implements a `ray_intersection` method that iterates over the shapes,
//!   searches for intersections, and returns the one closest to the ray origin.
use crate::hit_record::HitRecord;
use crate::ray::Ray;
use crate::shapes::Shape;
use std::ops::Add;

/// A `World` is a collection of scene objects.
pub struct World {
    pub objects: Vec<Box<dyn Shape>>,
}

impl World {
    /// Finds the nearest intersection between the ray and any object in the world.
    ///
    /// It iterates through all objects and returns the [`HitRecord`] of the
    /// closest hit within the ray's valid `[t_min, t_max]` interval.
    ///
    /// # Returns
    /// - `Some(HitRecord)` if at least one intersection is found.
    /// - `None` if the ray misses everything or internal errors occur in specific shapes.
    pub fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;
        let mut closest_t = ray.t_max;

        for object in &self.objects {
            // We use the Option returned by the shape.
            // If the shape had an internal Result error (like in point_to_uv),
            // it already returned None via .ok()?, so the world stays safe.
            if let Some(hit) = object.ray_intersection(ray)
                && hit.t < closest_t
                && hit.t > ray.t_min
            {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}

impl Add for World {
    type Output = World;

    /// Merges two [`World`] instances into one.
    ///
    /// This operation consumes both worlds and returns a new world containing
    /// the combined collection of objects. The objects from `rhs` are appended
    /// to the end of the existing object list.
    fn add(mut self, rhs: World) -> World {
        self.objects.extend(rhs.objects);
        self
    }
}
#[cfg(test)]
mod tests {
    use crate::functions::{IDENTITY_4X4, are_close};
    use crate::geometry::{is_close, Point, Vector};
    use crate::ray::Ray;
    use crate::shapes::{Plane, Sphere};
    use crate::transformations::{Scaling, Transformation, Translation};
    use crate::world::World;
    use anyhow::Result;

    fn setup() -> World {
        let sphere1 = Sphere::new(Translation::new(Vector::new(5.0, 0.0, 0.0)));
        let sphere2 = Sphere::new(Translation::new(Vector::new(0.0, 5.0, 0.0)));
        let bean = Sphere::new(Scaling::new([1.0, 1.0, 2.0]));

        World {
            objects: vec![Box::new(sphere1), Box::new(sphere2), Box::new(bean)],
        }
    }
    #[test]
    fn test_ray_intersection1() -> Result<()> {
        let world = setup();
        let point1 = Point::new(10.0, 0.0, 1.5);
        let dir = Vector::new(-1.0, 0.0, 0.0);

        let ray = Ray::new(point1, dir);

        // Clean error handling in tests: no match/panic, just expect or ?
        let hit = world
            .ray_intersection(&ray)
            .expect("Expected an intersection with the 'bean' sphere");

        let hit_point = hit.world_point;
        let implicit =
            hit_point.x * hit_point.x + hit_point.y * hit_point.y + hit_point.z * hit_point.z / 4.0;

        assert!(are_close(implicit, 1.0));
        Ok(())
    }

    #[test]
    fn test_ray_intersection2() {
        let world = setup();
        let test_cases = vec![
            (Point::new(10.0, 0.0, 0.0), Some(Point::new(6.0, 0.0, 0.0))),
            (Point::new(10.0, 1.0, 0.0), None),
            (Point::new(10.0, 3.0, 0.0), None),
            (Point::new(10.0, 5.0, 0.0), Some(Point::new(1.0, 5.0, 0.0))),
            (Point::new(10.0, 10.0, 0.0), None),
        ];

        for (origin, expected_point) in test_cases {
            let ray = Ray::new(origin, Vector::new(-1.0, 0.0, 0.0));
            let hit = world.ray_intersection(&ray);

            match (hit, expected_point) {
                (Some(h), Some(p)) => assert!(is_close(h.world_point, p)),
                (None, None) => {}
                _ => panic!("Intersection mismatch for origin {:?}", origin),
            }
        }
    }

    #[test]
    fn test_add() {
        let world_1 = setup();

        let transformation = Transformation::new(IDENTITY_4X4);
        let plane = Plane::new(transformation);
        let world_2 = World {
            objects: vec![Box::new(plane), Box::new(plane), Box::new(plane)],
        };

        let world = world_1 + world_2;

        assert_eq!(world.objects.len(), 6);
    }
}
