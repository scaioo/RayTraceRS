//! # Raytracer Core Library
//!
//! This crate provides the foundational components of the raytracer.
//! It is organized from low-level math primitives to higher-level scene
//! and rendering structures.
//!
//! ## Modules
//!
//! - [`functions`] — general-purpose utilities and helpers
//! - [`color`] — RGB color representation and arithmetic
//! - [`geometry`] — vectors, points, normals, and geometric operations
//! - [`transformations`] — affine transformations and matrix utilities
//! - [`ray`] — ray representation and ray transformation support
//! - [`hit_record`] — intersection data returned by shape queries
//! - [`shapes`] — geometric primitives and intersection logic
//! - [`world`] — scene container and global intersection traversal
//! - [`camera`] — observer models and ray generation
//! - [`image_tracer`] — rendering pipeline that traces rays into images
//! - [`hdr_image`] — HDR image storage, tone mapping, and export
//! - [`pfm_func`] — PFM reading and conversion utilities

pub mod camera;
pub mod color;
pub mod functions;
pub mod geometry;
pub mod hdr_image;
pub mod hit_record;
pub mod image_tracer;
pub mod pfm_func;
pub mod ray;
pub mod shapes;
pub mod transformations;
mod     PCG;
pub mod world;
