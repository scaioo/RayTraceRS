//! A scene is composed of many shapes.
//! This module implements a list of shapes: the `World` type.
//!
//! - It maintains a list of `Shape` objects.
//! - It implements a `ray_intersection` method that iterates over the shapes,
//! searches for intersections, and returns the one closest to the ray origin.

use crate::geometry::Point;
use crate::ray::Ray;
use crate::shapes::Shape;

pub struct World {
    pub objects: Vec<Box<dyn Shape>>,
}

impl World {
    pub fn ray_intersection(&self, ray: Ray) -> Option<Point> {
        // Note: this returns the first intersection

        // I would try not to dump the world object
        let iter = self.objects.iter().clone();

        let mut t = ray.t_max;
        let mut found_intersection = false;

        for object in iter {
            let t_intersection = match object.ray_intersection(ray) {
                Some(a) => a.t,
                None => continue,
            };
            if t_intersection < t && t_intersection > ray.t_min {
                t = t_intersection;
                found_intersection = true;
            }
        }

        if found_intersection {
            Some(ray.at(t))
        } else { None }
    }
}


#[cfg(test)]
mod tests {
    use crate::functions::are_close;
    use crate::geometry::{is_close, Point, Vector};
    use crate::ray::Ray;
    use crate::shapes::{Sphere};
    use crate::transformations::{Translation, Scaling,};
    use crate::world::World;

    fn setup() -> World {
        let sphere1 = Sphere::new(Translation::new(Vector::new(5.0, 0.0, 0.0)));
        let sphere2 = Sphere::new(Translation::new(Vector::new(0.0, 5.0, 0.0)));
        let bean = Sphere::new(Scaling::new([1.0, 1.0, 2.0]));

        World{objects: vec![Box::new(sphere1), Box::new(sphere2), Box::new(bean)]}
    }

    #[test]
    fn test_ray_intersection1() {
        let world = setup();
        let point1 = Point::new(10.0, 0.0, 1.5);
        let dir = Vector::new(-1.0, 0.0, 0.0);

        // Only captures the bean
        let ray = Ray::new(point1, dir);
        let hit_point = match world.ray_intersection(ray) {
            Some(a) => a,
            None => panic!("No intersection found."),
        };
        let implicit =
            hit_point.x * hit_point.x
            + hit_point.y * hit_point.y
            + hit_point.z * hit_point.z / 4.0;
        assert!(are_close(implicit, 1.0));
    }

    #[test]
    fn test_ray_intersection2() {
        let world = setup();
        let points = vec![
            Point::new(10.0, 0.0, 0.0),
            Point::new(10.0, 1.0, 0.0),
            Point::new(10.0, 3.0, 0.0),
            Point::new(10.0, 5.0, 0.0),
            Point::new(10.0, 10.0, 0.0)
        ];
        let rays = points
            .clone()
            .iter()
            .map(|point| Ray::new(point.clone(), Vector::new(-1.0, 0.0, 0.0)))
            .collect::<Vec<Ray>>();

        let expected : [Option<Point>; 5] = [
            Some(Point::new(6.0, 0.0, 0.0)),
            None,
            None,
            Some(Point::new(1.0,5.0,0.0)),
            None
        ];

        for i in 0..5 {
            assert_eq!(expected[i], world.ray_intersection(rays[i]));
        }
    }
}