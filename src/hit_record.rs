//! HitRecord returns information about an intersection. (Doc Draft)
//!
//! This class contains the following informations:
//! - `world_point` : 3D point where the intersection occurred (`geometry::Point`);
//! - `normal` : surface normal at the intersection (`geometry::Normal`);
//! - `surface_point` : (u,v) coordinates of the intersection (`geometry::Vec2d`);
//! - `t`: ray parameter associated with the intersection (`f32`);
//! - `ray`: the light ray that caused the intersection (`ray::Ray`).
//!
//! Note for programmers: add `is_close`/`are_close` method for debugging.

use crate::ray::Ray;
use crate::geometry::{Point, Normal};
