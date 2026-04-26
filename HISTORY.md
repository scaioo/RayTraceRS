#  RayTraceRS

## [Unreleased]

### Added
- Implemented `is_close` method for `Ray` type to facilitate robust geometric testing 
and added comprehensive unit tests for floating-point edge cases ([PR #10](https://github.com/scaioo/RayTraceRS/pull/10)).
- Introduced a unified camera system via `Camera` trait
- Added `OrthogonalCamera` and `PerspectiveCamera` implementations
- Added configurable aspect ratio support for cameras
- Added perspective distance control for `PerspectiveCamera`
- Introduced `ImageTracer` abstraction for per-pixel ray generation
- Added pixel-to-NDC mapping for ray generation in `ImageTracer`
- Added camera-based ray generation via `fire_ray(u, v)`

### Changed
- Refactored `geometry::are_close` to provide robust handling of non-finite values ([PR #10](https://github.com/scaioo/RayTraceRS/pull/10)).
- Streamlined `is_close` implementations across the codebase to follow idiomatic Rust expression-based returns.
- Ray generation is now fully camera-driven via `Camera::fire_ray`
- Rendering flow moved from implicit ray construction to `ImageTracer`
- Pixel sampling now uses normalized coordinates (u, v) based on image resolution
- Transformation pipeline now consistently applies homogeneous matrix transforms to rays

### Fixes
- Resolved potential silent failures in ray-surface intersection tests caused by floating-point rounding errors.

### Notes
- This release corresponds to the `camera` branch and introduces a major architectural refactor of the rendering system.
- Camera system replaces previous fixed-direction ray generation model.
- Perspective and orthogonal projections are now explicitly separated via distinct camera types.

### Fixed
- Fix an issue with the vertical order of the images 
([#7](https://github.com/scaioo/RayTraceRS/issues/7), [PR#8](https://github.com/scaioo/RayTraceRS/pull/8))
- Resolved CI failures on GitHub Actions caused by linting and formatting issues
- Updated codebase to pass `cargo clippy` and `cargo fmt --check`

---

## [0.1.0] - Initial Release

### Added
- Basic ray tracing framework
- Core geometry primitives (Point, Vector, Ray)
- Transformation system with homogeneous matrices
- Sphere-based ray intersection support
- Basic rendering pipeline with static camera direction
