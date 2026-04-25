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
    world_point: Point,
    normal: Normal,
    surface_normal: Vec2D,
    t : f32,
    ray: Ray,
}


impl HitRecord{
    pub fn is_close(&self, HR : &HitRecord)->bool{
        is_close(self.world_point, HR.world_point )
        && is_close(self.normal, HR.normal)
        && are_close(self.t, HR.t )
        // TODO: FINISH AFTER CORRECTION IN MASTER!
    }
}