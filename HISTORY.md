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
- Fix an issue with sphere-ray intersections
- [#15](https://github.com/scaioo/RayTraceRS/issues/15), [PR#16](https://github.com/scaioo/RayTraceRS/pull/16).

---

## [0.1.0] - Initial Release

- First release of the code
