#  RayTraceRS

## [Unreleased]

### Added
- Introduced a unified camera system via `Camera` trait
- Added `OrthogonalCamera` and `PerspectiveCamera` implementations
- Added configurable aspect ratio support for cameras
- Added perspective distance control for `PerspectiveCamera`
- Introduced `ImageTracer` abstraction for per-pixel ray generation
- Added pixel-to-NDC mapping for ray generation in `ImageTracer`
- Added camera-based ray generation via `fire_ray(u, v)`

### Changed
- Ray generation is now fully camera-driven via `Camera::fire_ray`
- Rendering flow moved from implicit ray construction to `ImageTracer`
- Pixel sampling now uses normalized coordinates (u, v) based on image resolution
- Transformation pipeline now consistently applies homogeneous matrix transforms to rays

### Notes
- This release corresponds to the `camera` branch and introduces a major architectural refactor of the rendering system.
- Camera system replaces previous fixed-direction ray generation model.
- Perspective and orthogonal projections are now explicitly separated via distinct camera types.

---

## [0.1.0] - Initial Release

### Added
- Basic ray tracing framework
- Core geometry primitives (Point, Vector, Ray)
- Transformation system with homogeneous matrices
- Sphere-based ray intersection support
- Basic rendering pipeline with static camera direction
