// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! The `ImageTracer` module ties together the `Camera` and the `HDR` image.
//!
//! It acts as the main rendering engine. Its primary responsibilities are:
//! 1. Iterating over every pixel of the target [`HDR`] image.
//! 2. Mapping discrete pixel coordinates (columns and rows) to normalized `(u, v)`
//!    screen coordinates.
//! 3. Firing rays through the camera.
//! 4. Using a shading function to evaluate the color of each ray and saving the
//!    result back into the image.
use crate::camera::Camera;
use crate::color::Color;
use crate::hdr_image::HDR;
use crate::pcg::PCG;
use crate::ray::Ray;
use crate::world::World;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

/// The engine responsible for shooting rays through the camera and painting the image.
///
/// `ImageTracer` pairs an [`HDR`] image (the canvas) with a [`Camera`] (the observer).
/// It provides methods to shoot individual rays with sub-pixel precision, or to
/// render the entire scene automatically using a provided shading closure.
/// # Examples
///
/// ```rust,no_run
/// use rstrace::image_tracer::ImageTracer;
/// use rstrace::camera::PerspectiveCamera;
/// use rstrace::hdr_image::HDR;
/// use rstrace::transformations::Transformation;
/// use rstrace::functions::IDENTITY_4X4;
///
/// let image = HDR::new(1920, 1080);
/// let camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
/// let tracer = ImageTracer::new(image, camera, 3);
/// ```

#[derive(Clone, Debug, PartialEq)]
pub struct ImageTracer<C: Camera> {
    pub image: HDR,
    pub camera: C,
    pub n: usize, // 0 for no antialiasing!
}

impl<C: Camera> ImageTracer<C> {
    /// Creates a new `ImageTracer` binding a canvas to an observer.
    pub fn new(image: HDR, camera: C, n: usize) -> Self {
        ImageTracer { image, camera, n }
    }
    /// Fires a single ray passing through a specific pixel.
    ///
    /// The coordinates are provided as integer pixel indices (`col`, `row`) along with
    /// fractional sub-pixel offsets (`u_pixel`, `v_pixel`), which are extremely useful
    /// for anti-aliasing (firing multiple rays per pixel).
    ///
    /// # Coordinate Mapping
    /// Image coordinates (rows and columns) have their origin `(0, 0)` at the **top-left**.
    /// Camera screen coordinates `(u, v)` have their origin `(0.0, 0.0)` at the **bottom-left**.
    /// Therefore, the `v` coordinate is inverted (`1.0 - ...`) to ensure the image
    /// is not rendered upside-down.
    ///
    /// # Arguments
    /// * `col` - The column index of the 2D image (maps to the horizontal `u` coordinate / 3D Y-axis).
    /// * `row` - The row index of the 2D image (maps to the vertical `v` coordinate / 3D Z-axis).
    /// * `u_pixel` - The horizontal sub-pixel offset (typically `0.5` for the center).
    /// * `v_pixel` - The vertical sub-pixel offset (typically `0.5` for the center). for the center).
    pub fn fire_ray(&self, col: usize, row: usize, u_pixel: f32, v_pixel: f32) -> Ray {
        let u = (col as f32 + u_pixel) / (self.image.width as f32);
        let v = 1.0 - ((row as f32 + v_pixel) / (self.image.height as f32));
        self.camera.fire_ray(u, v)
    }
    /// Renders the entire image by firing rays for every pixel.
    ///
    /// This method iterates through every `(col, row)` of the target HDR image,
    /// fires a ray through the center of the pixel, and evaluates its color
    /// using the provided `func` closure.
    ///
    /// # Arguments
    /// * `world` - A reference to the 3D scene containing the shapes.
    /// * `func` - A closure (the "shader") that takes a `Ray` and the `World`
    ///   and returns the computed [`Color`] for that ray.
    ///
    /// # Errors
    /// Returns an error if the shading function fails or if setting the pixel
    /// goes out of bounds (which should mathematically never happen here).
    pub fn fire_all_rays<F>(&mut self, world: &World, mut renderer: F) -> Result<()>
    where
        // `func` takes a Ray and returns a Color (adjust return type as needed)
        F: FnMut(Ray, &World) -> Result<Color>,
    {
        // Import and initialize the toolbar tools
        let total_pixels = (self.image.width * self.image.height) as u64;
        let pb = ProgressBar::new(total_pixels);
        // Let’s give the bar a “Pro” look (Elapsed time, Bar, Percentage)
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({pos}/{len} px) - ETA: {eta_precise}"
            )?
                .progress_chars("#>-")
        );

        let mut pcg = PCG::default();
        for row in 0..self.image.height {
            for col in 0..self.image.width {
                let squares = (self.n * self.n) as f32;
                let mut color: Color = Color::default();

                if self.n == 0 {
                    let ray = self.fire_ray(col, row, 0.5, 0.5);
                    color += renderer(ray, world)?;
                } else {
                    for i in 0..self.n {
                        for j in 0..self.n {
                            let u = (i as f32 + pcg.random_float()) / self.n as f32;
                            let v = (j as f32 + pcg.random_float()) / self.n as f32;
                            let ray = self.fire_ray(col, row, u, v);
                            color += renderer(ray, world)? / squares;
                        }
                    }
                }

                self.image.set_pixel(col, row, color)?;

                // For every pixel calculated, we advance the bar by 1
                pb.inc(1);
            }
        }
        pb.finish_with_message("Rendering completed!");

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::brdf::DiffusiveBrdf;
    use crate::camera::PerspectiveCamera;
    use crate::color::Color;
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::{Point, Vector};
    use crate::materials::Material;
    use crate::pigments::UniformPigment;
    use crate::shapes::{Shape, Sphere};
    use crate::transformations::{Scaling, Transformation, Translation};

    #[test]
    fn test_image_tracer() -> Result<()> {
        let image = HDR::new(4, 2);
        let mut camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        camera.set_aspect_ratio(2.0)?;
        let tracer = ImageTracer::new(image, camera, 1);

        let ray_1 = tracer.fire_ray(0, 0, 2.5, 1.5);
        let ray_2 = tracer.fire_ray(2, 1, 0.5, 0.5);
        assert!(ray_1.is_close(ray_2));
        Ok(())
    }

    #[test]
    fn test_orientation() {
        let image = HDR::new(4, 2);
        let mut camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        camera.set_aspect_ratio(2.0).unwrap();
        let tracer = ImageTracer::new(image, camera, 1);
        let top_left_ray = tracer.fire_ray(0, 0, 0.0, 0.0);
        println!("top left: {:?}", top_left_ray.at(1.0));

        assert!(Point::new(0.0, 2.0, 1.0).is_close(&top_left_ray.at(1.0)));

        let bottom_right_ray = tracer.fire_ray(3, 1, 1.0, 1.0);
        println!("bottom right: {:?}", bottom_right_ray.at(1.0));
        assert!(Point::new(0.0, -2.0, -1.0).is_close(&bottom_right_ray.at(1.0)));
    }

    #[test]
    fn test_image_coverage() -> Result<()> {
        fn color_image(ray: Ray, world: &World) -> Result<Color> {
            let inters = world.ray_intersection(&ray);
            match inters {
                Some(_x) => {
                    let color = Color::new(1.0, 1.0, 1.0);
                    Ok(color)
                }
                None => {
                    let color = Color::new(0.0, 0.0, 0.0);
                    Ok(color)
                }
            }
        }

        fn demo_world() -> World {
            let material = Material {
                pigment: Box::new(UniformPigment::new(Color::new(10.0, 10.0, 10.0))),
                brdf: Box::new(DiffusiveBrdf {}),
                emitted_radiance: Box::new(UniformPigment::new(Color::new(10.0, 10.0, 10.0))),
            };
            let sphere_scaling : Scaling = 1.0.into();

            let central_spheres = vec![Sphere::new(
                Translation::new(Vector::new(0.0, 0.0, -0.0)) * sphere_scaling,
                material,
            )];

            let objects: Vec<Box<dyn Shape>> = central_spheres
                .into_iter()
                .map(|s| Box::new(s) as Box<dyn Shape>)
                .collect();

            World {
                objects,
                light_sources: vec![],
            }
        }
        let image = HDR::new(4, 2);
        let mut camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        camera.set_aspect_ratio(2.0)?;
        let mut tracer = ImageTracer::new(image, camera, 1);
        tracer.fire_all_rays(&demo_world(), color_image)?;

        // 2. Iterate through the tracer's image to verify the pixels
        let expected_color = Color::new(1.0, 1.0, 1.0);
        for row in 0..tracer.image.height {
            for col in 0..tracer.image.width {
                // Assuming you have a get_pixel method and Color implements PartialEq
                let pixel_color = tracer.image.get_pixel(col, row)?;

                // If Color implements `is_close`, use that. Otherwise, `assert_eq!` works if it derives `PartialEq`.
                assert_eq!(pixel_color, expected_color);
            }
        }
        Ok(())
    }
}
