# RayTraceRS 🦀

RayTraceRS is a physically based ray tracing engine written in Rust.

The project focuses on building a clean and extensible rendering architecture,
covering core concepts such as ray–object intersections, materials, lighting,
reflections, and camera geometry.

## 🗺️ Roadmap

The project is currently in the **Foundational Architecture** phase.
Development is prioritized as follows:

- [x] **v0.1.0: Core Architecture** *(Current)*
  - [x] Basic Rust project structure (library + binary).
  - [x] Robust `Color` and math utilities.
  - [x] HDR image buffer and `.pfm` file I/O.
- [ ] **v0.2.0: Primitive Geometry** *(In Progress)*
  - [ ] Add `Point`, `Vec`, and `Normal` basic architecture.
  - [ ] Add `Transform` architecture to handle transformations.
  - [ ] Add `Sphere` and `Plane` geometry.
  - [ ] Basic camera system with viewport mapping.

## 🚀 Getting Started

### Prerequisites
Make sure you have the Rust toolchain installed.

### Build and Run
To run the project with full optimizations, use:

```bash
cargo run --release
```

## 📂 Features & Directory Structure

### Features
The current `v0.1.0` release provides the following building blocks:

- `Color` module for RGB color representation and operations.
- `HDR image` module for storing image pixels.
- PFM file I/O support for reading and writing `.pfm` images.

### Directory Structure
The repository follows the standard Cargo project structure.

```text
RayTraceRS/
├── examples/             # Example scenes and usage demos
├── files/                # Reference .pfm images for validation
├── src/                  # Core source code
│   ├── color.rs          # Color types and spectrum math
│   ├── functions.rs      # Math utilities and helper functions
│   ├── hdr_image.rs      # HDR buffer and image processing logic
│   ├── lib.rs            # Crate root and public API
│   ├── main.rs           # CLI entry point
│   └── pfm_func.rs       # PFM file format I/O handling
├── tests/                # Integration tests
├── Cargo.toml            # Project dependencies and metadata
└── LICENSE               # MIT License
```