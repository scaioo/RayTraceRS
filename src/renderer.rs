//! This module contains the various renderers this raytracer implements

use crate::color::{BLACK, Color, WHITE};
use crate::pcg::PCG;
use crate::ray::Ray;
use crate::world::World;
use anyhow::Result;

pub trait Renderer {
    fn render(&self, ray: &Ray, world: &World, pcg: &mut PCG) -> Result<Color>;
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
            color: WHITE,
            background_color: BLACK,
        }
    }
}
impl Renderer for OnOffRenderer {
    fn render(&self, ray: &Ray, world: &World, _pcg: &mut PCG) -> Result<Color> {
        let inters = world.ray_intersection(ray);
        match inters {
            Some(_x) => Ok(self.color),
            None => Ok(self.background_color),
        }
    }
}
// =================================================================
// FlatRenderer
// =================================================================
/// A simple renderer that computes the color of a surface without lighting.
///
/// It queries the ray intersection with the world. If an object is hit,
/// it evaluates the object's pigment at the intersection UV coordinates.
/// If nothing is hit, it returns a default background color.
#[derive(Clone, Debug, PartialEq)]
pub struct FlatRenderer {
    /// The color returned when a ray does not hit any object.
    pub background_color: Color,
}
impl FlatRenderer {
    /// Creates a new `FlatRenderer` with the specified background color
    pub fn new(background_color: Color) -> Self {
        FlatRenderer { background_color }
    }
}
impl Default for FlatRenderer {
    fn default() -> Self {
        Self {
            background_color: BLACK,
        }
    }
}
impl Renderer for FlatRenderer {
    /// Computes the color for a given ray in the given world.
    ///
    /// This method extracts the material from the `HitRecord`, queries the `Pigment`
    /// using the UV coordinates of the intersection, and returns the resulting `Color`.
    ///
    /// # Errors
    ///
    /// Propagates the error of [`get_color`].
    fn render(&self, ray: &Ray, world: &World, _pcg: &mut PCG) -> Result<Color> {
        // Find the closest intersection in the world
        match world.ray_intersection(ray) {
            Some(hit) => {
                // The ray hit an object!
                // We ask the material's pigment for the color at the specific (u, v) coordinates.
                let color = hit.material.pigment.get_color(&hit.uv)?;
                Ok(color)
            }
            None => {
                // The ray missed everything, return the background color.
                Ok(self.background_color)
            }
        }
    }
}

// =================================================================
// PathTracing
// =================================================================
#[derive(Clone, Debug, PartialEq)]
pub struct PathTracer {
    pub background_color: Color,
    pub n_rays: usize,
    pub max_depth: usize,
    pub depth_russian_roulette: usize,
}
impl PathTracer {
    pub fn new(
        background_color: Color,
        n_rays: usize,
        max_depth: usize,
        depth_russian_roulette: usize,
    ) -> Self {
        Self {
            background_color,
            n_rays,
            max_depth,
            depth_russian_roulette,
        }
    }
}
impl Renderer for PathTracer {
    fn render(&self, ray: &Ray, world: &World, pcg: &mut PCG) -> Result<Color> {
        // 1. Termination fot max depth
        if ray.depth > self.max_depth {
            return Ok(Color::new(0.0, 0.0, 0.0));
        }

        // 2. Intersection with the world
        let hit_record = match world.ray_intersection(ray) {
            Some(hit) => hit,
            None => return Ok(self.background_color),
        };

        let material = &hit_record.material;
        let mut hit_color = material.pigment.get_color(&hit_record.uv)?;
        let emitted_radiance = material.emitted_radiance.get_color(&hit_record.uv)?;

        let hit_color_lum = hit_color.r.max(hit_color.g).max(hit_color.b);

        // 3. Russian Roulette
        if ray.depth >= self.depth_russian_roulette {
            let q = 0.05_f32.max(1.0 - hit_color_lum);

            if pcg.random_float() > q {
                // Compensation of the energy for killed rays
                hit_color = hit_color * (1.0 / (1.0 - q));
            } else {
                // Anticipate termination
                return Ok(emitted_radiance);
            }
        }

        // 4. Monte Carlo Integration
        let cum_radiance = if hit_color_lum > 0.0 {
            // Iteration from 0 to n_rays
            (0..self.n_rays).try_fold(Color::new(0.0, 0.0, 0.0), |acc, _| -> Result<Color> {
                let new_ray = material.brdf.scatter_ray(
                    pcg,
                    ray.dir,
                    hit_record.world_point,
                    hit_record.normal,
                    ray.depth + 1,
                );

                // Recursive call. `?` propagate errors if present.
                let new_radiance = self.render(&new_ray, world, pcg)?;

                // Return the new value in Ok()
                Ok(acc + (hit_color * new_radiance))
            })?
        } else {
            Color::new(0.0, 0.0, 0.0)
        };

        // 5. Computation of Radiance
        let final_color = emitted_radiance + (cum_radiance / self.n_rays as f32);

        Ok(final_color)
    }
}
// =================================================================================
//                                    TESTS
// =================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::brdf::DiffusiveBrdf;
    use crate::camera::OrthogonalCamera;
    use crate::color::{BLACK, Color, WHITE};
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::{Point, Vector};
    use crate::hdr_image::HDR;
    use crate::image_tracer::ImageTracer;
    use crate::materials::Material;
    use crate::pcg::PCG;
    use crate::pigments::UniformPigment;
    use crate::shapes::Sphere;
    use crate::transformations::{Scaling, Transformation, Translation};
    use anyhow::Result;
    use approx::assert_relative_eq;
    #[test]
    fn test_on_off_renderer() -> Result<()> {
        // Define variables
        let scaling = Scaling::new([0.2, 0.2, 0.2]);
        let translation = Translation::new(Vector::new(2., 0., 0.));
        let pigment = UniformPigment::new(WHITE);
        let emitted_radiance = UniformPigment::new(BLACK);
        let brdf = DiffusiveBrdf {};
        let material = Material::new(pigment, brdf, emitted_radiance);
        let sphere = Sphere::new(translation * scaling, material);

        let image = HDR::new(3, 3);
        let camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let mut tracer = ImageTracer::new(image, camera);
        let world = World {
            objects: vec![Box::new(sphere)],
        };

        let mut pcg = PCG::new();
        let renderer = OnOffRenderer::default();

        let _ =
            tracer.fire_all_rays(&world, |ray, world| renderer.render(&ray, world, &mut pcg))?;

        assert!(
            tracer
                .image
                .get_pixel(0, 0)
                .expect("Pixel (0,0) should exist")
                .is_close(&BLACK),
            "Color mismatch at (0,0)"
        );
        assert!(
            tracer
                .image
                .get_pixel(1, 0)
                .expect("Pixel (1,0) should exist")
                .is_close(&BLACK),
            "Color mismatch at (1,0)"
        );
        assert!(
            tracer
                .image
                .get_pixel(2, 0)
                .expect("Pixel (2,0) should exist")
                .is_close(&BLACK),
            "Color mismatch at (2,0)"
        );

        assert!(
            tracer
                .image
                .get_pixel(0, 1)
                .expect("Pixel (0,1) should exist")
                .is_close(&BLACK),
            "Color mismatch at (0,1)"
        );
        assert!(
            tracer
                .image
                .get_pixel(1, 1)
                .expect("Pixel (1,1) should exist")
                .is_close(&WHITE),
            "Color mismatch at (1,1)"
        );
        assert!(
            tracer
                .image
                .get_pixel(2, 1)
                .expect("Pixel (2,1) should exist")
                .is_close(&BLACK),
            "Color mismatch at (2,1)"
        );

        assert!(
            tracer
                .image
                .get_pixel(0, 2)
                .expect("Pixel (0,2) should exist")
                .is_close(&BLACK),
            "Color mismatch at (0,2)"
        );
        assert!(
            tracer
                .image
                .get_pixel(1, 2)
                .expect("Pixel (1,2) should exist")
                .is_close(&BLACK),
            "Color mismatch at (1,2)"
        );
        assert!(
            tracer
                .image
                .get_pixel(2, 2)
                .expect("Pixel (2,2) should exist")
                .is_close(&BLACK),
            "Color mismatch at (2,2)"
        );
        Ok(())
    }
    #[test]
    fn test_flat_renderer() -> Result<()> {
        let sphere_color = Color::new(1.0, 2.0, 3.0);
        // Setup sphere and color specified
        let scaling = Scaling::new([0.2, 0.2, 0.2]);
        let translation = Translation::new(Vector::new(2., 0., 0.));
        let pigment = UniformPigment::new(sphere_color);
        let brdf = DiffusiveBrdf {};
        let emitted_radiance = UniformPigment::new(BLACK);
        let material = Material::new(pigment, brdf, emitted_radiance);
        let sphere = Sphere::new(translation * scaling, material);
        let mut pcg = PCG::new();
        // Setup scene & raytracer
        let image = HDR::new(3, 3);
        let camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let mut tracer = ImageTracer::new(image, camera);
        let world = World {
            objects: vec![Box::new(sphere)],
        };

        let renderer = FlatRenderer::default();

        let _ =
            tracer.fire_all_rays(&world, |ray, world| renderer.render(&ray, world, &mut pcg))?;

        assert!(
            tracer
                .image
                .get_pixel(0, 0)
                .expect("Pixel (0,0) should exist")
                .is_close(&BLACK),
            "Mismatch at (0,0)"
        );
        assert!(
            tracer
                .image
                .get_pixel(1, 0)
                .expect("Pixel (1,0) should exist")
                .is_close(&BLACK),
            "Mismatch at (1,0)"
        );
        assert!(
            tracer
                .image
                .get_pixel(2, 0)
                .expect("Pixel (2,0) should exist")
                .is_close(&BLACK),
            "Mismatch at (2,0)"
        );

        assert!(
            tracer
                .image
                .get_pixel(0, 1)
                .expect("Pixel (0,1) should exist")
                .is_close(&BLACK),
            "Mismatch at (0,1)"
        );
        // Verify that flat_renderer return the color of the sphere
        assert!(
            tracer
                .image
                .get_pixel(1, 1)
                .expect("Pixel (1,1) should exist")
                .is_close(&sphere_color),
            "Mismatch at (1,1)"
        );
        assert!(
            tracer
                .image
                .get_pixel(2, 1)
                .expect("Pixel (2,1) should exist")
                .is_close(&BLACK),
            "Mismatch at (2,1)"
        );

        assert!(
            tracer
                .image
                .get_pixel(0, 2)
                .expect("Pixel (0,2) should exist")
                .is_close(&BLACK),
            "Mismatch at (0,2)"
        );
        assert!(
            tracer
                .image
                .get_pixel(1, 2)
                .expect("Pixel (1,2) should exist")
                .is_close(&BLACK),
            "Mismatch at (1,2)"
        );
        assert!(
            tracer
                .image
                .get_pixel(2, 2)
                .expect("Pixel (2,2) should exist")
                .is_close(&BLACK),
            "Mismatch at (2,2)"
        );

        Ok(())
    }
    #[test]
    fn furnace_test() -> Result<()> {
        let mut pcg = PCG::new();
        // Run the furnace test several times using random values of L_e and ρ_d
        for _i in 0..5 {
            let emitted_radiance = pcg.random_float();
            let reflectance = pcg.random_float() * 0.9; // Avoid numbers that are too close to 1

            let enclosure_material = Material::new(
                UniformPigment::new(WHITE * reflectance),
                DiffusiveBrdf {},
                UniformPigment::new(WHITE * emitted_radiance),
            );
            let sphere = Sphere::new(Transformation::new(IDENTITY_4X4), enclosure_material);
            let world = World {
                objects: vec![Box::new(sphere)],
            };
            let path_tracer = PathTracer::new(WHITE, 1, 100, 101);
            let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(1., 0., 0.));
            let color = path_tracer.render(&ray, &world, &mut pcg)?;
            let expected = emitted_radiance / (1. - reflectance);

            assert_relative_eq!(color.r, expected, epsilon = 1e-3);
            assert_relative_eq!(color.g, expected, epsilon = 1e-3);
            assert_relative_eq!(color.b, expected, epsilon = 1e-3);
        }
        Ok(())
    }
}
