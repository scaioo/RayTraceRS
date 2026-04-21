//! The Ray module allows to represent light rays.
//!
//! The `Ray` object stores:
//! - `origin : Point` (origin of the light ray)
//! - `dir : Vector` (direction of the light ray)
//! - `t_min : f32` ... TODO DOCKING

use std::f32;
use crate::geometry::{Vector, Point};


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
//                          Constructor
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

    pub fn set_depth(&mut self, depth : usize) {
        self.depth = depth;
    }

    pub fn set_borders(&mut self, t_max: f32, t_min: f32){
        self.t_max = t_max;
        self.t_min = t_min;
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