# RayTraceRS 🦀

RayTraceRS is a ray tracing engine written in Rust.

The project focuses on building a clean and extensible rendering architecture,
covering core concepts such as ray-object intersections, materials, lighting,
reflections, and camera geometry.

## Current Status

The project is currently able to generate a first rendered image using basic
ray-shape intersections.

Development is prioritized as follows:

- [x] **v0.1.0 — Core Architecture**
  - [x] Basic Rust project structure (library + binary)
  - [x] `Color` and math utilities
  - [x] HDR image buffer and `.pfm` file I/O

- [x] **v0.2.0 — Primitive Geometry**
  - [x] `Point`, `Vector`, and `Normal` basic architecture
  - [x] `Transform` architecture for geometric transformations
  - [x] `Sphere`, `Plane`, and `Triangle` geometry
  - [x] Basic camera system with viewport mapping
  - [x] `World` struct to contain scene shapes
  - [x] `Ray` type with shape-intersection utilities

- [ ] **v0.3.0 — Ray Tracing Engine**
  - [ ] Implement a path tracer

## 🚀 Installation

### Prerequisites

Make sure you have the Rust toolchain installed.

### Build and Test

To run the test suite and verify that everything works:

```bash
cargo test --release
````

To build and run the project:

```bash
cargo run --release
```


## 🖥️ Command Line Interface

The project provides a small CLI for generating demo scenes, converting HDR
images, and testing the rendering pipeline.

### Global Options

These options are available for the rendering commands:
- `--width <N>`: output image width
- `--height <N>`: output image height
- `--orthogonal`: use an orthogonal camera instead of a perspective camera

### Commands

`demo``

Renders a demo scene made of multiple spheres, saves the result as a `.pfm`
image, and converts it to a `.png` preview.

```bash
cargo run --release -- demo output.pfm
```

With an orthogonal camera:

```bash
cargo run --release -- --orthogonal demo ortho_scene.pfm
```

With a custom resolution:

```bash
cargo run --release -- --width 1920 --height 1080 demo hd_scene.pfm
```

`pfm2png``

Converts a `.pfm` HDR image into an LDR image such as `.png`.

```bash
cargo run --release -- pfm2png input.pfm output.png 0.18 2.2
```

Arguments:
- `input.pfm`: input HDR image path,
- `output.png`: output image,
- `0.18`: normalization factor used for tone mapping,
- `2.2`: gamma correction value.

## ✨ Features

The current release provides the following building blocks:
- `Color` module for RGB color representation and operations
- `HDR` image module for storing image pixels
- PFM file I/O support for reading and writing `.pfm` images
- Basic geometric primitives and affine transformations
- Ray representation and ray transformation support
- Scene management through the `World` type
- Camera abstraction for perspective and orthogonal projections

### Directory Structure
The repository follows the standard Cargo project structure.

```text
RayTraceRS/
├── outputs/              # Generated outputs
├── src/                  # Core source code
│   ├── camera.rs         # Camera types and viewport mapping
│   ├── color.rs          # Color types and color math
│   ├── functions.rs      # Math utilities and helper functions
│   ├── geometry.rs       # Vector, Point, and Normal types
│   ├── hdr_image.rs      # HDR buffer and image processing logic
│   ├── lib.rs            # Crate root and public API
│   ├── main.rs           # CLI entry point
│   ├── pfm_func.rs       # PFM file format I/O handling
│   ├── ray.rs            # Ray type and ray utilities
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
- Generated images are usually stored in the output/ directory.