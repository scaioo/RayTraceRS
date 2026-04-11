# RayTraceRS 🦀
RayTraceRS is a physically based ray tracing engine written in Rust, developed as part of the Ray Tracing course. 
The project explores core rendering concepts including ray–object intersections, materials, lighting models, 
reflections, and camera geometry, with a focus on clean design, performance, and numerical robustness.

## 🚀 Getting Started

---

### Prerequisites
Ensure you have the Rust toolchain installed.

### Installation & Run
To render a scene with full optimizations (highly recommended for raytracing):
```bash
cargo run --release
```

## 📂 Features & Directory Structure

---
The organization of the repository is meant to follow the typical rust-cargo format.

```text
RayTraceRS/
├── examples/             # [TBD] Example scenes and usage demos
├── files/                # Reference .pfm images for validation
├── src/                  # Core Source Code
│   ├── color.rs          # Color types and spectrum math
│   ├── functions.rs      # Math utilities and helper functions
│   ├── hdr_image.rs      # HDR buffer and image processing logic
│   ├── lib.rs            # Crate root and public API
│   ├── main.rs           # CLI Entry point
│   └── pfm_func.rs       # PFM file format IO handling
├── tests/                # [TBD] Integration tests
├── Cargo.toml            # Project dependencies and metadata
└── LICENSE               # MIT License
```

## 🗺️ Roadmap

---

The project is currently in the **Foundational Architecture** phase. 
Development is prioritized as follows:

- [x] **v0.1.0: Core Architecture** (Current)
    - [x] Basic Rust project structure (Library + Binary).
    - [x] HDR Image buffer and `.pfm` file IO.
    - [x] Robust `Color` and math utilities.
- [ ] **v0.2.0: Primitive Geometry** (In Progress)
    - [ ] Implement `Hittable` trait for generic object intersections.
    - [ ] Add `Sphere` and `Plane` geometry.
    - [ ] Basic Camera system with viewport mapping.

---