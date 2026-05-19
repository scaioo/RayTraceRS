//! This module contains the various renderers this raytracer implements

use crate::color::Color;
use crate::ray::Ray;
use crate::world::World;

pub trait Renderer {
    fn render(&self, ray: &Ray, world: &World) -> Color;
}

// =================================================================
// OnOffRenderer
// =================================================================

/// If the ray hits something, that pixel is colored with `color`, otherwise with `background_color`.
pub struct OnOffRenderer {
    pub color: Color,
    pub background_color: Color,
}

impl OnOffRenderer {
    pub fn new(color: Color, background_color: Color) -> Self {
        Self {
            color,
            background_color,
        }
    }
}

impl Default for OnOffRenderer {
    fn default() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0),
            background_color: Color::new(0.0, 0.0, 0.0),
        }
    }
}

impl Renderer for OnOffRenderer {
    fn render(&self, ray: &Ray, world: &World) -> Color {
        let inters = world.ray_intersection(ray);
        match inters {
            Some(_x) => self.color,
            None => self.background_color,
        }
    }
}
