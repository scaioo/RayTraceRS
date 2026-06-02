//! # Shapes
//!
//! This module defines the geometric primitives that can be placed in a ray-tracer scene.
//!
//! ## Structure
//!
//! - [`Shape`] — the core trait every scene object must implement.
//! - [`Sphere`] — a unit sphere, transformed via an [`IsHomogeneousMatrix`].
//! - [`Plane`] — the xy-plane, transformed via an [`IsHomogeneousMatrix`].
//! - [`Triangle`] — an arbitrary triangle defined by three world-space vertices.
//!
//! All shapes return a [`HitRecord`] on intersection, containing the world-space hit point,
//! surface normal, UV coordinates, ray parameter `t`, and a reference to the shape's [`Material`].
//!
//! ## Design note
//!
//! Sphere and Plane are generic over any homogeneous transformation `T` (translation, scaling,
//! rotation, or composed). Triangle operates directly in world space and does not accept a
//! transformation parameter; apply transformations to its vertices before construction.

use crate::functions::{Within, are_close, cramer};
use crate::geometry::{Cross, Dot, Normal, Point, Vec2D, Vector};
use crate::hit_record::HitRecord;
use crate::materials::Material;
use crate::pigments::ClonePigment;
use crate::ray::Ray;
use crate::transformations::IsHomogeneousMatrix;
use anyhow::{Result, anyhow};
use std::ops::Mul;
// ========================================================
// Supertrait CloneShape
// ========================================================

/// Helper supertrait that makes `Box<dyn Shape>` cloneable.
///
/// You never need to implement this manually. The blanket `impl` below
/// provides it automatically for any type that implements `Shape + Clone`.
pub trait CloneShape {
    fn clone_shape(&self) -> Box<dyn Shape>;
}

/// Blanket implementation: any `T: Shape + Clone + 'static` gets
/// [`CloneShape`] for free by boxing a normal `.clone()` call.
impl<T> CloneShape for T
where
    T: Shape + Clone + 'static,
{
    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

/// Core trait for ray-intersectable scene objects.
///
/// Every shape placed in the scene must implement this trait. The four methods together
/// provide all information the renderer needs to compute lighting at a surface point.
pub trait Shape: CloneShape {
    /// Tests whether `ray` intersects this shape.
    ///
    /// Returns the closest valid [`HitRecord`] within `[ray.t_min, ray.t_max]`,
    /// or `None` if no intersection exists in that range.
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>>;

    /// Returns the outward surface normal at `point`, oriented against `ray`.
    ///
    /// The normal is flipped so that it always points toward the incoming ray origin,
    /// i.e. `normal · ray.dir < 0`.
    ///
    /// # Note
    /// `point` must lie on the surface of the shape. Behaviour is undefined otherwise.
    fn normal_at(&self, point: Point, ray: &Ray) -> Normal;

    /// Maps a surface point to UV texture coordinates in `[0,1)²`.
    ///
    /// Returns an error if `point` does not lie on (or sufficiently near) the surface.
    fn point_to_uv(&self, point: &Point) -> Result<Vec2D>;

    /// Returns a reference to the [`Material`] assigned to this shape.
    fn material(&self) -> &Material;
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Box<dyn Shape> {
        self.clone_shape()
    }
}

// =================================================================================
/// A unit sphere centered at the origin, subject to a homogeneous transformation.
///
/// The sphere is defined implicitly as `x² + y² + z² = 1` in its local (object) space.
/// Any ellipsoid, oblate sphere, or translated sphere can be obtained by composing
/// an appropriate [`IsHomogeneousMatrix`] transformation.
///
/// # UV mapping
///
/// Surface coordinates follow the standard spherical parametrisation:
/// - `u = φ / 2π ∈ [0, 1]` — longitude (azimuthal angle around the z-axis)
/// - `v = θ / π ∈ [0, 1]` — colatitude (polar angle from the +z pole)
#[derive(Clone)]
pub struct Sphere<T: IsHomogeneousMatrix> {
    /// The world-from-object transformation applied to the unit sphere.
    pub transformation: T,
    /// Surface material (pigment + BRDF + emitted radiance).
    pub material: Material,
}

impl<T: IsHomogeneousMatrix> Sphere<T> {
    pub fn new(transformation: T, material: Material) -> Self {
        Self {
            transformation,
            material,
        }
    }
}
impl<T> Shape for Sphere<T>
where
    T: IsHomogeneousMatrix
        + Mul<Ray, Output = Ray>
        + Mul<Point, Output = Point>
        + Mul<Normal, Output = Normal>
        + Mul<Vector, Output = Vector>
        + Copy
        + 'static,
{
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        let inverse_transformation = self.transformation.inverse_transformation();
        let transformed_ray = inverse_transformation * (*ray);

        let origin = transformed_ray.origin - Point::new(0.0, 0.0, 0.0);

        let a = transformed_ray.dir.squared_norm();
        let half_b = origin.dot(&transformed_ray.dir);
        let cross = transformed_ray.dir.cross(&origin);

        let discriminant = a - cross.squared_norm();

        if discriminant < 0.0 || are_close(discriminant, 0.0) {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let t1 = (-half_b - sqrt_d) / a;
        let t2 = (-half_b + sqrt_d) / a;

        let condition = |t: f32| t > transformed_ray.t_min && t < transformed_ray.t_max;

        let t = if condition(t1) {
            t1
        } else if condition(t2) {
            t2
        } else {
            return None;
        };

        let hit_point = transformed_ray.at(t);
        let uv = self.point_to_uv(&hit_point).ok()?;

        Some(HitRecord {
            world_point: self.transformation * hit_point,
            normal: self.transformation * self.normal_at(hit_point, &transformed_ray),
            uv,
            t,
            ray: *ray,
            material: &self.material,
        })
    }

    fn normal_at(&self, point: Point, ray: &Ray) -> Normal {
        let result = Normal::new(point.x, point.y, point.z);
        let vector = point - Point::new(0.0, 0.0, 0.0);
        if (vector.dot(&ray.dir)) < 0.0 {
            result
        } else {
            -result
        }
    }

    fn point_to_uv(&self, point: &Point) -> Result<Vec2D> {
        let pi = std::f32::consts::PI;
        let mut u = point.y.atan2(point.x) / (2.0 * pi);
        if u < 0.0 {
            u += 1.0;
        }

        let v = point.z.clamp(-1.0, 1.0).acos() / pi;

        Ok(Vec2D { x: u, y: v })
    }

    fn material(&self) -> &Material {
        &self.material
    }
}
// =================================================================================
/// The infinite xy-plane (`z = 0` in object space), subject to a homogeneous transformation.
///
/// The canonical plane is `z = 0`; normals point along ±z and are oriented against the
/// incoming ray. Any tilted or elevated plane can be obtained via transformation.
///
/// # UV mapping
///
/// Two modes are available, selected by `procedural_texture`:
/// - **Tiled** (`false`): `u = frac(x)`, `v = frac(y)` — repeats the texture every unit square.
/// - **Procedural** (`true`): `u = x`, `v = y` — raw world coordinates, useful for
///   procedural patterns that must be scale-aware.
#[derive(Clone)]
pub struct Plane<T: IsHomogeneousMatrix> {
    /// The world-from-object transformation applied to the canonical xy-plane.
    pub transformation: T,
    /// Surface material (pigment + BRDF + emitted radiance).
    pub material: Material,
    /// If `true`, UV coordinates are raw world-space `(x, y)` rather than tiled fractions.
    pub procedural_texture: bool,
}
impl<T: IsHomogeneousMatrix> Plane<T> {
    pub fn new(transformation: T, material: Material, procedural_texture: bool) -> Self {
        Self {
            transformation,
            material,
            procedural_texture,
        }
    }
}
impl<T> Shape for Plane<T>
where
    T: IsHomogeneousMatrix
        + Mul<Ray, Output = Ray>
        + Mul<Point, Output = Point>
        + Mul<Normal, Output = Normal>
        + Mul<Vector, Output = Vector>
        + 'static
        + Copy,
{
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        let inverse_transformation = self.transformation.inverse_transformation();
        let transformed_ray = inverse_transformation * (*ray);

        if are_close(transformed_ray.dir.z, 0.0) {
            return None;
        }
        let t = -transformed_ray.origin.z / transformed_ray.dir.z;

        if t <= transformed_ray.t_min || t >= transformed_ray.t_max {
            return None;
        }
        let hit_point = transformed_ray.at(t);
        let uv = self.point_to_uv(&hit_point).ok()?;

        Some(HitRecord {
            world_point: self.transformation * hit_point,
            normal: self.transformation * self.normal_at(hit_point, &transformed_ray),
            uv,
            t,
            ray: *ray,
            material: &self.material,
        })
    }

    fn normal_at(&self, _point: Point, ray: &Ray) -> Normal {
        let result = Normal::new(0.0, 0.0, 1.0);
        if ray.dir.z > 0.0 { -result } else { result }
    }

    fn point_to_uv(&self, point: &Point) -> Result<Vec2D> {
        if self.procedural_texture {
            Ok(Vec2D {
                x: point.x,
                y: point.y,
            })
        } else {
            Ok(Vec2D {
                x: point.x - point.x.floor(),
                y: point.y - point.y.floor(),
            })
        }
    }

    fn material(&self) -> &Material {
        &self.material
    }
}
// =================================================================================
/// A triangle defined by three world-space vertices, with flat shading.
///
/// Intersection is computed via Cramer's rule applied to the barycentric coordinate system.
/// The UV coordinates of a hit point are its barycentric coordinates `(β, γ)`, so
/// `u = β`, `v = γ`, with `α = 1 − β − γ` implicitly giving the weight of vertex `a`.
///
/// # Winding order and normals
///
/// The outward normal is `(b − a) × (c − a)`. It is **not** normalized; its magnitude
/// equals the area of the parallelogram spanned by the two edges. Ensure your lighting
/// model accounts for this, or normalize in [`Shape::normal_at`] if needed.
///
/// # Note on transformations
///
/// Unlike [`Sphere`] and [`Plane`], `Triangle` has no generic transformation parameter.
/// To place a triangle in an arbitrary position, transform the vertices `a`, `b`, `c`
/// before passing them to [`Triangle::new`].
#[derive(Clone)]
pub struct Triangle {
    /// First vertex.
    pub a: Point,
    /// Second vertex.
    pub b: Point,
    /// Third vertex.
    pub c: Point,
    /// Surface material (pigment + BRDF + emitted radiance).
    pub material: Material,
}
//                           For triangle implementation
impl Triangle {
    /// Creates a new triangle from three world-space vertices and a material.
    pub fn new(a: Point, b: Point, c: Point, material: Material) -> Self {
        Self { a, b, c, material }
    }

    /// Solves the ray–triangle intersection using Cramer's rule.
    ///
    /// Returns `(t, β, γ)` where:
    /// - `t` is the ray parameter of the hit point,
    /// - `β`, `γ` are the barycentric coordinates with respect to vertices `b` and `c`,
    /// - the coordinate for vertex `a` is `α = 1 − β − γ`.
    ///
    /// The intersection is valid only if `β ∈ (0,1)`, `γ ∈ (0,1)`, and `β + γ ≤ 1`.
    /// Border points (`β = 0`, `γ = 0`, `β + γ = 1`) are excluded.
    ///
    /// Returns `Err` if the ray misses the triangle or is coplanar with it.
    pub fn intersection(&self, ray: Ray) -> Result<(f32, f32, f32)> {
        let mat: [f32; 9] = [
            self.b.x - self.a.x,
            self.c.x - self.a.x,
            -ray.dir.x,
            self.b.y - self.a.y,
            self.c.y - self.a.y,
            -ray.dir.y,
            self.b.z - self.a.z,
            self.c.z - self.a.z,
            -ray.dir.z,
        ];
        let right_member = [
            ray.origin.x - self.a.x,
            ray.origin.y - self.a.y,
            ray.origin.z - self.a.z,
        ];

        let result = cramer(&mat, right_member)?;

        let beta = result[0];
        let gamma = result[1];
        let t = result[2];

        // We ignore borders
        if beta.is_between_open(&0.0, &1.0)
            && gamma.is_between_open(&0.0, &1.0)
            && (beta + gamma < 1.0 || are_close(beta + gamma, 1.0))
        {
            Ok((t, beta, gamma))
        } else {
            Err(anyhow!(
                "No Ray-Triangle intersection!!\nBeta: {}\nGamma: {}",
                beta,
                gamma
            ))
        }
    }
}
impl Shape for Triangle {
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        let (t, beta, gamma) = self.intersection(*ray).ok()?;

        if t.is_between_open(&ray.t_min, &ray.t_max) {
            let hit_point = ray.at(t);
            Some(HitRecord {
                world_point: hit_point,
                normal: self.normal_at(hit_point, ray),
                uv: Vec2D::new(beta, gamma),
                t,
                ray: *ray,
                material: &self.material,
            })
        } else {
            None
        }
    }
    fn normal_at(&self, _point: Point, ray: &Ray) -> Normal {
        let result = (self.b - self.a).cross(&(self.c - self.a));
        let result = Normal {
            x: result.x,
            y: result.y,
            z: result.z,
        };

        if ray.dir.dot(&result) > 0.0 {
            -result
        } else {
            result
        }
    }
    fn point_to_uv(&self, point: &Point) -> Result<Vec2D> {
        let normal = (self.b - self.a).cross(&(self.c - self.a));
        let origin = *point - normal;
        let ray = Ray::new(origin, normal);

        let (_, beta, gamma) = self.intersection(ray)?;

        Ok(Vec2D { x: beta, y: gamma })
    }
    fn material(&self) -> &Material {
        &self.material
    }
}

// =================================================================================

// =================================================================================
//
//                                    TESTS
//
// =================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brdf::DiffusiveBrdf;
    use crate::color::Color;
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::{X_AXIS, is_close};
    use crate::pigments::UniformPigment;
    use crate::transformations::{Scaling, Transformation, Translation};

    fn setup1() -> (Sphere<Transformation>, [Ray; 3]) {
        let rays = [
            Ray::new(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0)),
            Ray::new(Point::new(3.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0)),
            Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)),
        ];

        let transformation = Transformation::new(IDENTITY_4X4);
        let sphere = Sphere::new(transformation, Material::default());
        (sphere, rays)
    }

    #[test]
    fn test_sphere_ray_point_intersection1() {
        let (sphere, rays) = setup1();

        let points: [Point; 3] = [
            Point::new(0.0, 0.0, 1.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
        ];

        for i in 0..3 {
            let hit_record = sphere
                .ray_intersection(&rays[i])
                .expect("ray_intersection IS WRONG! (Expected a hit)");
            assert!(
                is_close(hit_record.world_point, points[i]),
                "Error occurred: index {} is responsible.\n",
                i
            );
        }
    }

    #[test]
    fn test_sphere_uv_point_intersection1() {
        let (sphere, rays) = setup1();

        let uv_points: [Vec2D; 3] = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.0, 0.5),
            Vec2D::new(0.0, 0.5),
        ];

        for i in 0..3 {
            let hit_record = sphere
                .ray_intersection(&rays[i])
                .expect("ray_intersection IS WRONG! (Expected a hit)");
            assert!(
                hit_record.uv.is_close(&uv_points[i]),
                "Error occurred: index {} is responsible.\n",
                i
            );
        }
    }

    #[test]
    fn test_sphere_ray_normal_att1() {
        let (sphere, rays) = setup1();

        let normals: [Normal; 3] = [
            Normal::new(0.0, 0.0, 1.0),
            Normal::new(1.0, 0.0, 0.0),
            Normal::new(-1.0, 0.0, 0.0),
        ];

        for i in 0..3 {
            let hit_record = sphere
                .ray_intersection(&rays[i])
                .expect("ray_intersection IS WRONG! (Expected a hit)");
            assert!(
                is_close(hit_record.normal, normals[i]),
                "Error occurred: index {} is responsible.\n",
                i
            );
        }
    }

    fn setup2() -> (Sphere<Translation>, Ray, Ray) {
        let translation = Translation::new(Vector::new(10.0, 0.0, 0.0));
        let sphere = Sphere::new(translation, Material::default());
        let ray = Ray::new(Point::new(10.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0));
        let ray2 = Ray::new(Point::new(13.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0));

        (sphere, ray, ray2)
    }

    #[test]
    fn test_sphere_ray_point_intersection2() {
        let (sphere, ray, ray2) = setup2();

        let point = Point::new(10.0, 0.0, 1.0);

        let hit_record = match sphere.ray_intersection(&ray) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.world_point, point),
            "Error occurred (1): point : {}\nhit_record.world_point : {}\n",
            point,
            hit_record.world_point
        );

        let point = Point::new(11.0, 0.0, 0.0);

        let hit_record = match sphere.ray_intersection(&ray2) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.world_point, point),
            "Error occurred (2): point : {}\nhit_record.world_point : {}\n",
            point,
            hit_record.world_point
        );
    }

    #[test]
    fn test_sphere_ray_normal_att2() {
        let (sphere, ray, ray2) = setup2();

        let normal = Normal::new(0.0, 0.0, 1.0);

        let hit_record = match sphere.ray_intersection(&ray) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.normal, normal),
            "Error occurred (1): normal : {}\nhit_record.normal : {}\n",
            normal,
            hit_record.normal
        );

        let normal = Normal::new(1.0, 0.0, 0.0);

        let hit_record = match sphere.ray_intersection(&ray2) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            is_close(hit_record.normal, normal),
            "Error occurred (2): normal : {}\nhit_record.normal : {}\n",
            normal,
            hit_record.normal
        );
    }

    #[test]
    fn test_sphere_uv_point_intersection2() {
        let (sphere, ray, ray2) = setup2();

        let uv = Vec2D::new(0.0, 0.0);

        let hit_record = match sphere.ray_intersection(&ray) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            uv.is_close(&hit_record.uv),
            "Error occurred (1): uv : {:?}\nhit_record.uv : {:?}\n",
            uv,
            hit_record.uv
        );

        let uv = Vec2D::new(0.0, 0.50);

        let hit_record = match sphere.ray_intersection(&ray2) {
            None => panic!("ray_intersection IS WRONG!"),
            Some(h) => h,
        };

        assert!(
            uv.is_close(&hit_record.uv),
            "Error occurred (2): uv : {:?}\nhit_record.uv : {:?}\n",
            uv,
            hit_record.uv
        );
    }

    #[test]
    fn test_sphere_ray_miss() {
        let (sphere, _, _) = setup2();
        let ray = Ray::new(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0));

        let hit_record = sphere.ray_intersection(&ray);
        assert!(
            hit_record.is_none(),
            "Error occurred (1): there is intersection where shouldn't!\n{}",
            hit_record.unwrap().world_point
        );

        let ray = Ray::new(Point::new(-10.0, 0.0, 0.0), Vector::new(0.0, 0.0, -1.0));

        let hit_record = sphere.ray_intersection(&ray);
        assert!(
            hit_record.is_none(),
            "Error occurred (2): there is intersection where shouldn't!\n{}",
            hit_record.unwrap().world_point
        );
    }

    #[test]
    fn test_sphere_ray_intersection_bug15() {
        let sphere: Sphere<Scaling> =
            Sphere::new(Scaling::new([0.1, 0.1, 0.1]), Material::default());
        for i in 0..100 {
            let ray = Ray::new(Point::new(-10.0 * i as f32, 0.0, 0.0), X_AXIS);
            let hit_record = sphere.ray_intersection(&ray);
            assert!(
                !hit_record.is_none(),
                "Error occurred ({}): there should be intersection!",
                i + 1
            );
        }
    }

    fn setup_plane() -> (Plane<Transformation>, Ray, Ray, Ray) {
        let material = Material {
            pigment: Box::new(UniformPigment::new(Color::new(10.0, 10.0, 10.0))),
            brdf: Box::new(DiffusiveBrdf {}),
            emitted_radiance: Box::new(UniformPigment::new(Color::new(10., 10., 10.))),
        };
        let transformation = Transformation::new(IDENTITY_4X4);
        let plane = Plane::new(transformation, material, false);

        // Ray from top to bottom
        let ray_top = Ray::new(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, -1.0));
        // Ray from bottom to top
        let ray_bottom = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        // Ray parallel to the plane
        let ray_parallel = Ray::new(Point::new(0.0, 0.0, 2.0), Vector::new(1.0, 0.0, 0.0));

        (plane, ray_top, ray_bottom, ray_parallel)
    }

    #[test]
    fn test_plane_intersection_and_normal() {
        let (plane, ray_top, ray_bottom, ray_parallel) = setup_plane();

        // Test 1: Impact from top
        let hit_top = plane
            .ray_intersection(&ray_top)
            .expect("Should hit the plane");
        assert!(are_close(hit_top.t, 5.0));
        assert!(is_close(hit_top.world_point, Point::new(0.0, 0.0, 0.0)));
        assert!(is_close(hit_top.normal, Normal::new(0.0, 0.0, 1.0)));
        assert!(hit_top.uv.is_close(&Vec2D::new(0.0, 0.0)));

        // Test 2: Impact from bottom (normal change sign)
        let hit_bottom = plane
            .ray_intersection(&ray_bottom)
            .expect("Should hit the plane");
        assert!(are_close(hit_bottom.t, 5.0));
        assert!(is_close(hit_bottom.world_point, Point::new(0.0, 0.0, 0.0)));
        assert!(is_close(hit_bottom.normal, Normal::new(0.0, 0.0, -1.0)));
        assert!(hit_bottom.uv.is_close(&Vec2D::new(0.0, 0.0)));

        // Test 3: parallel impact (no impact)
        let hit_parallel = plane.ray_intersection(&ray_parallel);
        assert!(
            hit_parallel.is_none(),
            "Parallel ray should not hit the plane"
        );
    }

    #[test]
    fn test_plane_uv_fractional_coordinates() {
        let transformation = Transformation::new(IDENTITY_4X4);
        let plane = Plane::new(transformation, Material::default(), false);

        // A ray hits the plane in x = 2.5, y = -1.3
        let ray = Ray::new(Point::new(2.5, -1.3, 5.0), Vector::new(0.0, 0.0, -1.0));

        let hit = plane.ray_intersection(&ray).expect("Should hit the plane");

        //  x = 2.5 -> 2.5 - 2.0 = 0.5
        //  y = -1.3 -> -1.3 - floor(-1.3) = -1.3 - (-2.0) = 0.7
        assert!(hit.uv.is_close(&Vec2D::new(0.5, 0.7)));
    }

    fn setup_triangle1() -> Triangle {
        Triangle {
            a: Point::new(0.0, 4.0, 0.0),
            b: Point::new(0.0, -1.0, 0.0),
            c: Point::new(0.0, 0.0, 4.0),
            material: Material::default(),
        }
    }

    #[test]
    fn test_triangle_intersection() {
        let triangle = setup_triangle1();

        let ray = Ray::new(Point::new(-1.0, 0.0, 2.0), Vector::new(1.0, 0.0, 0.0));

        let (t, beta, gamma) = triangle.intersection(ray).unwrap();
        assert!(are_close(t, 1.0), "t: {}\nexpected: 1.0", t);
        assert!(beta.is_between_open(&0.0, &1.0), "b: {}", beta);
        assert!(gamma.is_between_open(&0.0, &1.0), "gamma: {}", gamma);
    }

    #[test]
    fn test_triangle_intersection_none() {
        let triangle = setup_triangle1();

        let ray = Ray::new(Point::new(-1.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0));
        assert!(
            triangle.intersection(ray).is_err(),
            "The ray out of the scope must return Err"
        );

        let ray = Ray::new(Point::new(-1.0, 0.0, 10.0), Vector::new(1.0, 0.0, 0.0));
        assert!(
            triangle.intersection(ray).is_err(),
            "The ray out of the scope must return Err"
        );
    }

    #[test]
    fn test_triangle_ray_intersection() {
        let triangle = setup_triangle1();
        let ray = Ray::new(Point::new(-1.0, 0.0, 2.0), Vector::new(1.0, 0.0, 0.0));

        let hit_record = triangle
            .ray_intersection(&ray)
            .expect("Should hit the triangle");

        assert!(
            is_close(hit_record.world_point, Point::new(0.0, 0.0, 2.0)),
            "hit_record.world_point != hit_point"
        );
        assert!(
            is_close(hit_record.normal, Normal::new(-20.0, 0.0, 0.0)),
            "normal != hit_record.normal"
        );
        assert!(
            hit_record.uv.is_close(&Vec2D::new(0.4, 0.5)),
            "uv != hit_record.uv"
        );
        assert!(are_close(hit_record.t, 1.0), "t != hit_record.t");
        assert!(ray.is_close(hit_record.ray), "world_point != hit_point");
    }

    #[test]
    fn test_triangle_point_to_uv() -> Result<()> {
        let triangle = setup_triangle1();

        // Success case: return Ok with the correct value of the object
        let uv = triangle.point_to_uv(&Point::new(0.0, 0.0, 2.0))?;
        assert!(uv.is_close(&Vec2D::new(0.4, 0.5)));

        // Error case: return error
        // We use assert!(res.is_err()) for confirming the bug in not invinsible.
        let invalid_point = Point::new(10.0, 0.0, 0.0);
        let result = triangle.point_to_uv(&invalid_point);

        assert!(result.is_err(), "Should fail for point out of the triangle");
        println!("Error correctly intercepted: {}", result.unwrap_err());

        Ok(())
    }
}
