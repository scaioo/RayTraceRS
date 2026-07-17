//! # Raytracer Core Library
//!
//! This crate provides the foundational components of a physically-based
//! raytracer, organized from low-level math primitives up to the full
//! rendering pipeline.
//!
//! ## Architecture overview
//!
//! ```text
//!            ┌─────────────────────────────────────────────┐
//!            │                  Rendering                  │
//!            │        image_tracer  ←  renderer            │
//!            │               ↑            ↑                │
//!            │            camera         world             │
//!            │                            ↑                │
//!            │                      light_source           │
//!            ├─────────────────────────────────────────────┤
//!            │                   Scene                     │
//!            │     shapes  ←  materials  ←  pigments       │
//!            │                            ←  brdf          │
//!            │               hit_record                    │
//!            ├─────────────────────────────────────────────┤
//!            │                 Math / IO                   │
//!            │  geometry  transformations  ray  color      │
//!            │  functions  pcg  hdr_image  pfm_func        │
//!            └─────────────────────────────────────────────┘
//! ```
//!
//! ## Modules
//!
//! ### Math and primitives
//!
//! - [`color`] — RGB colour representation, arithmetic, and tone mapping.
//! - [`geometry`] — vectors, points, normals, 2D texture coordinates,
//!   and geometric operations (dot product, cross product, ONB construction).
//! - [`transformations`] — affine transformation matrices (translation,
//!   scaling, rotation) and the [`IsHomogeneousMatrix`](transformations::IsHomogeneousMatrix)
//!   trait.
//! - [`ray`] — ray representation (origin, direction, depth, `t` interval).
//! - [`functions`] — general-purpose utilities: floating-point comparison,
//!   Cramer's rule solver, and common constants.
//! - [`pcg`] — a fast PCG pseudo-random number generator used for stochastic
//!   sampling throughout the renderer.
//!
//! ### Materials
//!
//! - [`pigments`] — the [`Pigment`](pigments::Pigment) trait and implementations:
//!   uniform colour, checkerboard, HDR image texture, and gradient.
//! - [`brdf`] — the [`BRDF`](brdf::BRDF) trait and implementations:
//!   diffuse (Lambertian) and perfect specular (mirror) reflectance.
//! - [`materials`] — [`Material`](materials::Material), which bundles a pigment,
//!   a BRDF, and an emitted radiance into a single surface descriptor.
//!
//! ### Scene
//!
//! - [`shapes`] — geometric primitives ([`Sphere`](shapes::Sphere),
//!   [`Plane`](shapes::Plane), [`Triangle`](shapes::Triangle),
//!   [`AABB`](shapes::AABB), [`Cube`](shapes::Cube),
//!   [`Cylinder`](shapes::Cylinder)) and the [`Shape`](shapes::Shape) trait.
//! - [`mesh`] — [`SimpleMesh`](mesh::SimpleMesh), a triangle mesh loaded from
//!   Wavefront OBJ files. Triangles share a vertex array via [`IndexTriangle`](mesh::IndexTriangle)
//!   indices; a tight [`AABB`](shapes::AABB) provides broad-phase rejection.
//! - [`hit_record`] — [`HitRecord`](hit_record::HitRecord), the intersection
//!   data returned by shape queries (world point, normal, UV, `t`, material).
//! - [`world`] — [`World`](world::World), the scene container that holds a
//!   collection of shapes and light sources, and finds the closest ray intersection.
//! - [`light_source`] — the [`LightSource`](light_source::LightSource) trait and
//!   implementations: [`PointLightSource`](light_source::PointLightSource) (punctual
//!   emitter with shadow testing) and
//!   [`SphericalLightSource`](light_source::SphericalLightSource) (area light via
//!   Monte Carlo disk sampling).
//!
//! ### Rendering pipeline
//!
//! - [`camera`] — observer models ([`PerspectiveCamera`](camera::PerspectiveCamera),
//!   [`OrthogonalCamera`](camera::OrthogonalCamera)) and ray generation.
//! - [`renderer`] — the [`Renderer`](renderer::Renderer) trait and implementations:
//!   [`OnOffRenderer`](renderer::OnOffRenderer) (debug),
//!   [`FlatRenderer`](renderer::FlatRenderer) (unlit),
//!   [`PointLightRenderer`](renderer::PointLightRenderer) (Whitted-style direct
//!   illumination), and [`PathTracer`](renderer::PathTracer) (full Monte Carlo
//!   path tracing).
//! - [`image_tracer`] — [`ImageTracer`](image_tracer::ImageTracer), which drives
//!   the rendering loop: fires rays through every pixel and writes colours to an HDR image.
//!
//! ### Image I/O
//!
//! - [`hdr_image`] — HDR image buffer, tone mapping (Reinhard), gamma correction,
//!   and PNG export.
//! - [`pfm_func`] — PFM file format reading and byte-order handling.
//!
//! ### License
//!
//! This file is licensed under the EUPL-1.2. See LICENSE.md.

pub mod brdf;
pub mod camera;
pub mod color;
pub mod functions;
pub mod geometry;
pub mod hdr_image;
pub mod hit_record;
pub mod image_tracer;
pub mod lexer;
pub mod light_source;
pub mod materials;
pub mod mesh;
pub mod parser;
pub mod pcg;
pub mod pfm_func;
pub mod pigments;
pub mod ray;
pub mod renderer;
pub mod shapes;
pub mod transformations;
pub mod world;
