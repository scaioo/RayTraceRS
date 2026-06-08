// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! # Renderers
//!
//! This module defines the [`Renderer`] trait and its three implementations,
//! each representing a different strategy for computing the colour of a pixel
//! from a ray cast into a [`World`].
//!
//! ## Available renderers
//!
//! | Type | Description | Use case |
//! |---|---|---|
//! | [`OnOffRenderer`] | Binary hit/miss colouring | Scene debugging, silhouettes |
//! | [`FlatRenderer`] | Surface colour, no lighting | Material and UV debugging |
//! | [`PathTracer`] | Full Monte Carlo path tracing | Final physically-based renders |
//!

use crate::color::{BLACK, Color, WHITE};
use crate::geometry::{Dot, Vector};
use crate::pcg::PCG;
use crate::ray::Ray;
use crate::world::World;
use anyhow::Result;

/// Core trait for ray-to-colour evaluation strategies.
///
/// A `Renderer` takes a ray, a scene, and a random number generator, and
/// returns the estimated colour for that ray. Different implementations trade
/// physical accuracy for speed.
///
/// # Errors
///
/// Implementations may return an error if a [`Pigment::get_color`] call fails
/// (e.g. an [`ImagePigment`](crate::pigments::ImagePigment) with no pixels).
pub trait Renderer {
    /// Computes the colour contribution of `ray` in `world`.
    ///
    /// `pcg` is used by stochastic renderers (e.g. [`PathTracer`]) for
    /// direction sampling and Russian Roulette. Deterministic renderers
    /// (`OnOffRenderer`, `FlatRenderer`) ignore it.
    fn render(&self, ray: &Ray, world: &World, pcg: &mut PCG) -> Result<Color>;
}
// =================================================================
// OnOffRenderer
// =================================================================

/// A binary renderer that colours pixels based solely on ray–object intersection.
///
/// Pixels where the ray hits any object are painted with `color`; pixels where
/// it misses everything are painted with `background_color`. No lighting,
/// shading, or material properties are considered.
///
/// # Use case
///
/// Useful for quick scene validation: verifying object placement, checking
/// transformations, and producing silhouette renders.
#[derive(Clone, Debug, PartialEq)]
pub struct OnOffRenderer {
    /// Colour used for pixels where the ray hits an object.
    pub color: Color,
    /// Colour used for pixels where the ray misses all objects.
    pub background_color: Color,
}
impl OnOffRenderer {
    /// Creates a new `OnOffRenderer` with the given hit and background colours.
    pub fn new(color: Color, background_color: Color) -> Self {
        Self {
            color,
            background_color,
        }
    }
}
impl Default for OnOffRenderer {
    /// Returns an `OnOffRenderer` with white hits on a black background.
    fn default() -> Self {
        Self {
            color: WHITE,
            background_color: BLACK,
        }
    }
}
impl Renderer for OnOffRenderer {
    /// Returns `color` if `ray` hits any object, `background_color` otherwise.
    ///
    /// The random number generator `pcg` is not used.
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
    /// Returns a `FlatRenderer` with a black background.
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
/// A physically-based renderer that solves the rendering equation by
/// recursive Monte Carlo path tracing.
#[derive(Clone, Debug, PartialEq)]
pub struct PathTracer {
    /// Colour returned when a ray escapes the scene without hitting anything.
    pub background_color: Color,
    /// Number of shadow/scatter rays sampled per bounce (Monte Carlo samples).
    /// Higher values reduce variance (noise) at the cost of render time.
    pub n_rays: usize,
    /// Hard upper bound on ray recursion depth.
    /// Rays exceeding this depth return black, introducing a small bias.
    pub max_depth: usize,
    /// Recursion depth at which Russian Roulette begins.
    /// Set greater than `max_depth` to disable Russian Roulette entirely.
    pub depth_russian_roulette: usize,
}
impl PathTracer {
    /// Creates a new `PathTracer` with explicit control over all parameters.
    ///
    /// # Parameters
    ///
    /// - `background_color` — radiance returned for rays that escape the scene.
    /// - `n_rays` — Monte Carlo samples per bounce. `1` is typical for path
    ///   tracing (noise is reduced by averaging over many pixel samples instead).
    /// - `max_depth` — maximum number of ray bounces before forced termination.
    /// - `depth_russian_roulette` — bounce depth at which Russian Roulette
    ///   stochastic termination activates. Set above `max_depth` to disable.
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
    /// Estimates the radiance of `ray` by recursive Monte Carlo path tracing.
    ///
    /// # Errors
    ///
    /// Propagates errors from [`Pigment::get_color`](crate::pigments::Pigment::get_color)
    /// if a material's pigment or emitted radiance evaluation fails.
    fn render(&self, ray: &Ray, world: &World, pcg: &mut PCG) -> Result<Color> {
        // 1. Termination for max depth
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

// =================================================================
// Whitted Algorithm
// =================================================================
pub struct PointLightRenderer {
    background_color: Color,
}

impl PointLightRenderer {}

impl Renderer for PointLightRenderer {
    fn render(&self, ray: &Ray, world: &World, pcg: &mut PCG) -> Result<Color> {
        let hit_record = match world.ray_intersection(ray) {
            Some(hit) => hit,
            None => return Ok(self.background_color),
        };

        let mut color: Color = BLACK;

        for light_source in world.light_sources.iter() {
            color += light_source.source_contribution(&hit_record, world, pcg)?;
        }
        Ok(color)
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
    use crate::functions::{are_close, Within, IDENTITY_4X4};
    use crate::geometry::{Point, Vector, X_AXIS, Y_AXIS};
    use crate::hdr_image::HDR;
    use crate::image_tracer::ImageTracer;
    use crate::materials::Material;
    use crate::pcg::PCG;
    use crate::pigments::UniformPigment;
    use crate::shapes::{Shape, Sphere};
    use crate::transformations::{Scaling, Transformation, Translation};
    use anyhow::Result;
    use approx::assert_relative_eq;
    use crate::lexer::Keyword::SPHERE;
    use crate::light_source::{PointLightSource, SphericalLightSource};

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
        let mut tracer = ImageTracer::new(image, camera, 0);
        let world = World {
            objects: vec![Box::new(sphere)],
            light_sources: vec![],
        };

        let mut pcg = PCG::default();
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
        let mut pcg = PCG::default();
        // Setup scene & raytracer
        let image = HDR::new(3, 3);
        let camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let mut tracer = ImageTracer::new(image, camera, 0);
        let world = World {
            objects: vec![Box::new(sphere)],
            light_sources: vec![],
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
        let mut pcg = PCG::default();
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
                light_sources: vec![],
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

    fn give_sphere( point: Point, color: Color ) -> Sphere<Translation> {
        let material = Material {
            pigment: Box::new(UniformPigment::new(color)),
            brdf: Box::new(DiffusiveBrdf{}),
            emitted_radiance: Box::new(UniformPigment::new(BLACK)),
        };
        Sphere {
            transformation: Translation::new(point - Point::new(0.0,0.0,0.0)),
            material,
        }
    }

    #[test]
    fn test_point_light_renderer_two_lights_no_overlap() {
        let material = Material::new(
            UniformPigment::new(WHITE),
            DiffusiveBrdf {},
            UniformPigment::new(BLACK),
        );

        // Sphere centered at origin
        let sphere = Sphere::new(Transformation::new(IDENTITY_4X4), material);

        // Two lights on opposite sides, different colors
        let red_light  = PointLightSource::new(Point::new(-10.0, 0.0, 0.0), Color::new(1.0, 0.0, 0.0));
        let blue_light = SphericalLightSource {
            center: Point::new(10.0, 0.0, 0.0),
            radius: 1.0,
            color: Color::new(0.0,0.0,1.0),
            n_points: 1000
        };

        let world = World {
            objects: vec![Box::new(sphere)],
            light_sources: vec![Box::new(red_light), Box::new(blue_light)],
        };

        let renderer = PointLightRenderer { background_color: BLACK };
        let mut pcg = PCG::default();

        // Ray hits the sphere on the LEFT face (-X): facing red light, back to blue
        let ray_left = Ray::new(Point::new(-5.0, 0.0, 0.0), X_AXIS);
        let color_left = renderer.render(&ray_left, &world, &mut pcg).unwrap();

        // Ray hits the sphere on the RIGHT face (+X): facing blue light, back to red
        let ray_right = Ray::new(Point::new(5.0, 0.0, 0.0), - X_AXIS);
        let color_right = renderer.render(&ray_right, &world, &mut pcg).unwrap();

        // Ray misses: should return background
        let ray_miss = Ray::new(Point::new(0.0, 5.0, 0.0), Y_AXIS);
        let color_miss = renderer.render(&ray_miss, &world, &mut pcg).unwrap();

        println!("color_left:  {:?}", color_left);
        println!("color_right: {:?}", color_right);
        println!("color_miss:  {:?}", color_miss);

        // Left hit: only red contributes (n_dot_l = 1.0), blue is occluded by the sphere itself
        assert!(color_left.is_close(&Color::new(1.0, 0.0, 0.0)),  "left face: {:?}", color_left);
        // Right hit: only blue contributes (n_dot_l = 1.0), red is occluded
        assert!(are_close(color_right.r, 0.0), "right face red: {:?}", color_right.r);
        assert!(are_close(color_right.g, 0.0), "right face green: {:?}", color_right.g);
        assert!(color_right.b > 0.0, "right face blue should be positive: {:?}", color_right.b);
        // Miss: background color
        assert!(color_miss.is_close(&BLACK), "miss: {:?}", color_miss);
    }

    #[test]
    fn test_point_light_renderer_two_lights_overlap() {
        let material = Material::new(
            UniformPigment::new(WHITE),
            DiffusiveBrdf {},
            UniformPigment::new(BLACK),
        );

        let sphere = Sphere::new(Transformation::new(IDENTITY_4X4), material);

        let red_light = PointLightSource::new(Point::new(-10.0, 10.0, 0.0), Color::new(1.0, 0.0, 0.0));
        let blue_light = PointLightSource::new(Point::new(-10.0, -10.0, 0.0), Color::new(0.0, 1.0, 0.0));

        let world = World {
            objects: vec![Box::new(sphere)],
            light_sources: vec![Box::new(red_light), Box::new(blue_light)],
        };

        let renderer = PointLightRenderer { background_color: BLACK };
        let mut pcg = PCG::default();

        let ray = Ray::new(Point::new(-5.0, 0.0, 0.0), X_AXIS);
        let color = renderer.render(&ray, &world, &mut pcg).unwrap();

        // dir_to_red  = (-10,+10,0) - (-1,0,0) = (-9, +10, 0)
        // dir_to_blue = (-10,-10,0) - (-1,0,0) = (-9, -10, 0)
        // distance = sqrt(81 + 100) = sqrt(181)
        // normalized_dir_red  = (-9, +10, 0) / sqrt(181)
        // normalized_dir_blue = (-9, -10, 0) / sqrt(181)

        // normal    = (-1, 0, 0)   (outward normal at left pole)

        // normal • normalized_dir = (-1,0,0) · (-9,±10,0) / sqrt(181)
        //     = 9 / sqrt(181)

        let n_dot_l = 9.0 / 181.0_f32.sqrt();
        let expected_color = Color::new(1.0, 1.0, 0.0) * n_dot_l;
        println!("expected_color: {:?}", expected_color);
        println!("color: {:?}", color);
        assert!(color.is_close(&expected_color), "{:?}", color);
    }
}
