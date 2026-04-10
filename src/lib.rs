//! ## RayTraceRS - Review this text: NOT THE FINAL FORM!
//!
//! This is a Rust engine to compute raytracing simulation.
//!
//! ### Basic Usage
//! ```rust
//! use RayTraceRS::color::Color;
//!
//! // Create a new red color
//! let red = Color::new(1.0, 0.0, 0.0);
//!
//! // Check that the red component is correct
//! assert_eq!(red.r, 1.0);
//! ```

pub mod color;
pub mod functions;
pub mod hdr_image;
pub mod pfm_func;
