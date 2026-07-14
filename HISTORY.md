## Head

- Improve pfm-ldr/render CLI: rename the command, make factor-a/gamma options, scope --format
to render ([PR#35](https://github.com/scaioo/RayTraceRS/pull/35).
- Fix `GradientPigment` and `bilinear_interpolation` handling of negative coordinates 
([#33](https://github.com/scaioo/RayTraceRS/issues/33), [PR#34](https://github.com/scaioo/RayTraceRS/pull/34)).
- Add `--threads` CLI flag to control rendering thread count
  ([PR#32](https://github.com/scaioo/RayTraceRS/pull/32)).
- Parallelize rendering across CPU cores with `rayon`
  ([PR#32](https://github.com/scaioo/RayTraceRS/pull/32)).
- Add `--reflectance-policy {reject,rescale,ignore}` CLI flag to control how materials
  with out-of-range reflectance are handled
  ([PR#31](https://github.com/scaioo/RayTraceRS/pull/31)).
- Validate material reflectance while parsing scene files, reporting the material name
  and source location on failure
  ([PR#31](https://github.com/scaioo/RayTraceRS/pull/31)).
- Make `Material::new` return `Result` and reject pigments with invalid reflectance
  ([#30](https://github.com/scaioo/RayTraceRS/issues/30), [PR#31](https://github.com/scaioo/RayTraceRS/pull/31)).
- Fix `GradientPigment` so a rotated gradient stays within `[color1, color2]` instead of
  extrapolating past it (changes the rendered output of existing rotated gradients)
  ([PR#31](https://github.com/scaioo/RayTraceRS/pull/31)).
- Add `Color::validate_reflectance` and `Pigment::validate_reflectance` to check that
  reflectance colors stay within `[0,1]`
  ([#30](https://github.com/scaioo/RayTraceRS/issues/30), [PR#31](https://github.com/scaioo/RayTraceRS/pull/31)).
- Add Scene Interpreter ([PR#23](https://github.com/scaioo/RayTraceRS/pull/23)).
- Add `load_from_ldr` to `HdrImage` to support PNG/JPEG textures via
  inverse gamma correction and inverse tone mapping
  ([PR#29](https://github.com/scaioo/RayTraceRS/pull/29)).
- Implement Whitted point-light renderer with `PointLightSource` and `SphericalLightSource`
  ([PR#28](https://github.com/scaioo/RayTraceRS/pull/28)).
- Add `SimpleMesh` with OBJ loading via `tobj` 
  ([PR#26](https://github.com/scaioo/RayTraceRS/pull/26)).
- Add `IndexTriangle` for compact index-based triangle connectivity
  ([PR#26](https://github.com/scaioo/RayTraceRS/pull/26)).
- Add `AABB` (axis-aligned bounding box) with slab-method intersection and `contains` check
  ([PR#26](https://github.com/scaioo/RayTraceRS/pull/26)).
- Add `BRDFs` enum for lightweight BRDF variant tagging
  ([PR#26](https://github.com/scaioo/RayTraceRS/pull/26)).
- Add `IDENTITY_TRANSFORMATION` constant
  ([PR#26](https://github.com/scaioo/RayTraceRS/pull/26)).
- Add `Copy` supertrait bound to `IsHomogeneousMatrix`
  ([PR#26](https://github.com/scaioo/RayTraceRS/pull/26)).
- Fix an issue with UV coordinate orientation
  ([#25](https://github.com/scaioo/RayTraceRS/issues/25), [PR#27](https://github.com/scaioo/RayTraceRS/pull/27)).
- Implement Antialiasing ([PR#22](https://github.com/scaioo/RayTraceRS/pull/22)).
- Build lexer ([PR#18](https://github.com/scaioo/RayTraceRS/pull/18)).

## [0.3.0] - Path Tracing Engine


- Update demo scene with sky dome, chequered floor, diffuse sphere, and mirror sphere
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Extend `demo` CLI command with `--algorithm`, `--num-of-rays`, and `--max-depth` flags
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Implement `OnOffRenderer`, `FlatRenderer`, and `PathTracer`
  (Monte Carlo path tracing with furnace test)
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Add progress bar to the rendering loop via `indicatif`
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Extend `Shape` and `HitRecord` to carry `Material` references
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Implement `Clone` for `Material` and `World` via the supertrait mechanism
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Add `Material` type bundling pigment, BRDF, and emitted radiance
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Add `BRDF` trait with `DiffusiveBrdf` (Lambertian) and `SpecularBrdf` (mirror)
  implementations ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Add `branchless_onb` to `geometry` for orthonormal basis construction
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Implement `PCG` (Permuted Congruential Generator) pseudo-random number generator
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Add `Pigment` trait with `UniformPigment`, `CheckeredPigment`, `ImagePigment`,
  and `GradientPigment` implementations
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).
- Add `World::add` via the `Add` trait to merge scene collections
  ([PR#13](https://github.com/scaioo/RayTraceRS/pull/13)).

---

## [0.2.0] - demo command

- Implement `demo` command.
- Change license to
  [EUPL](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).
- Fix an issue with the vertical order of the images
  ([#7](https://github.com/scaioo/RayTraceRS/issues/7), [PR#8](https://github.com/scaioo/RayTraceRS/pull/8)).
- Fix an issue with sphere-ray intersections [#15](https://github.com/scaioo/RayTraceRS/issues/15), [PR#16](https://github.com/scaioo/RayTraceRS/pull/16).

---

## [0.1.0] - Initial Release

- First release of the code
