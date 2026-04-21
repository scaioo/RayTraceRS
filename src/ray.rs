//! The Ray module allows to represent light rays.
//!
//! The `Ray` object stores:
//! - `origin : Point` (origin of the light ray)
//! - `dir : Vector` (direction of the light ray)
//! - `t_min : f32` ... TODO DOCKING

use std::f32;
use crate::functions::are_close;
use crate::geometry::{Vector, Point, is_close};
use anyhow::{Result, anyhow};
use std::fmt::{Display, Formatter};


//================================================================
//                    Struct definition
//================================================================
// TODO DOCUMENTATION
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray{
    pub origin: Point,
    pub dir: Vector,
    pub t_max: f32,
    pub t_min: f32,
    pub depth : usize,
}

//================================================================
//                         Constructor
//================================================================

impl Ray {
   pub fn new(origin: Point, dir: Vector) -> Ray {
       Ray{
           origin,
           dir,
           t_max : f32::INFINITY,
           t_min : 1.0e-5,
           depth : 0,
       }
   }

}

//================================================================
//                         Display
//================================================================

impl Display for Ray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ray:\nO: {},\nD: {},\nt: [{}, {}],\ndepth: {}",
            self.origin,
            self.dir,
            self.t_min,
            self.t_max,
            self.depth
        )
    }
}


//================================================================
//                          Methods
//================================================================

impl Ray {

    pub fn set_depth(&mut self, depth : usize) {
        self.depth = depth;
    }

    pub fn set_borders(&mut self, t_max: f32, t_min: f32){
        self.t_max = t_max;
        self.t_min = t_min;
    }

    pub fn is_close(self, other: Ray) -> Result<bool> {
        let condition = is_close(self.origin, other.origin)
        && is_close(self.dir, other.dir)
        && self.t_max == other.t_max
        && self.t_min == other.t_min
        && self.depth == other.depth;

        if !condition {
            Err(anyhow!("Two rays are not approximately equal!\
            \nFirst ray:\n{}\nSecond ray:\n{}\n", self, other))
        } else { Ok(true) }
    }

    pub fn at(&self, t : f32) -> Point {
        self.origin + t * self.dir
    }
}


//================================================================
//                         Unit Tests
//================================================================

#[cfg(test)]
mod tests {
    use crate::functions::are_close;
    use super::*;

    // test constructor
    #[test]
    fn test_constructor(){
        let origin = Point::new(1.0, 2.0, 3.0);
        let dir = Vector::new(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, dir);
        assert_eq!(ray.origin, origin);
        assert_eq!(ray.dir, dir);
        assert!(ray.t_max.is_infinite());
        assert!(are_close(ray.t_min, 1e-5));
        assert_eq!(ray.depth, 0);
    }

    #[test]
    fn test_display(){
        let origin = Point::new(1.0, 2.0, 3.0);
        let dir = Vector::new(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, dir);

        assert_eq!(format!("{}", ray), "Ray:\nO: Point(x = 1, y = 2, z = 3),\n\
        D: Vector(x = 4, y = 5, z = 6),\nt: [0.00001, inf],\ndepth: 0");
    }

    #[test]
    fn test_is_close(){
        let ray1 = Ray::new(
            Point::new(1.0, 2.0, 3.0),
            Vector::new(4.0, 5.0, 6.0)
        );
        let ray2 = Ray::new(
            Point::new(1.0, 1.9999999, 3.0),
            Vector::new(4.000001, 5.0, 6.0)
        );
        ray1.is_close(ray2).unwrap();
    }

    #[test]
    fn test_ray_set_depth(){
        let mut ray = Ray::new(
            Point::new(1.0, 2.0, 3.0),
            Vector::new(4.0, 5.0, 6.0)
        );
        assert_eq!(ray.depth, 0);
        ray.set_depth(4);
        assert_eq!(ray.depth, 4);
    }

}