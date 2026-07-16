// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Ray representation utilities.
//!
//! A [`Ray`] represents a parametrized half-line in 3D space:
//!
//! `P(t) = O + tD`
//!
//! where:
//! - `O` is the origin point
//! - `D` is the direction vector
//! - `t` is the ray parameter
//!
//! Rays also store:
//! - a valid parameter interval `[t_min, t_max]`
//! - a recursion depth used for recursive ray tracing
//!
//! Rays can be transformed through homogeneous transformations
//! such as translations, rotations, and scalings.

use crate::functions::are_close;
use crate::geometry::{Point, Vector, is_close};
use crate::transformations::{
    Scaling, Transformation, Translation, XRotation, YRotation, ZRotation,
};
use anyhow::{Result, bail};
use std::f32;
use std::fmt::{Display, Formatter};
use std::ops::Mul;
//================================================================
//                    Struct definition
//================================================================
/// Represent a light ray
///
/// # Fields
/// - `origin`: the point where the light originates
/// - `dir`: the direction of the light
/// - `t_max`: the maximum distance-parameter value
/// - `t_min`: the minimum distance-parameter value
/// - `depth`: number of recursive bounces already performed
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    /// The point where the light originates.
    pub origin: Point,
    /// The direction of propagation.
    pub dir: Vector,
    /// Maximum value of the distance parameter `t`.
    pub t_max: f32,
    /// Minimum value of the distance parameter `t`.
    pub t_min: f32,
    /// Number of recursive bounces already performed.
    pub depth: usize,
}

//================================================================
//                         Constructor
//================================================================

impl Ray {
    /// Creates a new ray with default traversal parameters.
    ///
    /// Default values:
    /// - `t_min = 1.0e-5`
    /// - `t_max = f32::INFINITY`
    /// - `depth = 0`
    ///
    /// The small positive `t_min` helps avoid self-intersections
    /// caused by floating-point inaccuracies.
    pub fn new(origin: Point, dir: Vector) -> Ray {
        Ray {
            origin,
            dir,
            t_max: f32::INFINITY,
            t_min: 1.0e-5,
            depth: 0,
        }
    }
}

//================================================================
//                         Display
//================================================================

/// Implements Display for Ray struct.
impl Display for Ray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ray:\nO: {},\nD: {},\nt: [{}, {}],\ndepth: {}",
            self.origin, self.dir, self.t_min, self.t_max, self.depth
        )
    }
}

//================================================================
//                          Methods
//================================================================

impl Ray {
    /// Sets the depth value in Ray struct.
    ///
    /// # Example
    /// ```rust
    /// use rstrace::ray::Ray;
    /// use rstrace::geometry::{Point, Vector};
    ///
    /// let mut ray = Ray::new(
    ///     Point {x: 0.0, y: 0.0, z: 0.0},
    ///     Vector {x: 1.0, y: 0.0, z: 0.0}
    /// );
    ///
    /// ray.set_depth(5);
    ///
    /// assert_eq!(ray.depth, 5);
    /// ```
    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    /// Sets t-parameter extremes for the ray's valid interval.
    ///
    /// # Errors
    /// Returns an error if `t_min` is greater than or equal to `t_max`.
    ///
    /// # Example
    /// ```rust
    /// use rstrace::ray::Ray;
    /// use rstrace::geometry::{Point, Vector};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut ray = Ray::new(
    ///     Point {x: 0.0, y: 0.0, z: 0.0},
    ///     Vector {x: 1.0, y: 0.0, z: 0.0}
    /// );
    ///
    /// // input: (t_max, t_min)
    /// ray.set_borders(1e9, 0.0)?;
    ///
    /// assert_eq!(ray.t_max, 1e9);
    /// assert_eq!(ray.t_min, 0.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_borders(&mut self, t_max: f32, t_min: f32) -> Result<()> {
        if t_min >= t_max {
            bail!(
                "Invalid ray borders: t_min ({}) must be strictly less than t_max ({}).",
                t_min,
                t_max
            );
        }
        self.t_max = t_max;
        self.t_min = t_min;
        Ok(())
    }

    /// Performs an approximate equality comparison field by field.
    ///
    /// Each comparison utilize the [`are_close`] architecture
    /// which includes special handling for infinities and `NaN`.
    pub fn is_close(self, other: Ray) -> bool {
        is_close(self.origin, other.origin)
            && is_close(self.dir, other.dir)
            && are_close(self.t_max, other.t_max)
            && are_close(self.t_min, other.t_min)
            && self.depth == other.depth
    }

    /// Evaluates the ray equation at parameter `t`.
    ///
    /// Rays are parametrized as:
    ///
    /// `P(t) = origin + t * dir`
    ///
    /// This function returns the point corresponding to the
    /// given parameter value.
    pub fn at(&self, t: f32) -> Point {
        self.origin + t * self.dir
    }
}

//================================================================
//                   Transformation operator
//================================================================
/// Implements geometric transformation of a [`Ray`].
///
/// The transformation is applied to:
/// - the ray origin as a point
/// - the ray direction as a vector
///
/// Ray traversal parameters (`t_min`, `t_max`, `depth`)
/// are preserved.
macro_rules! impl_mul_ray {
    ($name :ident) => {
        impl Mul<Ray> for $name {
            type Output = Ray;
            fn mul(self, rhs: Ray) -> Self::Output {
                Ray {
                    origin: self * rhs.origin,
                    dir: self * rhs.dir,
                    t_max: rhs.t_max,
                    t_min: rhs.t_min,
                    depth: rhs.depth,
                }
            }
        }
    };
}

impl_mul_ray!(Translation);
impl_mul_ray!(Transformation);
impl_mul_ray!(Scaling);
impl_mul_ray!(XRotation);
impl_mul_ray!(YRotation);
impl_mul_ray!(ZRotation);

//================================================================
//                         Unit Tests
//================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::are_close;

    // test constructor
    #[test]
    fn test_constructor() {
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
    fn test_display() {
        let origin = Point::new(1.0, 2.0, 3.0);
        let dir = Vector::new(4.0, 5.0, 6.0);
        let ray = Ray::new(origin, dir);

        assert_eq!(
            format!("{}", ray),
            "Ray:\nO: Point(x = 1, y = 2, z = 3),\n\
        D: Vector(x = 4, y = 5, z = 6),\nt: [0.00001, inf],\ndepth: 0"
        );
    }

    #[test]
    fn test_is_close() {
        let ray1 = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(4.0, 5.0, 6.0));
        let mut ray2 = Ray::new(
            Point::new(1.0, 1.9999999, 3.0),
            Vector::new(4.000001, 5.0, 6.0),
        );

        assert!(ray1.is_close(ray2));
        ray2.t_min = 0.0001;
        assert!(!ray1.is_close(ray2));
    }

    #[test]
    fn test_ray_set_depth() {
        let mut ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(4.0, 5.0, 6.0));
        assert_eq!(ray.depth, 0);
        ray.set_depth(4);
        assert_eq!(ray.depth, 4);
    }

    #[test]
    fn test_ray_set_borders() -> Result<()> {
        let mut ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(4.0, 5.0, 6.0));
        assert_eq!(ray.t_max, f32::INFINITY);
        assert_eq!(ray.t_min, 1e-5);

        // Correct case
        ray.set_borders(10000.0, 2.0)?;
        assert_eq!(ray.t_max, 10000.0);
        assert_eq!(ray.t_min, 2.0);

        // Error case: inverted parameters
        let err_inverted = ray.set_borders(2.0, 10000.0);
        assert!(err_inverted.is_err());

        // Error case: null interval
        let err_zero_interval = ray.set_borders(5.0, 5.0);
        assert!(err_zero_interval.is_err());

        Ok(())
    }
    #[test]
    fn test_ray_at() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(4.0, 5.0, 6.0));
        assert_eq!(is_close(ray.at(0.0), ray.origin), true);
        let expected_point = Point::new(3.0, 4.5, 6.0);
        assert_eq!(ray.at(0.5), expected_point);
        let expected_point = Point::new(13.0, 17.0, 21.0);
        assert_eq!(ray.at(3.0), expected_point);
    }

    #[test]
    fn test_translation_mul() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(4.0, 5.0, 6.0));
        let translation = Translation::new(Vector::new(10.0, 20.0, 100.0));
        let result = translation * ray;
        println!("{}", result);
        assert!(is_close(result.origin, Point::new(11.0, 22.0, 103.0)));
        assert_eq!(result.dir, ray.dir);
        assert!(result.t_max.is_infinite());
        assert!(are_close(result.t_min, 1e-5));
        assert_eq!(result.depth, ray.depth);
    }

    #[test]
    fn test_scaling_mul() {
        let ray = Ray::new(Point::new(1.0, 2.0, 3.0), Vector::new(4.0, 5.0, 6.0));
        let scaling = Scaling::new([6.0, 7.0, 8.0]);
        let result = scaling * ray;
        assert_eq!(result.origin, Point::new(1.0 * 6.0, 2.0 * 7.0, 3.0 * 8.0));
        assert_eq!(result.dir, Vector::new(4.0 * 6.0, 5.0 * 7.0, 6.0 * 8.0));
        assert!(result.t_max.is_infinite());
        assert!(are_close(result.t_min, 1e-5));
        assert_eq!(result.depth, ray.depth);
    }

    #[test]
    fn test_rotations() {
        let point = Point::new(1.0, 2.0, 3.0);
        let vector = Vector::new(4.0, 5.0, 6.0);

        let ray = Ray::new(point, vector);

        let rotation_x = XRotation::new(f32::consts::PI);
        let rotation_y = YRotation::new(f32::consts::PI / 2.0);
        let rotation_z = ZRotation::new(f32::consts::PI / 3.0);

        let result = rotation_x * ray;
        let expected_point = rotation_x * point;
        let expected_vector = rotation_x * vector;
        assert_eq!(result.origin, expected_point);
        assert_eq!(result.dir, expected_vector);
        assert!(result.t_max.is_infinite());
        assert!(are_close(result.t_min, 1e-5));
        assert_eq!(result.depth, ray.depth);

        let result = rotation_y * ray;
        let expected_point = rotation_y * point;
        let expected_vector = rotation_y * vector;
        assert_eq!(result.origin, expected_point);
        assert_eq!(result.dir, expected_vector);
        assert!(result.t_max.is_infinite());
        assert!(are_close(result.t_min, 1e-5));
        assert_eq!(result.depth, ray.depth);

        let result = rotation_z * ray;
        let expected_point = rotation_z * point;
        let expected_vector = rotation_z * vector;
        assert_eq!(result.origin, expected_point);
        assert_eq!(result.dir, expected_vector);
        assert!(result.t_max.is_infinite());
        assert!(are_close(result.t_min, 1e-5));
        assert_eq!(result.depth, ray.depth);
    }

    #[test]
    fn test_transformations_mul() {
        let point = Point::new(1.0, 2.0, 3.0);
        let vector = Vector::new(4.0, 5.0, 6.0);
        let ray = Ray::new(point, vector);
        let transformation = Scaling::new([1.0, 2.0, 3.0]) * ZRotation::new(f32::consts::PI);

        let result = transformation * ray;
        let expected_point = transformation * point;
        let expected_vector = transformation * vector;
        assert_eq!(result.origin, expected_point);
        assert_eq!(result.dir, expected_vector);
        assert_eq!(result.depth, ray.depth);
    }
}
