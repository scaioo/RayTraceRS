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
//! ## Modules
//! - [`functions`] — Small utility and helper functions
//! - [`color`] — RGB color representation and operations
//! - [`hdr_image`] — High Dynamic Range image structure and manipulation
//! - [`pfm_func`] — PFM file reading utilities

pub mod functions;
pub mod color;
pub mod hdr_image;
pub mod pfm_func;