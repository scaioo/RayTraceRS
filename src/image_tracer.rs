use crate::camera::Camera;
use crate::color::Color;
use crate::hdr_image::HDR;
use crate::ray::Ray;
use anyhow::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct ImageTracer<C: Camera> {
    pub image: HDR,
    pub camera: C,
}

impl<C: Camera> ImageTracer<C> {
    pub fn new(image: HDR, camera: C) -> Self {
        ImageTracer { image, camera }
    }
    pub fn fire_ray(&self, col: usize, row: usize, u_pixel: f32, v_pixel: f32) -> Ray {
        // -1!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        let u = (col as f32 + u_pixel) / (self.image.width as f32);
        let v = 1.0 - ((row as f32 + v_pixel) / (self.image.height as f32));
        self.camera.fire_ray(u, v)
    }
    pub fn fire_all_rays<F>(&mut self, func: F) -> Result<()>
    where
        // `func` takes a Ray and returns a Color (adjust return type as needed)
        F: Fn(Ray) -> Result<Color>,
    {
        for row in 0..self.image.height {
            for col in 0..self.image.width {
                // Using 0.5 as the default pixel offsets like in Python
                let ray = self.fire_ray(col, row, 0.5, 0.5);

                let color = func(ray)?;

                self.image
                    .set_pixel(col, row, color)
                    .expect("TODO: panic message");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::PerspectiveCamera;
    use crate::color::Color;
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::Point;
    use crate::transformations::Transformation;

    #[test]
    fn test_image_tracer() -> Result<()> {
        let image = HDR::new(4, 2);
        let mut camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        camera.set_aspect_ratio(2.0);
        let tracer = ImageTracer::new(image, camera);

        let ray_1 = tracer.fire_ray(0, 0, 2.5, 1.5);
        let ray_2 = tracer.fire_ray(2, 1, 0.5, 0.5);
        assert!(ray_1.is_close(ray_2));
        Ok(())
    }

    #[test]
    fn test_orientation() {
        let image = HDR::new(4, 2);
        let mut camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        camera.set_aspect_ratio(2.0);
        let tracer = ImageTracer::new(image, camera);
        let top_left_ray = tracer.fire_ray(0, 0, 0.0, 0.0);
        println!("top left: {:?}", top_left_ray.at(1.0));

        assert!(Point::new(0.0, 2.0, 1.0).is_close(&top_left_ray.at(1.0)));

        let bottom_right_ray = tracer.fire_ray(3, 1, 1.0, 1.0);
        println!("bottom right: {:?}", bottom_right_ray.at(1.0));
        assert!(Point::new(0.0, -2.0, -1.0).is_close(&bottom_right_ray.at(1.0)));
    }

    #[test]
    fn test_image_coverage() -> Result<()> {
        let image = HDR::new(4, 2);
        let mut camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        camera.set_aspect_ratio(2.0);
        let mut tracer = ImageTracer::new(image, camera);

        tracer.fire_all_rays(|_ray| Ok(Color::new(1.0, 2.0, 3.0)))?;

        // 2. Iterate through the tracer's image to verify the pixels
        let expected_color = Color::new(1.0, 2.0, 3.0);
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
