//! The camera is responsible for firing light rays into the 3D scene.
//! By default, before any transformation is applied, the camera looks down the
//! **positive X-axis**.
//!
//! The virtual screen (canvas) is placed perpendicular to the X-axis:
//! - It spans from `-aspect_ratio` to `aspect_ratio` along the Y-axis.
//! - It spans from `-1.0` to `1.0` along the Z-axis.
//!
//! The normalized coordinates `(u, v)` provided to the camera range from `0.0` to `1.0`.
//! - `u` maps the horizontal axis (Y-axis).
//! - `v` maps the vertical axis (Z-axis).

use crate::functions::are_close;
use crate::geometry::Point;
use crate::geometry::X_AXIS;
use crate::ray::Ray;
use crate::transformations::IsHomogeneousMatrix;
use anyhow::{Result, bail};
use std::ops::Mul;
// =======================================================================
// CAMERA TRAIT
// =======================================================================

/// Common interface for all camera types.
pub trait Camera {
    /// Sets the aspect ratio of the camera (width / height).
    ///
    /// # Errors
    /// Returns Err if the `aspect_ratio` is zero or negative.
    fn set_aspect_ratio(&mut self, aspect_ratio: f32) -> Result<()>;

    /// Fires a ray through the virtual screen at the normalized coordinates `(u, v)`.
    ///
    /// * `u` ranges from 0.0 (left) to 1.0 (right).
    /// * `v` ranges from 0.0 (bottom) to 1.0 (top).
    fn fire_ray(&self, u: f32, v: f32) -> Ray;
}

// =======================================================================
// ORTHOGONAL CAMERA
// =======================================================================
/// An orthogonal (orthographic) camera.
///
/// In an orthogonal camera, all rays are fired parallel to each other.
/// Objects do not appear smaller as they get further away. This is highly useful
/// for architectural rendering, engineering, or isometric games.
///
/// # Examples
///
/// ```rust,no_run
/// # use rstrace::camera::{OrthogonalCamera, Camera};
/// # use rstrace::transformations::{Transformation, Translation};
/// # use rstrace::geometry::Vector;
/// # fn main() {
/// // Move the camera 5 units back and 2 units up
/// let transform = Translation::new(Vector::new(-5.0, 2.0, 0.0));
/// let mut camera = OrthogonalCamera::new(transform);
///
/// // Set a standard widescreen aspect ratio
/// camera.set_aspect_ratio(16.0 / 9.0);
///
/// // Fire a ray straight through the center of the screen
/// let center_ray = camera.fire_ray(0.5, 0.5);
/// # }
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OrthogonalCamera<T: IsHomogeneousMatrix> {
    pub transformation: T,
    pub aspect_ratio: f32,
}

impl<T: IsHomogeneousMatrix> OrthogonalCamera<T> {
    /// Creates a new `OrthogonalCamera` with a default aspect ratio of 1.0.
    pub fn new(transformation: T) -> OrthogonalCamera<T> {
        OrthogonalCamera {
            transformation,
            aspect_ratio: 1.0,
        }
    }
}

impl<T> Camera for OrthogonalCamera<T>
where
    T: IsHomogeneousMatrix + Mul<Ray, Output = Ray> + Copy,
{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32) -> Result<()> {
        if aspect_ratio < 0.0 || are_close(aspect_ratio, 0.0) {
            bail!(
                "Invalid aspect ratio: {}. Must be strictly positive.",
                aspect_ratio
            );
        }
        self.aspect_ratio = aspect_ratio;
        Ok(())
    }
    /// Fires an orthogonal ray through the virtual screen.
    ///
    /// Unlike a perspective camera, orthogonal rays do not diverge from a single point.
    /// Instead, the ray's origin is mapped linearly across the screen, and the direction
    /// is **always strictly parallel** to the camera's local X-axis.
    ///
    /// # Mathematical Mapping
    /// - `u` is mapped from `[0.0, 1.0]` to `[-aspect_ratio, aspect_ratio]` on the Y-axis.
    /// - `v` is mapped from `[0.0, 1.0]` to `[-1.0, 1.0]` on the Z-axis.
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        // Maps `u` between [-aspect_ratio, aspect_ratio] and `v` between [-1.0, 1.0]
        let point = Point {
            x: -1.0,
            y: -self.aspect_ratio * (2.0 * u - 1.0),
            z: 2.0 * v - 1.0,
        };
        let ray = Ray {
            origin: point,
            dir: X_AXIS, // Orthogonal rays are always parallel
            t_max: f32::INFINITY,
            t_min: 1e-5,
            depth: 0,
        };
        // Apply the transformation of the camera to the generated ray
        self.transformation * ray
    }
}

// =======================================================================
// PERSPECTIVE CAMERA
// =======================================================================
/// A perspective camera.
///
/// This is the most common camera model. All rays originate from a single point
/// (the observer's eye) and diverge. Objects appear smaller as they get further away,
/// mimicking human vision and standard photography.
///
/// # Examples
///
/// ```rust,no_run
/// # use rstrace::camera::{PerspectiveCamera, Camera};
/// # use rstrace::transformations::{Transformation, YRotation};
/// # use std::f32::consts::PI;
/// # fn main() {
/// // Rotate the camera 45 degrees to look sideways
/// let transform = YRotation::new(PI / 4.0);
/// let mut camera = PerspectiveCamera::new(transform);
///
/// // Set up a widescreen cinematic view
/// camera.set_aspect_ratio(16.0 / 9.0);
///
/// // Move the screen closer to the observer's eye for a wide-angle effect (FOV)
/// camera.set_distance(0.5);
///
/// // Fire a ray towards the top-left corner of the screen
/// let top_left_ray = camera.fire_ray(0.0, 1.0);
/// # }
/// ``
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerspectiveCamera<T: IsHomogeneousMatrix> {
    pub transformation: T,
    pub aspect_ratio: f32,
    /// The distance from the camera's origin to the virtual screen.
    /// Changing this value effectively changes the Field of View (FOV).
    pub distance: f32,
}

impl<T: IsHomogeneousMatrix> PerspectiveCamera<T> {
    /// Creates a new `PerspectiveCamera` with a default aspect ratio and distance of 1.0.
    pub fn new(transformation: T) -> PerspectiveCamera<T> {
        PerspectiveCamera {
            transformation,
            aspect_ratio: 1.0,
            distance: 1.0,
        }
    }
    /// Sets the distance to the virtual screen.
    /// A smaller distance creates a wide-angle lens effect, while a larger distance
    /// creates a telephoto lens effect.
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }
}

impl<T> Camera for PerspectiveCamera<T>
where
    T: IsHomogeneousMatrix + Mul<Ray, Output = Ray> + Copy,
{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32) -> Result<()> {
        if aspect_ratio < 0.0 || are_close(aspect_ratio, 0.0) {
            bail!(
                "Invalid aspect ratio: {}. Must be strictly positive.",
                aspect_ratio
            );
        }
        self.aspect_ratio = aspect_ratio;
        Ok(())
    }
    /// Fires a perspective ray through the virtual screen.
    ///
    /// In the camera's local space, the observer's eye is located at `(-distance, 0.0, 0.0)`
    /// and looks towards the origin. The virtual screen is positioned exactly
    /// on the `YZ` plane (where `x = 0.0`).
    ///
    /// The normalized coordinates `(u, v)` are mapped to the physical screen as follows:
    /// - `u` maps horizontally along the Y-axis (from `aspect_ratio` down to `-aspect_ratio`).
    /// - `v` maps vertically along the Z-axis (from `-1.0` up to `1.0`).
    ///
    /// Finally, the generated ray is multiplied by the camera's transformation matrix
    /// to position and orient it correctly within the global World Space.
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        let point = Point {
            x: 0.0,
            y: self.aspect_ratio * (1.0 - 2.0 * u),
            z: 2.0 * v - 1.0,
        };

        let ray = Ray {
            origin: Point {
                x: -self.distance,
                y: 0.0,
                z: 0.0,
            },
            // The direction goes from the observer's eye to the point on the screen
            dir: point - Point::new(-self.distance, 0.0, 0.0),
            t_max: f32::INFINITY,
            t_min: 1e-5,
            depth: 0,
        };
        // Apply the transformation of the camera to the generated ray
        self.transformation * ray
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::{IDENTITY_4X4, equal_matrices};
    use crate::geometry::{Cross, Vector, is_close};
    use crate::transformations::{
        Scaling, Transformation, Translation, XRotation, YRotation, ZRotation,
    };
    #[test]
    fn test_orthogonal_camera() {
        let transformation = Scaling::new([1.0, 2.0, 3.0]);
        let camera = OrthogonalCamera::new(transformation);
        let mat: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(&mat, &camera.transformation.mat));
        assert_eq!(camera.aspect_ratio, 1.0);

        // Verify constructor compiles
        let _ = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let _ = OrthogonalCamera::new(XRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(YRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(ZRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = OrthogonalCamera::new(Translation::new(Vector::new(1.0, 2.0, 1.0)));
    }
    #[test]
    fn test_orthogonal_camera_transform() {
        let transformation = Translation::new(Vector::new(0.0, -2.0, 0.0))
            * ZRotation::new(std::f32::consts::FRAC_PI_2);

        let camera = OrthogonalCamera::new(transformation);
        let ray = camera.fire_ray(0.5, 0.5);

        assert!(is_close(ray.at(1.0), Point::new(0.0, -2.0, 0.0)));
    }

    #[test]
    fn test_oc_set_ar() {
        let mut orthogonal_camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        assert_eq!(orthogonal_camera.aspect_ratio, 1.0);

        // Must work (is_ok)
        assert!(orthogonal_camera.set_aspect_ratio(16.0 / 9.0).is_ok());
        assert_eq!(orthogonal_camera.aspect_ratio, 16.0 / 9.0);

        // Must be Error (is_err)
        assert!(orthogonal_camera.set_aspect_ratio(-9.0).is_err());
        assert!(orthogonal_camera.set_aspect_ratio(0.0).is_err());
    }

    #[test]
    fn test_oc_fire_ray() {
        let mut orthogonal_camera = OrthogonalCamera::new(Transformation::new(IDENTITY_4X4));
        let aspect_ratio = 16.0 / 9.0;
        orthogonal_camera.set_aspect_ratio(aspect_ratio).unwrap();
        let ray1 = orthogonal_camera.fire_ray(0.0, 0.0);
        let ray2 = orthogonal_camera.fire_ray(0.0, 1.0);
        let ray3 = orthogonal_camera.fire_ray(1.0, 0.0);
        let ray4 = orthogonal_camera.fire_ray(1.0, 1.0);
        let vec = vec![ray1, ray2, ray3, ray4];

        for i in 0..3 {
            let cross_product = vec[i].dir.cross(&vec[i + 1].dir);
            assert!(is_close(cross_product, Vector::new(0.0, 0.0, 0.0)));
        }
        assert!(ray1.at(1.0).is_close(&Point::new(0.0, aspect_ratio, -1.0)));
        assert!(ray2.at(1.0).is_close(&Point::new(0.0, aspect_ratio, 1.0)));
        assert!(ray3.at(1.0).is_close(&Point::new(0.0, -aspect_ratio, -1.0)));
        assert!(ray4.at(1.0).is_close(&Point::new(0.0, -aspect_ratio, 1.0)));
    }

    #[test]
    fn test_perspective_camera_constructor() {
        let theta = std::f32::consts::PI / 4.0;
        let cos = theta.cos();
        let sin = theta.sin();
        let mat: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let transformation = XRotation::new(theta);
        let camera = PerspectiveCamera::new(transformation);
        assert!(equal_matrices(&mat, &camera.transformation.mat));
        assert_eq!(camera.aspect_ratio, 1.0);

        // Verify constructor compiles
        let _ = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        let _ = PerspectiveCamera::new(Scaling::new([1.0, 2.0, 3.0]));
        let _ = PerspectiveCamera::new(YRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = PerspectiveCamera::new(ZRotation::new(std::f32::consts::FRAC_PI_4));
        let _ = PerspectiveCamera::new(Translation::new(Vector::new(1.0, 2.0, 1.0)));
    }

    #[test]
    fn test_perspective_camera_transformation() {
        let transformation = ZRotation::new(std::f32::consts::PI);
        let camera = PerspectiveCamera::new(transformation);
        let ray = camera.fire_ray(1.0, 0.0);

        // Default aspect_ration and distance
        println!("{:?}", ray);
        assert!(is_close(ray.at(1.0), Point::new(0.0, 1.0, -1.0)));
    }

    #[test]
    fn test_pc_set_distance() {
        let mut perspective_camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        perspective_camera.set_distance(16.0);
        assert_eq!(perspective_camera.distance, 16.0);
    }

    #[test]
    fn test_pc_set_ar() {
        let mut perspective_camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));

        assert!(perspective_camera.set_aspect_ratio(19.0).is_ok());
        assert_eq!(perspective_camera.aspect_ratio, 19.0);

        // Value very close to zero will give an Error
        assert!(perspective_camera.set_aspect_ratio(0.00000001).is_err());
        assert!(perspective_camera.set_aspect_ratio(0.0).is_err());
        assert!(perspective_camera.set_aspect_ratio(-5.0).is_err());
    }

    #[test]
    fn test_pc_fire_ray() {
        let mut perspective_camera = PerspectiveCamera::new(Transformation::new(IDENTITY_4X4));
        perspective_camera.set_aspect_ratio(2.0).unwrap();
        perspective_camera.set_distance(1.0);

        let angles = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];

        let mut rays: Vec<Ray> = Vec::with_capacity(4);

        for i in 0..4 {
            let matrix = angles[i];
            let ray = perspective_camera.fire_ray(matrix[0], matrix[1]);
            let screen = Point {
                x: 0.0,
                y: -2.0 * (2.0 * matrix[0] - 1.0),
                z: 2.0 * matrix[1] - 1.0,
            };
            let expected_vector = screen - Point::new(-1.0, 0.0, 0.0);
            assert!(is_close(expected_vector, ray.dir));
            rays.push(ray);
        }

        for i in 0..3 {
            assert!(
                is_close(rays[i].origin, rays[i + 1].origin),
                "ray{}:\n{}\nray{}:\n{}",
                i + 1,
                rays[i],
                i + 2,
                rays[i + 1]
            );

            assert!(!is_close(rays[i].dir, rays[i + 1].dir),);
        }
    }
}
