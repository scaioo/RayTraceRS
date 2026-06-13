use crate::geometry::Point;
use crate::materials::Material;
use crate::shapes::AABB;

#[derive(Clone, Debug, PartialEq)]
pub struct IndexTriangle {
    pub i: u32,
    pub j: u32,
    pub k: u32,
}

impl IndexTriangle {
    pub fn new(i: u32, j: u32, k: u32) -> IndexTriangle {
        IndexTriangle { i, j, k }
    }
}

#[derive(Clone)]
pub struct SimpleMesh {
    pub points: Vec<Point>,
    pub index_triangles: Vec<IndexTriangle>,
    pub aabb: AABB,
    pub material: Material,
}

fn runtime_aabb(points: &Vec<Point>) -> AABB {
    let n = points.len();
    let mut p_min = points[0];
    let mut p_max = points[0];
    for i in 1..n {
        if points[i].x < p_min.x {
            p_min.x = points[i].x;
        }
        if points[i].y < p_min.y {
            p_min.y = points[i].y;
        }
        if points[i].z < p_min.z {
            p_min.z = points[i].z;
        }
        if points[i].x > p_max.x {
            p_max.x = points[i].x;
        }
        if points[i].y > p_max.y {
            p_max.y = points[i].y;
        }
        if points[i].z > p_max.z {
            p_max.z = points[i].z;
        }
    }
    AABB::new(p_min, p_max, Material::default()).unwrap()
}

impl SimpleMesh {
    pub fn new(
        points: Vec<Point>,
        index_triangles: Vec<IndexTriangle>,
        material: Material,
    ) -> Self {
        Self {
            points: points.clone(),
            index_triangles,
            material,
            aabb: runtime_aabb(&points),
        }
    }
}

// **********************************************
// Tests
// **********************************************

#[cfg(test)]
mod tests {
    use super::*;

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

    fn setup_parallelepiped() -> (Vec<Point>, Vec<IndexTriangle>, SimpleMesh) {
        let points = vec![
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }, // P0
            Point {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }, // P1
            Point {
                x: 1.0,
                y: 2.0,
                z: 0.0,
            }, // P2
            Point {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            }, // P3
            Point {
                x: 0.0,
                y: 0.0,
                z: 3.0,
            }, // P4
            Point {
                x: 1.0,
                y: 0.0,
                z: 3.0,
            }, // P5
            Point {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }, // P6
            Point {
                x: 0.0,
                y: 2.0,
                z: 3.0,
            }, // P7
        ];

        let index_triangles = vec![
            IndexTriangle { i: 0, j: 1, k: 2 },
            IndexTriangle { i: 0, j: 2, k: 3 },
            IndexTriangle { i: 0, j: 1, k: 5 },
            IndexTriangle { i: 4, j: 5, k: 0 },
            IndexTriangle { i: 4, j: 5, k: 7 },
            IndexTriangle { i: 5, j: 6, k: 7 },
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
            aabb: runtime_aabb(&points),
        };

        (points, index_triangles, mesh)
    }

    #[test]
    fn test_mesh_runtime_aabb() {
        let (_, _, mesh) = setup_parallelepiped();
        let p_max = Point::new(1.0, 2.0, 3.0);
        let p_min = Point::new(0.0, 0.0, 0.0);

        assert!(mesh.aabb.p_max.is_close(&p_max));
        assert!(mesh.aabb.p_min.is_close(&p_min));
    }

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
}
