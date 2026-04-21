//! # Raytracer Core Library
//!
//! This crate defines the foundational components of the raytracer.
//! It currently focuses on core data structures and utilities,
//! without implementing the rendering pipeline yet.
//!
//! The provided modules cover:
//! - color representation and manipulation
//! - small mathematical utilities
//! - HDR image storage
//! - PFM (Portable Float Map) file handling
//!
//! //! // Create a new red color
// //! let red = Color::new(1.0, 0.0, 0.0);
// //!
// //! // Check that the red component is correct
// //! assert_eq!(red.r, 1.0);
// //! ```
//! ## Modules
//! - [`functions`] — Small utility and helper functions
//! - [`color`] — RGB color representation and operations
//! - [`hdr_image`] — High Dynamic Range image structure and manipulation
//! - [`pfm_func`] — PFM file reading utilities
//! - [`geometry`] - Vector, Point and Normal representation and operations
//! - [`transformations`] - Affine transformations representation and operations
//! - [`ray`] - Light ray representation and manipulation

pub mod color;
pub mod functions;
pub mod hdr_image;
pub mod pfm_func;
pub mod geometry;
pub mod transformations;
pub mod ray;
