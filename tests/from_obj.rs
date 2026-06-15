use std::path::PathBuf;
use anyhow::Result;
use rstrace::geometry::Point;
use rstrace::materials::Material;
use rstrace::mesh::SimpleMesh;
use rstrace::transformations::IDENTITY_TRANSFORMATION;

fn contains_point(points: &[Point], target: &Point) -> bool {
    points.iter().any(|p| p.is_close(target))
}

#[test]
fn test_from_obj_two_pawns() -> Result<()> {
    // Note: In the file 184 vertices are present for the first, however one is orphan
    // so tobj::load_obj doesn't stores it in SimpleMes. 
    //
    // LLMs found orphan in position 183 of the file. 
    
    // Build the pawns SimpleMesh
    let path = PathBuf::from("tests/assets/2_pawn.obj");
    let pawns = SimpleMesh::from_obj(&path, Material::default(), IDENTITY_TRANSFORMATION)?;

    // Two pawns: 183 + 183 = 367 vertices, 362 + 362 = 724 triangles.
    let n = pawns.points.len() as u32;
    assert_eq!(
        n, 366,
        "expected 366 merged vertices, got {}", pawns.points.len()
    );
    assert_eq!(
        pawns.index_triangles.len(), 724,
        "expected 724 merged triangles, got {}", pawns.index_triangles.len()
    );

    // If the base offset is wrong, second-object indices will exceed 367.
    for (idx, triangle) in pawns.index_triangles.iter().enumerate() {
        assert!(triangle.i < n, "triangle {idx}: i={} out of bounds (n={n})", triangle.i);
        assert!(triangle.j < n, "triangle {idx}: j={} out of bounds (n={n})", triangle.j);
        assert!(triangle.k < n, "triangle {idx}: k={} out of bounds (n={n})", triangle.k);
    }

    // Points manually picked
    let expected_points = vec![
        // First pawn (vertices 0..184)
        Point::new(-0.033697, -4.026902, -0.000088), // vertex 0
        Point::new(0.252794, -4.595588, 2.538875), // vertex 40
        Point::new(-0.469561, -5.410888, 1.412636), // vertex 110
        Point::new(0.353503, -4.730892, 2.824665),  // vertex 182

        // Second pawn (vertices 184..367)
        Point::new(0.486549, 0.016849, -0.000044),   // vertex 183
        Point::new(-0.470698, -0.326629, 0.112080),   // vertex 183 + 150 = 333
        Point::new(0.134554, -0.176751, 1.412333),   // vertex 365
    ];

    // Check if a point is assigned correctly to little pawn or big pawn
    let first_pawn = &pawns.points[..183];
    let second_pawn = &pawns.points[183..];

    for (i, point) in expected_points.iter().enumerate() {
        if i < 4 {
            assert!(
                contains_point(first_pawn, point),
                "Assert violation - index {i}\nPoint {point} is not in first pawn"
            );
        } else {
            assert!(
                contains_point(second_pawn, point),
                "Assert violation - index {i}\nPoint {point} is not in second pawn"
            );
        }
    }

    Ok(())
}

