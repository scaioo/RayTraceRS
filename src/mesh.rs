// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! # Mesh
//!
//! This module defines triangle mesh geometry for the ray-tracer.
//!
//! ## Structure
//!
//! - [`IndexTriangle`] — three vertex indices referencing a shared [`SimpleMesh::points`] array.
//! - [`SimpleMesh`] — a flat triangle mesh stored as a shared vertex list plus index triples.
//!
//! ## Design note
//!
//! `SimpleMesh` operates entirely in world space. Transformations are applied once at load time
//! (see [`SimpleMesh::from_obj`]) rather than at every intersection query.
//! An axis-aligned bounding box is computed automatically at construction.
//! Flat meshes may be expanded by a small epsilon along degenerate axes.
//!
//! BVH acceleration is not included here and is planned as a separate follow-up.

use crate::functions::Within;
use crate::geometry::{Normal, Point, Vec2D};
use crate::hit_record::HitRecord;
use crate::materials::Material;
use crate::ray::Ray;
use crate::shapes::{AABB, Shape, Triangle};
use crate::transformations::IsHomogeneousMatrix;
use anyhow::{Result, anyhow};
use std::ops::Mul;
use std::path::Path;

/// A triangle specified by three indices into a [`SimpleMesh::points`] array.
///
/// Using indices rather than storing three vertices directly lets multiple triangles
/// share the same point data without duplication, and keeps the struct small
/// (3 × `u32` = 12 bytes).
#[derive(Clone, Debug, PartialEq)]
pub struct IndexTriangle {
    /// Index of the first vertex.
    pub i: u32,
    /// Index of the second vertex.
    pub j: u32,
    /// Index of the third vertex.
    pub k: u32,
}

impl IndexTriangle {
    /// Creates a new [`IndexTriangle`] from three vertex indices.
    pub fn new(i: u32, j: u32, k: u32) -> IndexTriangle {
        IndexTriangle { i, j, k }
    }
}

/// A triangle mesh stored as a shared vertex list and a list of [`IndexTriangle`]s.
///
/// All vertices are held in world space — any transformation supplied to
/// [`SimpleMesh::from_obj`] is baked into [`SimpleMesh::points`] at load time.
///
/// ## Intersection strategy
///
/// Intersection is a two-phase process:
/// 1. **Broad phase** — the ray is tested against the mesh's [`AABB`]. If the ray origin
///    is outside the box and the box is missed, the entire mesh is skipped.
/// 2. **Narrow phase** — every triangle is tested in turn; the closest hit within
///    `[ray.t_min, ray.t_max]` is returned.
///
/// ## UV coordinates
///
/// The UV at a hit point is the barycentric pair `(β, γ)` of the struck triangle,
/// consistent with how [`Triangle`] reports UV. Full texture mapping is left as future work.
///
/// ## Note on transformations
///
/// Unlike [`Sphere`] and [`Plane`], `SimpleMesh` is not generic over a transformation.
/// Pass a transformation to [`SimpleMesh::from_obj`]; it is applied to every vertex once
/// during loading. For meshes constructed directly via [`SimpleMesh::new`], pre-transform
/// the points before passing them in.
#[derive(Clone)]
pub struct SimpleMesh {
    /// World-space vertices shared by all triangles.
    pub points: Vec<Point>,
    /// Triangle connectivity encoded as index triples into [`SimpleMesh::points`].
    pub index_triangles: Vec<IndexTriangle>,
    /// Tight axis-aligned bounding box over all vertices, used for broad-phase rejection.
    pub aabb: AABB,
    /// Surface material applied to every triangle in the mesh.
    pub material: Material,
}

/// Computes the tight axis-aligned bounding box over `points`.
///
/// # Errors
///
/// Returns an error if `points` is empty or if `AABB::new`
/// cannot construct a valid bounding box from the supplied points.
fn runtime_aabb(points: &[Point]) -> Result<AABB> {
    if points.is_empty() {
        return Err(anyhow!("runtime_aabb(): empty points"));
    }
    let mut p_min = points[0];
    let mut p_max = points[0];
    for point in points.iter() {
        if point.x < p_min.x {
            p_min.x = point.x;
        }
        if point.y < p_min.y {
            p_min.y = point.y;
        }
        if point.z < p_min.z {
            p_min.z = point.z;
        }
        if point.x > p_max.x {
            p_max.x = point.x;
        }
        if point.y > p_max.y {
            p_max.y = point.y;
        }
        if point.z > p_max.z {
            p_max.z = point.z;
        }
    }
    AABB::new(p_min, p_max, Material::default())
}

impl SimpleMesh {
    /// Creates a [`SimpleMesh`] from pre-built vertex and index data.
    ///
    /// The bounding box is computed automatically from `points`.
    ///
    /// # Errors
    ///
    /// Returns an error if `points` is empty or if `AABB::new`
    /// cannot construct a valid bounding box from the supplied points.
    pub fn new(
        points: Vec<Point>,
        index_triangles: Vec<IndexTriangle>,
        material: Material,
    ) -> Result<Self> {
        let aabb = runtime_aabb(&points)?;
        Ok(Self {
            points,
            index_triangles,
            material,
            aabb,
        })
    }

    /// Loads a triangle mesh from a Wavefront OBJ file and returns a [`SimpleMesh`].
    ///
    /// All models present in the file are merged into a single mesh. The supplied
    /// `transformation` is applied to every vertex after loading, letting the caller
    /// place or orient the mesh in world space without a separate pass.
    ///
    /// The file is loaded with [`tobj::OFFLINE_RENDERING_LOAD_OPTIONS`], which
    /// triangulates polygons automatically; the indices check is therefore a
    /// defensive safety net rather than a common code path.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - the OBJ file cannot be read or parsed by `tobj`,
    /// - any model's positions array length is not divisible by 3,
    /// - any model's indices array length is not divisible by 3,
    /// - the file contains no geometry.
    pub fn from_obj<T>(path: &Path, material: Material, transformation: T) -> Result<Self>
    where
        T: IsHomogeneousMatrix + Mul<Point, Output = Point>,
    {
        let (models, _) = match tobj::load_obj(path, &tobj::OFFLINE_RENDERING_LOAD_OPTIONS) {
            Ok((a, b)) => (a, b),
            Err(e) => return Err(anyhow!("Error loading OBJ file {:?}: {}", path, e)),
        };

        let mut points: Vec<Point> = Vec::new();
        let mut index_triangles: Vec<IndexTriangle> = Vec::new();

        for model in &models {
            let mesh = &model.mesh;

            // Check1: Positions must be a multiple of 3.
            if mesh.positions.len() % 3 != 0 {
                return Err(anyhow!(
                    "Incorrect number of points: Model '{}' has a positions array \
                    whose length ({}) is not divisible by 3",
                    model.name,
                    mesh.positions.len()
                ));
            }

            // Check2: Indices must be a multiple of 3.
            if mesh.indices.len() % 3 != 0 {
                return Err(anyhow!(
                    "Incorrect number of indices: Model '{}' has an indices array whose length ({}) \
                    is not divisible by 3 after triangulation — check that the mesh is triangulated in Blender.",
                    model.name,
                    mesh.indices.len()
                ));
            }

            // Every mesh object stored in the file adds a series of points.
            // However, Triangle Indexing always starts from 0, so it would point
            // always to the same ones
            let base = points.len() as u32;

            // Convert [x0,y0,z0, x1,y1,z1, ...] into Points vector.
            for chunk in mesh.positions.chunks(3) {
                points.push(Point::new(chunk[0], chunk[1], chunk[2]));
            }

            for tri in mesh.indices.chunks(3) {
                index_triangles.push(IndexTriangle::new(
                    base + tri[0],
                    base + tri[1],
                    base + tri[2],
                ));
            }
        }

        if points.is_empty() {
            return Err(anyhow!("OBJ file at {:?} contained no geometry", path));
        }

        let transformed_points = points.iter().map(|p| transformation * *p).collect();

        Self::new(transformed_points, index_triangles, material)
    }

    /// Returns the closest triangle hit by `ray`, together with the ray parameter and
    /// barycentric coordinates `(β, γ)` at the hit point.
    ///
    /// Uses the mesh [`AABB`] as a broad-phase guard: if the ray origin lies outside
    /// the bounding box and the box is missed, `None` is returned immediately without
    /// testing any triangles. Rays originating inside the box always proceed to the
    /// narrow phase.
    fn triangle_hit(&self, ray: &Ray) -> Option<(Triangle, f32, f32, f32)> {
        // AABB optimization check
        if !self.aabb.contains(&ray.origin) {
            self.aabb.ray_intersection(ray)?;
        }

        // Parameters
        let mut closest_t: f32 = ray.t_max;
        let mut b = 0.0;
        let mut g = 0.0;
        let mut hit_index: Option<u32> = None;

        // Hit loop
        for (idx, triangle) in self.index_triangles.iter().enumerate() {
            let helper_triangle = Triangle::new(
                self.points[triangle.i as usize],
                self.points[triangle.j as usize],
                self.points[triangle.k as usize],
                Material::default(),
            );

            let Ok((t, beta, gamma)) = helper_triangle.intersection(*ray) else {
                continue;
            };

            if t.is_between_open(&ray.t_min, &closest_t) && t > 0.0 {
                closest_t = t;
                hit_index = Some(idx as u32);
                (b, g) = (beta, gamma);
            }
        }

        // Return the result if it hits
        hit_index.map(|idx| {
            let tri_idx = &self.index_triangles[idx as usize];
            let triangle = Triangle {
                a: self.points[tri_idx.i as usize],
                b: self.points[tri_idx.j as usize],
                c: self.points[tri_idx.k as usize],
                material: self.material.clone(),
            };
            (triangle, closest_t, b, g)
        })
    }
}

impl Shape for SimpleMesh {
    /// Returns the closest [`HitRecord`] for `ray` against this mesh, or `None` on a miss.
    ///
    /// The normal is computed per-triangle (flat shading). UV coordinates are the
    /// barycentric pair `(β, γ)` of the hit triangle.
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        let (triangle, t, beta, gamma) = self.triangle_hit(ray)?;
        let world_point = ray.at(t);

        Some(HitRecord {
            world_point,
            normal: triangle.normal_at(world_point, ray),
            uv: Vec2D::new(beta, gamma),
            t,
            ray: *ray,
            material: &self.material,
        })
    }

    /// Not implemented for `SimpleMesh`.
    ///
    /// Normals are computed inside [`SimpleMesh::ray_intersection`] via the struck
    /// [`Triangle`] directly. This method is provided only to satisfy the [`Shape`]
    /// trait and will panic if called.
    fn normal_at(&self, _point: Point, _ray: &Ray) -> Normal {
        todo!()
    }

    /// Not implemented for `SimpleMesh`.
    ///
    /// UV coordinates are returned as barycentric pairs `(β, γ)` directly from
    /// [`SimpleMesh::ray_intersection`]. This method is provided only to satisfy the
    /// [`Shape`] trait and will panic if called.
    fn point_to_uv(&self, _point: &Point) -> Result<Vec2D> {
        todo!()
    }

    fn material(&self) -> &Material {
        &self.material
    }
}

// **********************************************
// Tests
// **********************************************

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::are_close;
    use crate::geometry::{Y_AXIS, Z_AXIS};
    use crate::transformations::{IDENTITY_TRANSFORMATION, Translation};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // ---- IndexTriangle constructor ----------------------------
    #[test]
    fn test_index_triangle_constructor() {
        let i: u32 = 1;
        let j: u32 = 2;
        let k: u32 = 3;

        let tri = IndexTriangle::new(i, j, k);
        assert_eq!(i, tri.i);
        assert_eq!(j, tri.j);
        assert_eq!(k, tri.k);
    }

    // ---- SimpleMesh parallelepiped ----------------------------

    fn setup_parallelepiped() -> (Vec<Point>, Vec<IndexTriangle>, SimpleMesh) {
        let points = vec![
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }, // P1
            Point {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }, // P2
            Point {
                x: 1.0,
                y: 2.0,
                z: 0.0,
            }, // P3
            Point {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            }, // P4
            Point {
                x: 0.0,
                y: 0.0,
                z: 3.0,
            }, // P5
            Point {
                x: 1.0,
                y: 0.0,
                z: 3.0,
            }, // P6
            Point {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }, // P7
            Point {
                x: 0.0,
                y: 2.0,
                z: 3.0,
            }, // P8
        ];

        let index_triangles = vec![
            IndexTriangle { i: 0, j: 1, k: 2 },
            IndexTriangle { i: 0, j: 2, k: 3 },
            IndexTriangle { i: 4, j: 5, k: 0 },
            IndexTriangle { i: 0, j: 1, k: 5 },
            IndexTriangle { i: 5, j: 6, k: 7 },
            IndexTriangle { i: 4, j: 5, k: 7 },
            IndexTriangle { i: 3, j: 2, k: 7 },
            IndexTriangle { i: 7, j: 2, k: 6 },
            IndexTriangle { i: 0, j: 7, k: 3 },
            IndexTriangle { i: 0, j: 4, k: 7 },
            IndexTriangle { i: 1, j: 2, k: 5 },
            IndexTriangle { i: 5, j: 2, k: 6 },
        ];

        let mesh = SimpleMesh {
            points: points.clone(),
            index_triangles: index_triangles.clone(),
            material: Material::default(),
            aabb: runtime_aabb(&points).unwrap(),
        };

        (points, index_triangles, mesh)
    }

    // ---- SimpleMesh aabb computing ----------------------------

    #[test]
    fn test_mesh_runtime_aabb() {
        let (_, _, mesh) = setup_parallelepiped();
        let p_max = Point::new(1.0, 2.0, 3.0);
        let p_min = Point::new(0.0, 0.0, 0.0);

        assert!(mesh.aabb.p_max.is_close(&p_max));
        assert!(mesh.aabb.p_min.is_close(&p_min));
    }

    // ---- SimpleMesh constructor ----------------------------

    #[test]
    fn test_mesh_constructor_triangles() {
        let (_, triangles, mesh) = setup_parallelepiped();

        let mesh_triangles = mesh.index_triangles.clone();
        assert_eq!(triangles.len(), mesh_triangles.len());
        for (l, triangle) in triangles.iter().enumerate() {
            assert_eq!(mesh_triangles[l].i, triangle.i);
            assert_eq!(triangle.j, mesh_triangles[l].j);
            assert_eq!(triangle.k, mesh_triangles[l].k);
        }
    }

    #[test]
    fn test_mesh_constructor_points() {
        let (points, _, mesh) = setup_parallelepiped();

        let mesh_points = mesh.points.clone();
        assert_eq!(points.len(), mesh_points.len());
        for (l, point) in points.iter().enumerate() {
            assert!(point.is_close(&mesh_points[l]));
        }
    }

    // ---- SimpleMesh loading function -----------------------

    fn create_test_file(path: &PathBuf) -> Result<()> {
        let mut file = File::create(path)?;

        let content = r#"# Parallelepiped Mesh
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 2.0 0.0
v 0.0 2.0 0.0
v 0.0 0.0 3.0
v 1.0 0.0 3.0
v 1.0 2.0 3.0
v 0.0 2.0 3.0

f 1 2 3
f 1 3 4
f 5 6 1
f 1 2 6
f 6 7 8
f 5 6 8
f 4 3 8
f 8 3 7
f 1 8 4
f 1 5 8
f 2 3 6
f 6 3 7
"#;

        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_from_obj() -> Result<()> {
        // 1. Create a temporary directory
        let dir = tempdir()?;

        // 2. Define input and output paths inside the temp directory
        let path = dir.path().join("parallelepiped.obj");
        create_test_file(&path)?;

        // 3.1 Download mesh from file
        let mesh = SimpleMesh::from_obj(&path, Material::default(), IDENTITY_TRANSFORMATION)?;
        // 3.2 Create expected mesh
        let (_, _, expected_mesh) = setup_parallelepiped();

        // 4. Points asserts
        let points = mesh.points.clone();
        let expected_points = expected_mesh.points.clone();

        for (i, point) in points.iter().enumerate() {
            assert!(
                point.is_close(&expected_points[i]),
                "Broken assert {i} line:\npoint: {:?}\nexpected: {:?}",
                point,
                expected_points[i]
            );
        }

        // 5. Index asserts
        let indices = mesh.index_triangles.clone();
        let expected_indices = expected_mesh.index_triangles.clone();

        for (i, index) in indices.iter().enumerate() {
            assert_eq!(
                &expected_indices[i], index,
                "Broken assert {i} line:\nindex: {:?}\nexpected: {:?}",
                index, expected_indices[i]
            );
        }

        Ok(())
    }

    // ---- SimpleMesh pyramid helper ----------------------------

    fn setup_pyramid<T>(transformation: T) -> SimpleMesh
    where
        T: IsHomogeneousMatrix + Mul<Point, Output = Point> + Clone,
    {
        let points = vec![
            Point::new(1.0, 1.0, 0.0),   //P0
            Point::new(-1.0, 1.0, 0.0),  //P1
            Point::new(-1.0, -1.0, 0.0), //P2
            Point::new(1.0, -1.0, 0.0),  //P3
            Point::new(0.0, 0.0, 1.0),   //P4
        ];

        let points = points.iter().map(|p| transformation.clone() * *p).collect();

        let indices = vec![
            IndexTriangle::new(0, 1, 4),
            IndexTriangle::new(2, 1, 4),
            IndexTriangle::new(3, 2, 4),
            IndexTriangle::new(4, 3, 0),
            IndexTriangle::new(1, 3, 0),
            IndexTriangle::new(2, 3, 1),
        ];

        SimpleMesh::new(points, indices, Material::default()).unwrap()
    }
    #[test]
    fn test_from_obj_with_translation() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("parallelepiped.obj");
        create_test_file(&path)?;

        let transformation = Translation::new(Z_AXIS);
        let mesh = SimpleMesh::from_obj(&path, Material::default(), transformation)?;

        // Expected: same parallelepiped points, each translated by Z_AXIS
        let (points, index_triangles, _) = setup_parallelepiped();
        let expected_points: Vec<Point> = points.iter().map(|p| transformation * *p).collect();

        for (i, point) in mesh.points.iter().enumerate() {
            assert!(
                point.is_close(&expected_points[i]),
                "Point {i}: got {:?}, expected {:?}",
                point,
                expected_points[i]
            );
        }

        for (i, index) in mesh.index_triangles.iter().enumerate() {
            assert_eq!(
                &index_triangles[i], index,
                "Triangle {i}: got {:?}, expected {:?}",
                index, index_triangles[i]
            );
        }

        Ok(())
    }

    // ---- SimpleMesh triangle_hit ----------------------------
    // ---- Outside -> Miss
    #[test]
    fn test_triangle_hit_parallelepiped_miss() {
        let (_, _, parallelepiped) = setup_parallelepiped();
        let miss_ray = Ray::new(Point::new(1.5, 1.0, 10.0), -Z_AXIS);
        let result = parallelepiped.triangle_hit(&miss_ray);
        assert!(result.is_none(), "Assert missed-mesh failed!");
        let aabb = parallelepiped.aabb;
        let result = aabb.ray_intersection(&miss_ray);
        assert!(result.is_none(), "Assert missed AABB failed!");
    }

    #[test]
    fn test_triangle_hit_pyramid_miss() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let ray: Ray = Ray::new(Point::new(0.5, -10.0, 0.9), Y_AXIS);
        let result = pyramid.triangle_hit(&ray);
        assert!(result.is_none(), "Assert missed Mesh failed!");
        let aabb = pyramid.aabb;
        let result = aabb.ray_intersection(&ray);
        assert!(result.is_some(), "Assert hitted AABB failed!");
    }

    // ---- Outside -> Hit
    #[test]
    fn test_triangle_hit_parallelepiped_hit() {
        let (_, _, parallelepiped) = setup_parallelepiped();
        let ray: Ray = Ray::new(Point::new(0.25, 4.0, 3.0 / 4.0), -Y_AXIS);
        let (triangle, t_min, beta, gamma) = parallelepiped.triangle_hit(&ray).unwrap();

        assert!(
            are_close(t_min, 2.0),
            "Assert 1 fails: t_min: {}\nexpected_t: 2.0",
            t_min
        );
        assert!(
            are_close(beta, 0.25),
            "Assert 2 fails: beta: {}\nexpect_b: 0.5",
            beta
        );
        assert!(
            are_close(gamma, 0.25),
            "Assert 3 fails: gamma: {}\nexpect_g: 0.25",
            gamma
        );

        let points: [Point; 3] = [
            Point::new(0.0, 2.0, 0.0),
            Point::new(0.0, 2.0, 3.0),
            Point::new(1.0, 2.0, 0.0),
        ];

        assert!(
            points.contains(&triangle.a),
            "Assert 4 fails: triangle.a: {}",
            triangle.a
        );
        assert!(
            points.contains(&triangle.b),
            "Assert 5 fails: triangle.b: {}",
            triangle.b
        );
        assert!(
            points.contains(&triangle.c),
            "Assert 6 fails: triangle.c: {}",
            triangle.c
        );
    }

    #[test]
    fn test_triangle_hit_pyramid_hit() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let ray: Ray = Ray::new(Point::new(0.0, 0.5, 1.0), -Z_AXIS);
        let (triangle, t_min, beta, gamma) = pyramid.triangle_hit(&ray).unwrap();

        assert!(
            are_close(t_min, 0.5),
            "Assert 1 fails: t_min: {}\nexpected_t: 0.5",
            t_min
        );
        assert!(
            are_close(beta, 0.25),
            "Assert 2 fails: beta: {}\nexpect_b: 0.5",
            beta
        );
        assert!(
            are_close(gamma, 0.5),
            "Assert 3 fails: gamma: {}\nexpect_g: 0.5",
            gamma
        );

        let points: [Point; 3] = [
            Point::new(1.0, 1.0, 0.0),
            Point::new(-1.0, 1.0, 0.0),
            Point::new(0.0, 0.0, 1.0),
        ];

        assert!(
            points.contains(&triangle.a),
            "Assert 4 fails: triangle.a: {}",
            triangle.a
        );
        assert!(
            points.contains(&triangle.b),
            "Assert 5 fails: triangle.b: {}",
            triangle.b
        );
        assert!(
            points.contains(&triangle.c),
            "Assert 6 fails: triangle.c: {}",
            triangle.c
        );
    }

    // ---- Inside -> Hit
    #[test]
    fn test_triangle_hit_pyramid_inside() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let ray: Ray = Ray::new(Point::new(0.0, 0.5, 0.1), Z_AXIS);
        let (triangle, t_min, beta, gamma) = pyramid.triangle_hit(&ray).unwrap();

        assert!(
            are_close(t_min, 0.4),
            "Assert 1 fails: t_min: {}\nexpected_t: 0.5",
            t_min
        );
        assert!(
            are_close(beta, 0.25),
            "Assert 2 fails: beta: {}\nexpect_b: 0.5",
            beta
        );
        assert!(
            are_close(gamma, 0.5),
            "Assert 3 fails: gamma: {}\nexpect_g: 0.5",
            gamma
        );

        let points: [Point; 3] = [
            Point::new(1.0, 1.0, 0.0),
            Point::new(-1.0, 1.0, 0.0),
            Point::new(0.0, 0.0, 1.0),
        ];

        assert!(
            points.contains(&triangle.a),
            "Assert 4 fails: triangle.a: {}",
            triangle.a
        );
        assert!(
            points.contains(&triangle.b),
            "Assert 5 fails: triangle.b: {}",
            triangle.b
        );
        assert!(
            points.contains(&triangle.c),
            "Assert 6 fails: triangle.c: {}",
            triangle.c
        );
    }

    // ---- Inside -> Miss
    #[test]
    fn test_triangle_hit_pyramid_ray_range_miss() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let ray: Ray = Ray {
            origin: Point::new(0.0, 0.5, 0.1),
            dir: Z_AXIS,
            t_max: 0.3,
            t_min: 0.0,
            depth: 0,
        };

        assert!(pyramid.triangle_hit(&ray).is_none(), "pyramid is hit!");
    }

    // ---- Inside -> Hit SimpleMesh, Miss AABB
    #[test]
    fn test_triangle_hit_pyramid_ray_miss_aabb_but_hit() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let ray = Ray {
            origin: Point::new(0.0, -0.5, 0.1),
            dir: Z_AXIS,
            t_max: 0.7,
            t_min: 0.0,
            depth: 0,
        };

        assert!(
            pyramid.aabb.ray_intersection(&ray).is_none(),
            "pyramid's aabb is hit!"
        );
        assert!(pyramid.triangle_hit(&ray).is_some(), "pyramid is not hit!");
    }

    // ---- Inside -> Miss SimpleMesh, Hit AABB
    #[test]
    fn test_triangle_hit_pyramid_ray_inside_aabb_miss() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let ray = Ray {
            origin: Point::new(0.0, -0.5, 0.7),
            dir: Z_AXIS,
            t_max: f32::INFINITY,
            t_min: 0.0,
            depth: 0,
        };

        assert!(
            pyramid.aabb.ray_intersection(&ray).is_some(),
            "pyramid's aabb is not hit!"
        );
        assert!(pyramid.triangle_hit(&ray).is_none(), "pyramid is hit!");
    }

    // ---- SimpleMesh ray_intersection ----------------------------
    // ---- Outside -> Hit Parallelepiped
    #[test]
    fn test_ray_intersection_out_hit() {
        let (_, _, parallelepiped) = setup_parallelepiped();
        let ray: Ray = Ray::new(Point::new(0.25, 1.5, -1.0), Z_AXIS);
        let intersection = parallelepiped.ray_intersection(&ray).unwrap();

        let expected = HitRecord {
            world_point: Point::new(0.25, 1.5, 0.0),
            normal: Normal::from(-2.0 * Z_AXIS),
            uv: Vec2D { x: 0.25, y: 0.5 },
            t: 1.0,
            ray,
            material: &Default::default(),
        };

        assert!(
            expected.world_point.is_close(&intersection.world_point),
            "intersection point: {}",
            intersection.world_point.x
        );
        assert!(
            expected.normal.is_close(&intersection.normal),
            "normal: {}",
            intersection.normal
        );
        assert!(
            expected.uv.is_close(&intersection.uv),
            "uv: {:?}",
            intersection.uv
        );
        assert!(
            are_close(expected.t, intersection.t),
            "t: {}",
            intersection.t
        );
    }

    // ---- Outside -> Inside AABB and Hits Parallelepiped
    #[test]
    fn test_ray_intersection_in_aabb_hit() {
        let pyramid = setup_pyramid(IDENTITY_TRANSFORMATION);
        let dir = -(Z_AXIS + Y_AXIS);
        let origin = Point::new(0.0, 0.0, 0.0) - 0.75 * dir;
        let ray: Ray = Ray::new(origin, dir);
        let expected = Point::new(0.0, 0.5, 0.5);
        let hit = pyramid.ray_intersection(&ray).unwrap();

        assert!(
            expected.is_close(&hit.world_point),
            "intersection point: {}",
            hit.world_point
        );
    }

    // ---- Outside -> Miss AABB and Parallelepiped
    #[test]
    fn test_ray_intersection_out_miss() {
        let (_, _, parallelepiped) = setup_parallelepiped();
        let ray: Ray = Ray::new(Point::new(0.25, 2.5, -1.0), Z_AXIS);
        assert!(
            parallelepiped.ray_intersection(&ray).is_none(),
            "parallelepiped is hit!"
        );
    }

    // ---- Flat SimpleMesh case -----------------
    fn setup_flat() -> SimpleMesh {
        //
        //             *  P5
        //            /|
        //           / |
        //          /  |
        //     P6  * - *   P4
        //         |   |\
        //         |   | \
        //         |   |  \
        //         * - * - *
        //        P1  P2   P3
        //   (Scaling is not correct)
        let points: Vec<Point> = vec![
            Point::new(-1.0, 0.0, 0.0), //P1
            Point::new(0.0, 0.0, 0.0),  //P2
            Point::new(1.0, 0.0, 0.0),  //P3
            Point::new(0.0, 1.0, 0.0),  //P4
            Point::new(0.0, 2.0, 0.0),  //P5
            Point::new(-1.0, 1.0, 0.0), //P6
        ];

        let triangles: Vec<IndexTriangle> = vec![
            IndexTriangle::new(2, 3, 1),
            IndexTriangle::new(0, 3, 1),
            IndexTriangle::new(3, 4, 5),
            IndexTriangle::new(0, 3, 5),
        ];

        SimpleMesh::new(points, triangles, Material::default()).unwrap()
    }

    #[test]
    fn test_flat_case_aabb_construction() {
        let object = setup_flat();

        // Assert aabb is correctly constructed
        assert!(
            object.aabb.p_min.is_close(&Point::new(-1.0, 0.0, -1e-4)),
            "aabb.p_min : {:?}",
            object.aabb.p_min
        );
        assert!(
            object.aabb.p_max.is_close(&Point::new(1.0, 2.0, 1e-4)),
            "aabb.p_max : {:?}",
            object.aabb.p_max
        );
    }

    #[test]
    fn test_flat_case_hit() {
        let object = setup_flat();

        // Assert ray hits - border case
        let origin = Point::new(-0.5, 0.3, 1.0);
        let ray = Ray::new(origin, -Z_AXIS);
        // check aabb is hit
        assert!(
            object.aabb.ray_intersection(&ray).is_some(),
            "Failed aabb intersection asset: ray doesn't hit!"
        );
        // check the actual intersection is none since it's parallel to the plane
        let hit = object.ray_intersection(&ray).unwrap();
        let hit_point = hit.world_point;
        assert!(
            hit_point.is_close(&Point::new(-0.5, 0.3, 0.0)),
            "{:?}",
            hit_point
        );
        let normal = hit.normal;
        assert!(normal.is_close(&Normal::from(Z_AXIS)), "{:?}", normal);
    }

    #[test]
    fn test_flat_case_border() {
        let object = setup_flat();

        // Assert ray hits - border case
        let origin = Point::new(0.0, -10.0, 0.0);
        let ray = Ray::new(origin, Y_AXIS);
        // check aabb is hit
        assert!(
            object.aabb.ray_intersection(&ray).is_some(),
            "Failed aabb intersection asset: ray doesn't hit!"
        );
        // check the actual intersection is none since it's parallel to the plane
        assert!(
            object.ray_intersection(&ray).is_none(),
            "Failed ray intersection assert: ray hits!"
        );
    }
}
