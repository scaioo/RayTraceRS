//! A scene is composed of many shapes.
//! This module implements a list of shapes: the `World` type.
//!
//! - It maintains a list of `Shape` objects.
//! - It implements a `ray_intersection` method that iterates over the shapes,
//! searches for intersections, and returns the one closest to the ray origin.

use crate::geometry::Point;
use crate::shapes::Shape;

pub struct World {
    pub objects: Vec<Box<dyn Shape>>,
}

impl World {
    pub fn ray_intersection(&self) -> Point {
        panic!("Write function to ray_intersection!")
    }
}
