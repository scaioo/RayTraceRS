//! HitRecord returns information about an intersection. (Doc Draft)
//!
//! This class contains the following information:
//! - `world_point` : 3D point where the intersection occurred (`geometry::Point`);
//! - `normal` : surface normal at the intersection (`geometry::Normal`);
//! - `surface_point` : (u,v) coordinates of the intersection (`geometry::Vec2d`);
//! - `t`: ray parameter associated with the intersection (`f32`);
//! - `ray`: the light ray that caused the intersection (`ray::Ray`).
//!
//! Note for programmers: add `is_close`/`are_close` method for debugging.

use crate::functions::are_close;
use crate::ray::Ray;
use crate::geometry::{Point, Normal, Vec2D, is_close};

#[derive(Clone, Copy,  Debug, PartialEq)]
pub struct HitRecord{
    pub world_point: Point,
    pub normal: Normal,
    pub surface_normal: Vec2D,
    pub t : f32,
    pub ray: Ray,
}


impl HitRecord{
    pub fn is_close(&self, hr: &HitRecord) ->bool{
        is_close(self.world_point, hr.world_point )
            && is_close(self.normal, hr.normal)
            && are_close(self.t, hr.t)
            && self.surface_normal.is_close(&hr.surface_normal)
            && self.ray.is_close(hr.ray)
    }
}


#[cfg(test)]
mod tests{
    use crate::geometry::{Vector, Point, Vec2D};
    use super::*;
    #[test]
    fn test_is_close() {
        let ray = Ray::new(Point::new(0.0,1.0,-1.0), Vector::new(0.0, 0.0, 0.0));
        let mut hr1 = HitRecord{
            world_point : Point::new(1.0,2.0,3.0),
            normal : Normal::new(4.0, 5.0,6.0),
            surface_normal : Vec2D::new(7.0,8.0),
            t : 1.0,
            ray,
        };
        let ray2 = Ray::new(
            Point::new(0.0000001,1.0,-0.999999),
            Vector::new(0.0000001, 0.0, -0.0000001),
        );
        let mut hr2 = HitRecord {
            world_point : Point::new(1.0, 1.999999, 3.0),
            normal : Normal::new(4.0000001,5.0, 6.0),
            surface_normal : Vec2D::new(7.0, 8.0000001),
            t : 1.0000001,
            ray: ray2
        };
        assert!(hr1.is_close(&hr2));

        hr2.t = 0.0001;
        assert!(!hr1.is_close(&hr2));

        hr1.t = 0.0001; // Make it equal
        hr2.normal = Normal::new(4.001, 5.0, 6.0);
        assert!(!hr1.is_close(&hr2));

        hr1.normal = Normal::new(4.001, 5.0, 6.0); // Make it equal
        hr2.world_point = Point::new(1.0,2.001,3.0);
        assert!(!hr1.is_close(&hr2));

        hr1.world_point = Point::new(1.0,2.001,3.0);
        hr2.surface_normal = Vec2D::new(7.0,7.9999);
        assert!(!hr1.is_close(&hr2));

        hr1.surface_normal = Vec2D::new(7.0,7.9999);
        hr2.ray = Ray::new(
            Point::new(0.0,1.0001,-1.0),
            Vector::new(0.0, 0.0, 0.0)
        );
        assert!(!hr1.is_close(&hr2));

        hr1.ray =  Ray::new(
            Point::new(0.0,1.0001,-1.0),
            Vector::new(0.0, 0.0, 0.0)
        );
        hr2.ray = Ray::new(
            Point::new(0.0,1.0001,-1.0),
            Vector::new(0.0, 0.0, 0.0001)
        );
        assert!(!hr1.is_close(&hr2));
    }
}