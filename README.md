# RayTraceRS 🦀

RayTraceRS is a ray tracing engine written in Rust.

The project focuses on building a clean and extensible rendering architecture,
covering core concepts such as ray-object intersections, materials, lighting,
reflections, and camera geometry.

## Current Status

RayTraceRS is currently capable of rendering physically based images using a full Monte Carlo path tracer.

The engine already supports:

* Diffuse and perfect-specular materials
* Procedural pigments and HDR image textures
* Perspective and orthogonal cameras
* Multiple rendering algorithms selectable at runtime
* Recursive light transport through path tracing

The project is actively developed, with the following features planned for the first stable release.

### Roadmap to v1.0.0

* [ ] Scene file lexer
* [ ] Scene interpreter and parser
* [ ] Constructive Solid Geometry (CSG)
* [ ] Triangle and quadrilateral mesh support
* [ ] Antialiasing
* [ ] Additional example scenes and documentation

## 🚀 Installation

### Prerequisites

Make sure you have the Rust toolchain installed.

### Build and Test

To run the test suite and verify that everything works:

```
cargo test --release
```

To build the project:

```
cargo build --release
```

## 🖥️ Command Line Interface

The project provides a small CLI for generating demo scenes, converting HDR
images, and testing the rendering pipeline.

### Global Options

These options are available for the rendering commands:

- `--width <N>`: output image width
- `--height <N>`: output image height
- `--orthogonal`: use an orthogonal camera instead of a perspective camera
- `--format <T>`: choose the file format you want to save the first converted image into


### Commands

`demo`

Renders a demo scene made of multiple spheres and a chequered floor,
saves the result as a `.pfm` image, and converts it to a `.png` preview.

```
cargo run demo output
```

With a specific rendering algorithm (`onoff`, `flat`, or `pathtracing`):

```
cargo run demo output --algorithm flat
```

With custom path-tracing parameters:

```
cargo run demo output --num-of-rays 20 --max-depth 5
```

With an orthogonal camera:

```
cargo run -- --orthogonal demo ortho_scene
```

With a custom resolution:

```
cargo run -- --width 1920 --height 1080 demo hd_scene
```

Additional `demo` options:

- `--angle-deg <DEG>`: rotate the camera around the Z-axis (default: `0.0`).
- `--algorithm <ALG>`: rendering algorithm — `onoff`, `flat`, or `pathtracing` (default: `pathtracing`).
- `--num-of-rays <N>`: number of secondary rays per bounce for the path tracer (default: `10`).
- `--max-depth <N>`: maximum ray recursion depth (default: `3`).

`pfm2png`

Converts a `.pfm` HDR image into an LDR image such as `.png`.

```
cargo run pfm2png input.pfm output.png 0.18 2.2
```

Arguments:

- `input.pfm`: input HDR image path,
- `output.png`: output image,
- `0.18`: normalization factor used for tone mapping,
- `2.2`: gamma correction value.


## ✨ Features

The current release provides the following building blocks:

- `Color` module for RGB color representation and operations
- `HDR` image module for storing image pixels and bilinear interpolation
- PFM file I/O support for reading and writing `.pfm` images
- Basic geometric primitives and affine transformations
- `PCG` pseudo-random number generator for stochastic sampling
- `Pigment` trait with `UniformPigment`, `CheckeredPigment`, `ImagePigment`, and `GradientPigment`
- `BRDF` trait with diffusive (`DiffusiveBrdf`) and mirror (`SpecularBrdf`) reflectance models
- `Material` type bundling pigment, BRDF, and self-emitted radiance
- Ray representation and ray transformation support
- Scene management through the `World` type
- Camera abstraction for perspective and orthogonal projections
- `OnOffRenderer` (debug), `FlatRenderer` (unlit), and `PathTracer` (Monte Carlo path tracing)
- Progress bar during rendering via `indicatif`


### Directory Structure

The repository follows the standard Cargo project structure.

```
RayTraceRS/
├── outputs/              # Generated outputs
├── src/                  # Core source code
│   ├── brdf.rs           # BRDF trait and implementations (diffuse, specular)
│   ├── camera.rs         # Camera types and viewport mapping
│   ├── color.rs          # Color types and color math
│   ├── functions.rs      # Math utilities and helper functions
│   ├── geometry.rs       # Vector, Point, Normal types and ONB construction
│   ├── hdr_image.rs      # HDR buffer and image processing logic
│   ├── hit_record.rs     # Ray-surface intersection data
│   ├── image_tracer.rs   # Rendering loop and pixel traversal
│   ├── lib.rs            # Crate root and public API
│   ├── main.rs           # CLI entry point
│   ├── materials.rs      # Material type (pigment + BRDF + emission)
│   ├── pcg.rs            # PCG pseudo-random number generator
│   ├── pfm_func.rs       # PFM file format I/O handling
│   ├── pigments.rs       # Pigment trait and texture implementations
│   ├── ray.rs            # Ray type and ray utilities
│   ├── renderer.rs       # Renderer trait and rendering algorithms
│   ├── shapes.rs         # Scene geometry and intersections
│   ├── transformations.rs# Affine transformation utilities
│   └── world.rs          # Scene container and ray traversal
├── tests/                # Integration tests
├── Cargo.toml            # Project dependencies and metadata
├── README.md             # ReadMe file
└── LICENSE.md            # License file

```

## 📌 Notes

- The project is still in active development.
- Some modules are intentionally kept simple while the rendering pipeline is being expanded.
- Generated images are usually stored in the `outputs/` directory.