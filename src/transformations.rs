use crate::functions::{IDENTITY_4X4, are_close, fast_matrix_mul, inverse_4x4, transpose_matrix};
use crate::geometry::{Normal, Point, Vector};
use std::ops::Mul;

// =======================================================================
// TRAIT DEFINITIONS
// =======================================================================
/// This trait is the Marker Trait for Transformations
///
/// It gives the matrix and the inverse-transposed matrix of the transformation
pub trait IsHomogeneousMatrix {
    /// It returns the transformation homogeneous matrix
    fn mat(&self) -> &[f32; 16];

    /// It returns the inverse-transposed matrix of the transformation
    fn it_mat(&self) -> &[f32; 16];
}

// =======================================================================
// MACRO DEFINITIONS
// =======================================================================

#[macro_export]
macro_rules! impl_matrix_operations {
    ($t: ident) => {
        // Note: we totally ignored - so far - the possibility that Point vector
        // has a last coordinate different from one in the homogeneous space.

        // -----------------------       Marker trait      -------------------------
        impl IsHomogeneousMatrix for $t {
            fn mat(&self) -> &[f32; 16] {
                &self.mat
            }

            fn it_mat(&self) -> &[f32; 16] {
                &self.it_mat
            }
        }
        // -----------------------   Matrix * Matrix    -------------------------

        // Do we want to use the * symbol for the matrix-rhs product?
        // option 1: yes
        /// Creates a Transformation that Operates the two input-Transformation in the given order:
        ///
        /// Let `A`, `B` homogeneous transformations.
        /// `A*B` returns the combined transformation
        /// resulting of first `A` applied on `B*v`, where `v` is a vector.
        impl<RHS: IsHomogeneousMatrix> Mul<RHS> for $t {
            type Output = Transformation;

            fn mul(self, rhs: RHS) -> Transformation {
                let array = fast_matrix_mul(&self.mat, rhs.mat());
                let total_it = fast_matrix_mul(&self.it_mat(), &rhs.it_mat());

                Transformation {
                    mat: array,
                    it_mat: total_it,
                }
            }
        }

        // option 2: no
        impl $t {
            pub fn times_transformation<H: IsHomogeneousMatrix>(&self, rhs: H) -> Transformation {
                let array = fast_matrix_mul(&self.mat, rhs.mat());
                let total_it = fast_matrix_mul(&self.it_mat(), &rhs.it_mat());

                Transformation {
                    mat: array,
                    it_mat: total_it,
                }
            }
        }
    };
}

macro_rules! impl_mul_xrot {
    ($name:ident, $matrix: ident) => {
        impl Mul<$name> for XRotation {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                $name {
                    x: rhs.x,
                    y: self.$matrix[5] * rhs.y + self.$matrix[6] * rhs.z,
                    z: self.$matrix[9] * rhs.y + self.$matrix[10] * rhs.z,
                }
            }
        }
    };
}

macro_rules! impl_mul_yrot {
    ($name:ident, $matrix: ident) => {
        impl Mul<$name> for YRotation {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                $name {
                    x: rhs.x * self.$matrix[0] + rhs.z * self.$matrix[2],
                    y: rhs.y,
                    z: self.$matrix[8] * rhs.x + self.$matrix[10] * rhs.z,
                }
            }
        }
    };
}

macro_rules! impl_mul_zrot {
    ($name:ident, $matrix: ident) => {
        impl Mul<$name> for ZRotation {
            type Output = $name;
            fn mul(self, rhs: $name) -> $name {
                $name {
                    x: rhs.x * self.$matrix[0] + rhs.y * self.$matrix[1],
                    y: rhs.x * self.$matrix[4] + rhs.y * self.$matrix[5],
                    z: rhs.z,
                }
            }
        }
    };
}

// =======================================================================
// FUNCTIONS DEFINITIONS
// =======================================================================

pub fn is_consistent<T: IsHomogeneousMatrix>(matrix: &T) -> bool {
    let it_mat: [f32; 16] = transpose_matrix(matrix.it_mat());
    let mat = fast_matrix_mul(&matrix.mat(), &it_mat);
    let mut result = true;
    for i in 0..16 {
        result = result && are_close(mat[i], IDENTITY_4X4[i]);
    }
    result
}

// =======================================================================
// TRANSFORMATION
// =======================================================================
/// Transformation contains the transformation matrix
/// and its inverse-transposed matrix as unrolled `[f32; 16]` arrays.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transformation {
    // 0..3 are the first row,
    // 4..7 the second row...
    pub mat: [f32; 16],
    pub it_mat: [f32; 16],
}

impl Transformation {
    /// Transformation constructor gets an array and stores it in a Transformation class.
    pub fn new(mat: [f32; 16]) -> Transformation {
        // Add a check if they have properties
        let matrix = inverse_4x4(&mat);
        Transformation {
            mat,
            it_mat: transpose_matrix(&matrix),
        }
    }
}

impl_matrix_operations!(Transformation);
impl Mul<Vector> for Transformation {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        // Note that a homogeneous vector is [vx, vy, vz, 0]
        let mut vec = Vector::new(0.0, 0.0, 0.0);

        // Ugly but fasts - we use properties of the homogeneous space
        vec.x = self.mat[0] * rhs.x + self.mat[1] * rhs.y + self.mat[2] * rhs.z;
        vec.y = self.mat[4] * rhs.x + self.mat[5] * rhs.y + self.mat[6] * rhs.z;
        vec.z = self.mat[8] * rhs.x + self.mat[9] * rhs.y + self.mat[10] * rhs.z;

        let w = self.mat[12] * rhs.x + self.mat[13] * rhs.y + self.mat[14] * rhs.z;
        // For debugging - to be canceled later
        if w.abs() > 1.0e-8 {
            panic!("Invalid transformation!");
        }

        vec
    }
}
impl Mul<Point> for Transformation {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        // Note that a homogeneous vector is [vx, vy, vz, 1]
        let mut point = Point::new(0.0, 0.0, 0.0);

        // Ugly but fasts - we use properties of the homogeneous space
        point.x = self.mat[0] * rhs.x + self.mat[1] * rhs.y + self.mat[2] * rhs.z + self.mat[3];
        point.y = self.mat[4] * rhs.x + self.mat[5] * rhs.y + self.mat[6] * rhs.z + self.mat[7];
        point.z = self.mat[8] * rhs.x + self.mat[9] * rhs.y + self.mat[10] * rhs.z + self.mat[11];

        let w = self.mat[12] * rhs.x + self.mat[13] * rhs.y + self.mat[14] * rhs.z + self.mat[15];
        // For debugging - to be canceled later
        if (w - 1.0).abs() > 1.0e-8 {
            panic!("Invalid transformation!");
        }

        point
    }
}
impl Mul<Normal> for Transformation {
    type Output = Normal;
    fn mul(self, rhs: Normal) -> Normal {
        // Note that a homogeneous vector is [vx, vy, vz, 0]
        let mut nor = Normal::new(0.0, 0.0, 0.0);

        // Ugly but fasts - we use properties of the homogeneous space
        nor.x = self.it_mat[0] * rhs.x + self.it_mat[1] * rhs.y + self.it_mat[2] * rhs.z;
        nor.y = self.it_mat[4] * rhs.x + self.it_mat[5] * rhs.y + self.it_mat[6] * rhs.z;
        nor.z = self.it_mat[8] * rhs.x + self.it_mat[9] * rhs.y + self.it_mat[10] * rhs.z;

        nor
    }
}
// =======================================================================
// SCALING
// =======================================================================
/// Scaling contains the transformation matrix
/// and its inverse-transposed matrix as unrolled `[f32; 16]` arrays
/// for **scaling operators**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scaling {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16],
}

impl_matrix_operations!(Scaling);
impl Scaling {
    /// Scaling Transformation matrix storage
    ///
    /// # Panics
    /// Constructor panics if any of the 3 inputs is 0.0
    pub fn new(diagonal: [f32; 3]) -> Scaling {
        if diagonal.iter().any(|a| are_close(*a, 0.0)) {
            panic!(
                "Wrong inputs in Scaling Matrix definition\
            {:?}",
                diagonal
            );
        }
        let mut array = [0f32; 16];
        let mut it_mat = [0f32; 16];
        for i in 0..3 {
            array[i * 5] = diagonal[i];
            it_mat[i * 5] = 1.0 / array[i * 5];
        }
        array[15] = 1.0;
        it_mat[15] = 1.0;

        Scaling { mat: array, it_mat }
    }
}
impl Mul<Vector> for Scaling {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        Vector {
            x: self.mat[0] * rhs.x,
            y: self.mat[5] * rhs.y,
            z: self.mat[10] * rhs.z,
        }
    }
}
impl Mul<Point> for Scaling {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self.mat[0] * rhs.x,
            y: self.mat[5] * rhs.y,
            z: self.mat[10] * rhs.z,
        }
    }
}
impl Mul<Normal> for Scaling {
    type Output = Normal;
    fn mul(self, rhs: Normal) -> Normal {
        Normal {
            x: self.it_mat[0] * rhs.x,
            y: self.it_mat[5] * rhs.y,
            z: self.it_mat[10] * rhs.z,
        }
    }
}
// =======================================================================
// TRANSLATION
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Translation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16],
}
impl_matrix_operations!(Translation);
impl Translation {
    /// Translation transformation constructor
    pub fn new(k: Vector) -> Self {
        let mat = [
            1.0, 0.0, 0.0, k.x, 0.0, 1.0, 0.0, k.y, 0.0, 0.0, 1.0, k.z, 0.0, 0.0, 0.0, 1.0,
        ];

        let inverse_transposed = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -k.x, -k.y, -k.z, 1.0,
        ];

        Self {
            mat,
            it_mat: inverse_transposed,
        }
    }
}
impl Mul<Vector> for Translation {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        rhs
    }
}
impl Mul<Normal> for Translation {
    type Output = Normal;
    fn mul(self, rhs: Normal) -> Normal {
        rhs
    }
}
impl Mul<Point> for Translation {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self.mat[3] + rhs.x,
            y: self.mat[7] + rhs.y,
            z: self.mat[11] + rhs.z,
        }
    }
}

// =======================================================================
// ROTATION AROUND X-AXIS
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct XRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16],
}

impl XRotation {
    /// Returns a rotation around the x-axis.
    ///
    /// The input must be considered in ste-radiants
    pub fn new(theta: f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        let mat = [
            1.0, 0.0, 0.0, 0.0, 0.0, cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        Self { mat, it_mat: mat }
    }
}
impl_matrix_operations!(XRotation);
impl_mul_xrot!(Vector, mat);
impl_mul_xrot!(Normal, it_mat);
impl_mul_xrot!(Point, mat);

// =======================================================================
// ROTATION AROUND Y-AXIS
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct YRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16],
}

impl YRotation {
    pub fn new(theta: f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        let mat = [
            cos, 0.0, sin, 0.0, 0.0, 1.0, 0.0, 0.0, -sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        Self { mat, it_mat: mat }
    }
}
impl_matrix_operations!(YRotation);
impl_mul_yrot!(Vector, mat);
impl_mul_yrot!(Normal, it_mat);
impl_mul_yrot!(Point, mat);

// =======================================================================
// ROTATION AROUND Z-AXIS
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ZRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16],
}

impl ZRotation {
    pub fn new(theta: f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        let mat = [
            cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        Self { mat, it_mat: mat }
    }
}
impl_matrix_operations!(ZRotation);
impl_mul_zrot!(Vector, mat);
impl_mul_zrot!(Normal, it_mat);
impl_mul_zrot!(Point, mat);

// =======================================================================
// TESTS
// =======================================================================
#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use crate::functions::{
        IDENTITY_4X4, are_close, equal_matrices, fast_matrix_mul, inverse_4x4, transpose_matrix,
    };
    use crate::geometry::{Normal, Point, Vector, is_close};
    use crate::transformations::{
        IsHomogeneousMatrix, Scaling, Transformation, Translation, XRotation, YRotation, ZRotation,
        is_consistent,
    };
    use std::f32::consts;

    // Testing constants
    /* Scale matrix     *       Translation
    [2.0                   [1.0            3.0
         2.0                     1.0      -2.0
            2.0                      1.0   5.0
               1.0]                        1.0]
    */
    /// Transformation obtained from a Translation and a Scaling transformation
    static MAT1: [f32; 16] = [
        2.0, 0.0, 0.0, 6.0, 0.0, 2.0, 0.0, -4.0, 0.0, 0.0, 2.0, 10.0, 0.0, 0.0, 0.0, 1.0,
    ];

    static INVERSE_MAT1: [f32; 16] = [
        0.5, 0.0, 0.0, -3.0, 0.0, 0.5, 0.0, 2.0, 0.0, 0.0, 0.5, -5.0, 0.0, 0.0, 0.0, 1.0,
    ];

    static COS_45: f32 = consts::SQRT_2 / 2.0;

    /* Scale matrix     *       Rotation_z(90)      *    Rotation_x(45)
    [1.0                   [0.0 -1.0  0.0  0.0         [1.0
         2.0                1.0  0.0  0.0  0.0               COS  -SIN
            1.0             0.0  0.0  1.0  0.0               SIN   COS
               1.0]         0.0  0.0  0.0  1.0]                         1.0]
    */
    /// Transformation obtained from a Scaling and two Rotations transformation
    static MAT2: [f32; 16] = [
        0.0, -COS_45, COS_45, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, COS_45, COS_45, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    static INVERSE_MAT2: [f32; 16] = [
        0.0, 0.5, 0.0, 0.0, -COS_45, 0.0, COS_45, 0.0, COS_45, 0.0, COS_45, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    #[test]
    fn test_constants() {
        let result = fast_matrix_mul(&MAT1, &INVERSE_MAT1);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
        let result = fast_matrix_mul(&MAT2, &INVERSE_MAT2);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_consistency() {
        let transformation = Transformation {
            mat: MAT1,
            it_mat: transpose_matrix(&INVERSE_MAT1),
        };
        assert!(is_consistent(&transformation));
        // first element is 1.0 in the correct matrix
        let wrong: [f32; 16] = [
            1.0, 0.5, 0.0, 0.0, -COS_45, 0.0, COS_45, 0.0, COS_45, 0.0, COS_45, 0.0, 0.0, 0.0, 0.0,
            1.0,
        ];
        let transformation = Transformation {
            mat: MAT2,
            it_mat: transpose_matrix(&wrong),
        };
        assert!(!is_consistent(&transformation));
        let transformation = Transformation {
            mat: MAT1,
            it_mat: INVERSE_MAT1,
        };
        assert!(!is_consistent(&transformation));
    }

    // - - - - - - - - - - - - -   Transformation   - - - - - - - - - - - - - - -
    #[test]
    fn test_transformation_constructor() {
        let trans = Transformation::new(MAT1);
        assert!(equal_matrices(trans.mat(), &MAT1));
        let it_mat = transpose_matrix(&INVERSE_MAT1);
        assert!(equal_matrices(trans.it_mat(), &it_mat));
        let it_mat = transpose_matrix(&trans.it_mat);
        let result = fast_matrix_mul(&it_mat, trans.mat());
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_transformation_times_matrix() {
        let trans = Transformation::new(MAT1);
        let matrix = Scaling::new([1.0, 2.0, 3.0]);
        let result = trans * matrix;
        // check consistency
        assert!(is_consistent(&result));

        let expected: [f32; 16] = [
            2.0, 0.0, 0.0, 6.0, 0.0, 4.0, 0.0, -4.0, 0.0, 0.0, 6.0, 10.0, 0.0, 0.0, 0.0, 1.0,
        ];
        // Check algorithm
        assert!(equal_matrices(&expected, &result.mat));

        // Check existing implementation and consistency
        let matrix = Transformation::new(MAT2);
        let result = trans * matrix;
        assert!(is_consistent(&result));
        let matrix = XRotation::new(consts::PI / 2.0);
        let result = trans * matrix;
        assert!(is_consistent(&result));
        let matrix = YRotation::new(consts::PI);
        let result = trans * matrix;
        assert!(is_consistent(&result));
        let matrix = ZRotation::new(consts::PI / 2.0);
        let result = trans * matrix;
        assert!(is_consistent(&result));
        let matrix = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let result = trans * matrix;
        assert!(is_consistent(&result));
    }

    #[test]
    fn test_transformation_times_vector_and_normal() {
        let trans = Transformation::new(MAT1);
        let vec = Vector::new(1.0, -2.0, 3.0);
        let result = trans * vec;
        assert_eq!(result, 2.0 * vec);

        let normal = Normal::new(1.0, 2.0, 3.0);
        let result = trans * normal;
        assert_eq!(result, 0.5 * normal);
    }

    #[test]
    fn test_transformation_times_point() {
        let transformation = Transformation::new(MAT1);
        let point = Point::new(1.0, -2.0, 3.0);
        let result = transformation * point;
        let expected = Point::new(8.0, -8.0, 16.0);
        assert_eq!(result, expected);
    }

    // - - - - - - - - - - - - -   Scaling   - - - - - - - - - - - - - - -
    #[test]
    #[should_panic(expected = "Wrong inputs in Scaling Matrix definition")]
    fn test_scaling_constructor() {
        let scaling_matrix = [
            1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let scale = Scaling::new([1.0, 2.0, 3.0]);
        assert!(equal_matrices(scale.mat(), &scaling_matrix));

        let it_scaling_matrix = [
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.5,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / 3.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ];

        assert!(equal_matrices(scale.it_mat(), &it_scaling_matrix));
        let it_mat = transpose_matrix(&scale.it_mat);
        let result = fast_matrix_mul(&it_mat, scale.mat());
        assert!(equal_matrices(&result, &IDENTITY_4X4));

        let _ = Scaling::new([0.0, 2.0, 3.0]);
    }

    #[test]
    fn test_scaling_matrix_mul() {
        // just a quick check, code compiles a
        // nd works for Transformation
        let scale = Scaling::new([1.0, 2.0, 3.0]);
        let matrix = Transformation::new(MAT2);
        let result = scale * matrix;
        assert!(is_consistent(&result));
        let matrix = Scaling::new([1.0, 2.0, 3.0]);
        let result = scale * matrix;
        assert!(is_consistent(&result));
        let matrix = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let result = scale * matrix;
        assert!(is_consistent(&result));
        let matrix = XRotation::new(consts::PI / 2.0);
        let result = scale * matrix;
        assert!(is_consistent(&result));
        let matrix = YRotation::new(consts::PI / 2.0);
        let result = scale * matrix;
        assert!(is_consistent(&result));
        let matrix = ZRotation::new(consts::PI / 2.0);
        let result = scale * matrix;
        assert!(is_consistent(&result));
    }

    #[test]
    fn test_scaling_vector() {
        let v = Vector::new(1.0, 2.0, 3.0);
        let scale = Scaling::new([5.0, 6.0, -10.0]);
        let expected = Vector::new(5.0, 12.0, -30.0);
        assert_eq!(scale * v, expected);
    }

    #[test]
    fn test_scaling_point() {
        let p = Point::new(1.0, 2.0, 3.0);
        let scale = Scaling::new([5.0, 6.0, -10.0]);
        let expected = Point::new(5.0, 12.0, -30.0);
        assert_eq!(scale * p, expected);
    }

    #[test]
    fn test_scaling_normal() {
        let n = Normal::new(1.0, 2.0, 3.0);
        let scale = Scaling::new([2.0, 4.0, 10.0]);

        // The it_mat is 1/value
        let expected = Normal::new(1.0 / 2.0, 2.0 / 4.0, 3.0 / 10.0);

        let result = scale * n;

        assert!(are_close(result.x, expected.x));
        assert!(are_close(result.y, expected.y));
        assert!(are_close(result.z, expected.z));
    }

    // - - - - - - - - - - - - -   Translation   - - - - - - - - - - - - - - -
    #[test]
    fn test_translation_constructor() {
        let v = Vector::new(1.0, 2.0, 3.0);
        let translation = Translation::new(v);
        let expected: [f32; 16] = [
            1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 1.0, 3.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(translation.mat(), &expected));
        let expected: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, -2.0, -3.0, 1.0,
        ];
        assert!(equal_matrices(translation.it_mat(), &expected));

        let it_mat = transpose_matrix(&translation.it_mat);
        let result = fast_matrix_mul(&it_mat, translation.mat());
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_translation_matrix_mul() {
        // just a quick check, code compiles a
        // nd works for Transformation
        let translation = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let matrix = Transformation::new(MAT2);
        let result = translation * matrix;
        assert!(is_consistent(&result));
        let matrix = Scaling::new([1.0, 2.0, 3.0]);
        let result = translation * matrix;
        assert!(is_consistent(&result));
        let matrix = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let result = translation * matrix;
        assert!(is_consistent(&result));
        let matrix = XRotation::new(consts::PI / 2.0);
        let result = translation * matrix;
        assert!(is_consistent(&result));
        let matrix = YRotation::new(consts::PI / 2.0);
        let result = translation * matrix;
        assert!(is_consistent(&result));
        let matrix = ZRotation::new(consts::PI / 2.0);
        let result = translation * matrix;
        assert!(is_consistent(&result));
    }

    #[test]
    fn test_translation_times_vector_and_normal() {
        let trans = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let vec = Vector::new(1.0, -2.0, 3.0);
        assert_eq!(trans * vec, vec);

        let normal = Normal::new(1.0, 2.0, 3.0);
        assert_eq!(trans * normal, normal);
    }

    #[test]
    fn test_translation_times_point() {
        let transformation = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let point = Point::new(1.0, -2.0, 3.0);
        let result = transformation * point;
        let expected = Point::new(2.0, 0.0, 6.0);
        assert_eq!(result, expected);
    }

    // - - - - - - - - - - - - -   XRotation   - - - - - - - - - - - - - - -
    #[test]
    fn test_rotation_x_constructor() {
        let angle = consts::FRAC_PI_3;
        let rotation = XRotation::new(angle);
        let sin = angle.sin();
        let cos = angle.cos();
        let matrix: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(rotation.mat(), &matrix));
        assert!(equal_matrices(&rotation.it_mat, &matrix));
        let matrix = transpose_matrix(rotation.it_mat());
        let result = fast_matrix_mul(rotation.mat(), &matrix);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_rotation_x_matrix_mul() {
        let rotation = XRotation::new(consts::FRAC_PI_3);
        let matrix = Transformation::new(MAT1);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = Scaling::new([1.0, 2.0, 3.0]);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = XRotation::new(consts::PI);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = YRotation::new(consts::FRAC_PI_3);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = ZRotation::new(consts::PI / 2.0);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
    }

    #[test]
    fn test_rotation_x_vectorial() {
        let rotation = XRotation::new(consts::FRAC_PI_3);
        let vec = Vector::new(1.0, 2.0, 3.0);
        let result = rotation * vec;
        let expected = Vector {
            x: 1.0,
            y: 1.0 - 3.0_f32.sqrt() * 3.0 / 2.0,
            z: 3.0_f32.sqrt() + 1.5,
        };
        assert!(is_close(result, expected));
        let nor = Normal::new(1.0, 2.0, 3.0);
        let result = rotation * nor;
        let expected = Normal {
            x: 1.0,
            y: 1.0 - 3.0_f32.sqrt() * 3.0 / 2.0,
            z: 3.0_f32.sqrt() + 1.5,
        };
        assert!(is_close(result, expected));
        let point = Point::new(1.0, 2.0, 3.0);
        let result = rotation * point;
        let expected = Point {
            x: 1.0,
            y: 1.0 - 3.0_f32.sqrt() * 3.0 / 2.0,
            z: 3.0_f32.sqrt() + 1.5,
        };
        assert!(is_close(result, expected));
    }
    // - - - - - - - - - - - - -   YRotation   - - - - - - - - - - - - - - -
    #[test]
    fn test_rotation_y_constructor() {
        let theta = consts::FRAC_PI_6;
        let rotation = YRotation::new(theta);
        let cos = theta.cos();
        let sin = theta.sin();
        let matrix = [
            cos, 0.0, sin, 0.0, 0.0, 1.0, 0.0, 0.0, -sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(rotation.mat(), &matrix));
        assert!(equal_matrices(&rotation.it_mat, &matrix));
        let matrix = transpose_matrix(rotation.it_mat());
        let result = fast_matrix_mul(rotation.mat(), &matrix);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_rotation_y_matrix_mul() {
        let rotation = YRotation::new(consts::FRAC_PI_3);
        let matrix = Transformation::new(MAT1);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = Scaling::new([1.0, 2.0, 3.0]);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = XRotation::new(consts::PI);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = YRotation::new(consts::FRAC_PI_3);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = ZRotation::new(consts::PI / 2.0);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
    }

    #[test]
    fn test_rotation_y_vectorial() {
        let rotation = YRotation::new(consts::FRAC_PI_3);
        println!("{:?}", rotation.mat);
        let vec = Vector::new(1.0, 2.0, 3.0);
        let result = rotation * vec;
        let expected = Vector {
            x: 0.5 + 3.0_f32.sqrt() * 3.0 / 2.0,
            y: 2.0,
            z: -3.0_f32.sqrt() / 2.0 + 1.5,
        };
        assert!(is_close(result, expected), "{}\n{}", result, expected);
        let nor = Normal::new(1.0, 2.0, 3.0);
        let result = rotation * nor;
        let expected = Normal {
            x: 0.5 + 3.0_f32.sqrt() * 3.0 / 2.0,
            y: 2.0,
            z: -3.0_f32.sqrt() / 2.0 + 1.5,
        };
        assert!(is_close(result, expected));
        let point = Point::new(1.0, 2.0, 3.0);
        let result = rotation * point;
        let expected = Point {
            x: 0.5 + 3.0_f32.sqrt() * 3.0 / 2.0,
            y: 2.0,
            z: -3.0_f32.sqrt() / 2.0 + 1.5,
        };
        assert!(is_close(result, expected));
    }

    // - - - - - - - - - - - - -   ZRotation   - - - - - - - - - - - - - - -
    #[test]
    fn test_rotation_z_constructor() {
        let theta = consts::FRAC_PI_3;
        let rotation = ZRotation::new(theta);
        let cos = theta.cos();
        let sin = theta.sin();
        let matrix = [
            cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(rotation.mat(), &matrix));
        assert!(equal_matrices(&rotation.it_mat, &matrix));
        let matrix = transpose_matrix(rotation.it_mat());
        let result = fast_matrix_mul(rotation.mat(), &matrix);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_rotation_z_matrix_mul() {
        let rotation = ZRotation::new(consts::FRAC_PI_3);
        let matrix = Transformation::new(MAT1);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = Translation::new(Vector::new(1.0, 2.0, 3.0));
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = Scaling::new([1.0, 2.0, 3.0]);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = XRotation::new(consts::PI);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = YRotation::new(consts::FRAC_PI_3);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
        let matrix = ZRotation::new(consts::PI / 2.0);
        let result = rotation * matrix;
        assert!(is_consistent(&result));
    }

    #[test]
    fn test_rotation_z_vectorial() {
        let rotation = ZRotation::new(consts::FRAC_PI_3);
        println!("{:?}", rotation.mat);
        let vec = Vector::new(1.0, 2.0, 3.0);
        let result = rotation * vec;
        let expected = Vector {
            x: 0.5 - 3.0_f32.sqrt(),
            y: 3.0_f32.sqrt() / 2.0 + 1.0,
            z: 3.0,
        };
        assert!(is_close(result, expected), "{}\n{}", result, expected);
        let nor = Normal::new(1.0, 2.0, 3.0);
        let result = rotation * nor;
        let expected = Normal {
            x: 0.5 - 3.0_f32.sqrt(),
            y: 3.0_f32.sqrt() / 2.0 + 1.0,
            z: 3.0,
        };
        assert!(is_close(result, expected));
        let point = Point::new(1.0, 2.0, 3.0);
        let result = rotation * point;
        let expected = Point {
            x: 0.5 - 3.0_f32.sqrt(),
            y: 3.0_f32.sqrt() / 2.0 + 1.0,
            z: 3.0,
        };
        assert!(is_close(result, expected));
    }
}

// -------------------------------------------------------------
//                            NOTES
// -------------------------------------------------------------
