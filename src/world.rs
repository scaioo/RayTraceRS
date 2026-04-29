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
