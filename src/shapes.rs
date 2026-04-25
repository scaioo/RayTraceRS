//! Shapes module adds scene objects utilities.
//!
//! In this module we define the trait `RayIntersection` that collects all the shape the user
//! can put in the image tracer scene. Then follows the shape classes: `Sphere`, `Plane` and `Triangle`.
//!
//! All the documentation is a WIP - draft!
use crate::hit_record;


// =========================================================================
// =========================================================================
//
//
//                          IMPORTANT NOTE!!!
//
//         should we define a shape trait with all the utilities?
//
//
// =========================================================================
// =========================================================================

/// The class Sphere adds the possibility to represent spherical objects in images
///
/// Draft:
/// Sphere implements:
/// 1. `RayIntersection` trait that determines the point of intersection between
/// the ray and the sphere
/// 2. A method that returns the normal of the sphere
/// 3. A method that returns the $(u,v)$ coordinates given the point of intersection
///
/// # Note:
///
/// All of this is for the unit sphere. To obtain other pseudo-spherical objects
/// we use transformations.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere{}


/// The class Plane adds the possibility to represent the plane in an image
///
/// Draft:
/// Plane implements:
/// 1. `RayIntersection` trait that determines the point of intersection between
/// the ray and the plane
/// 2. A method that returns the normal of the plane
/// 3. A method that returns the $(u,v)$ coordinates given the point of intersection
///
/// # Note:
///
/// All of this is for the x-y plane. To obtain other pseudo-spherical objects
/// we use transformations.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane{}



/// The class Triangle adds the possibility to represent a triangle in an image
///
/// Draft:
/// Triangle implements:
/// 1. `RayIntersection` trait that determines the point of intersection between
/// the ray and the triangle
/// 2. A method that returns the normal of the triangle
/// 3. A method that returns the $(u,v)$ coordinates given the point of intersection
///
/// # Note:
///
/// Understand where to put the triangle properly for then further transformations!
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Triangle{}