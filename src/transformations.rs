use std::process::Output;
use std::ops::{Mul};
use crate::functions::{are_close, fast_matrix_mul, inverse_4x4, transpose_matrix, IDENTITY_4X4};
use crate::geometry::{Vector, Point, Normal};

// =======================================================================
// STRUCT DEFINITIONS
// =======================================================================
/// Transformation contains the transformation matrix
/// and its inverse-transposed matrix as unrolled `[f32; 16]` arrays.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transformation {
    // 0..3 are the first row,
    // 4..7 the second row...
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

/// Scaling contains the transformation matrix
/// and its inverse-transposed matrix as unrolled `[f32; 16]` arrays
/// for **scaling operators**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scaling {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Translation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct XRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct YRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ZRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

// =======================================================================
// CONSTRUCTORS
// =======================================================================
impl Transformation {
    /// Transformation constructor gets an array and stores it in a Transformation class.
    pub fn new(mat : [f32;16]) -> Transformation {
        // Add a check if they have properties
        let matrix = inverse_4x4(&mat);
        Transformation{
            mat,
            it_mat: transpose_matrix(&matrix)
        }
    }
}
impl Scaling {
    /// Scaling Transformation matrix storage
    ///
    /// # Panics
    /// Constructor panics if any of the 3 inputs is 0.0
    pub fn new(diagonal : [f32;3]) -> Scaling {
        if diagonal.iter().any(|a| are_close(*a,0.0)) {
            panic!("Wrong inputs in Scaling Matrix definition\
            {:?}", diagonal);
        }
        let mut array = [0f32; 16];
        let mut it_mat = [0f32; 16];
        for i in 0..3 {
            array[i*5] = diagonal[i];
            it_mat[i*5] = 1.0/ array[i*5];
        }
        array[15] = 1.0;
        it_mat[15] = 1.0;

        Scaling{
            mat : array,
            it_mat: it_mat
        }
    }
}
impl Translation {
    /// Translation transformation constructor
    pub fn new(k: Vector) -> Self {
        let mat = [
            1.0, 0.0, 0.0, k.x,
            0.0, 1.0, 0.0, k.y,
            0.0, 0.0, 1.0, k.z,
            0.0, 0.0, 0.0, 1.0,
        ];

        let inverse_transposed = [
            1.0,  0.0,  0.0,  0.0,
            0.0,  1.0,  0.0,  0.0,
            0.0,  0.0,  1.0,  0.0,
            -k.x, -k.y, -k.z, 1.0,
        ];

        Self {
            mat,
            it_mat: inverse_transposed
        }
    }
}
impl XRotation {
    /// Returns a rotation around the x-axis.
    ///
    /// The input must be considered in ste-radiants
    pub fn new(theta : f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        let mat = [
            1.0, 0.0, 0.0, 0.0,
            0.0, cos, -sin, 0.0,
            0.0, sin, cos, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];
        Self{
            mat,
            it_mat: mat
        }
    }
}
impl YRotation {
    pub fn new(theta : f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        let mat = [
            cos, 0.0, sin, 0.0,
            0.0, 1.0, 0.0, 0.0,
            -sin, 0.0, cos, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];
        Self{
            mat,
            it_mat: mat
        }
    }
}
impl ZRotation {
    pub fn new(theta : f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        let mat = [
            cos, -sin, 0.0, 0.0,
            sin, cos, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];
        Self{
            mat,
            it_mat: mat
        }
    }
}
// =======================================================================
// FUNCTIONS DEFINITIONS
// =======================================================================

pub fn is_consistent<T : IsHomogeneousMatrix>(matrix : &T) -> bool {
    let it_mat :[f32;16] = transpose_matrix(matrix.it_mat());
    let mat = fast_matrix_mul(&matrix.mat(), &it_mat);
    let mut result = true;
    for i in 0..16{
        result = result && mat[i] == IDENTITY_4X4[i];
    }
    result
}

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
            pub fn times_transformation<H: IsHomogeneousMatrix>(&self, rhs : H) -> Transformation {
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
                $name{
                    x : rhs.x,
                    y : self.$matrix[5] * rhs.y + self.$matrix[6] * rhs.z,
                    z : self.$matrix[9] * rhs.y + self.$matrix[10] * rhs.z
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
                $name{
                    x : rhs.x * self.$matrix[0] + rhs.y * self.$matrix[2],
                    y : rhs.y,
                    z : self.$matrix[8] * rhs.x + self.$matrix[10] * rhs.z
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
                $name{
                    x : rhs.x * self.$matrix[0] + rhs.y * self.$matrix[1],
                    y : rhs.x * self.$matrix[4] + rhs.y * self.$matrix[5],
                    z : rhs.z
                }
            }
        }
    };
}

// =======================================================================
// STRUCT DEFINITIONS
// =======================================================================

impl_matrix_operations!(Transformation);

impl Mul<Vector> for Transformation  {
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

impl_matrix_operations!(Scaling);

impl Mul<Vector> for Scaling {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        Vector {
            x : self.mat[0] * rhs.x,
            y: self.mat[5] * rhs.y,
            z: self.mat[10] * rhs.z
        }
    }
}

impl Mul<Point> for Scaling {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point{
            x : self.mat[0] * rhs.x,
            y: self.mat[5] * rhs.y,
            z: self.mat[10] * rhs.z
        }
    }
}

impl Mul<Normal> for Scaling {
    type Output = Normal;
    fn mul(self, rhs: Normal) -> Normal {
        Normal{
            x : self.it_mat[0] * rhs.x,
            y : self.it_mat[5] * rhs.y,
            z : self.it_mat[10] * rhs.z
        }
    }
}


impl_matrix_operations!(Translation);

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
        Point{
            x : self.mat[3] + rhs.x,
            y : self.mat[7] + rhs.y,
            z : self.mat[11] + rhs.z
        }
    }
}

impl_matrix_operations!(XRotation);
impl_mul_xrot!(Vector, mat);
impl_mul_xrot!(Normal, it_mat);
impl_mul_xrot!(Point, mat);

impl_matrix_operations!(YRotation);
impl_mul_yrot!(Vector, mat);
impl_mul_yrot!(Normal, it_mat);
impl_mul_yrot!(Point, mat);

impl_matrix_operations!(ZRotation);
impl_mul_zrot!(Vector, mat);
impl_mul_zrot!(Normal, it_mat);
impl_mul_zrot!(Point, mat);

/*
TODO [function][test]
- [X][X] Scaling constructor
- [][] Matrix product
- [][] Rotation constructor
- [][] Translation constructor
- [][] Property tests (i.e. for translations T^-1(k) = T(-k))
- [][] Macro that:
    Transformation * Transformation → Transformation
    Transformation * Point → Point
    Transformation * Vec → Vec.
    Transformation * Normal → Normal.
 */

// =======================================================================
// TESTS
// =======================================================================
#[cfg(test)]
mod test {
    use crate::functions::{are_close, equal_matrices, fast_matrix_mul, inverse_4x4, transpose_matrix, IDENTITY_4X4};
    use crate::geometry::{Vector, Point, Normal};
    use crate::transformations::{Transformation, Scaling, IsHomogeneousMatrix, Translation, YRotation, XRotation, ZRotation};

    // Testing constants
    /* Scale matrix     *       Translation
     [2.0                   [1.0            3.0
          2.0                     1.0      -2.0
             2.0                      1.0   5.0
                1.0]                        1.0]
     */
    /// Transformation obtained from a Translation and a Scaling transformation
    static MAT1:[f32;16] = [
        2.0, 0.0, 0.0, 6.0,
        0.0, 2.0, 0.0, -4.0,
        0.0, 0.0, 2.0, 10.0,
        0.0, 0.0, 0.0, 1.0,
    ];

    static INVERSE_MAT1 : [f32;16] =[
        0.5, 0.0, 0.0, -3.0,
        0.0, 0.5, 0.0, 2.0,
        0.0, 0.0, 0.5, -5.0,
        0.0, 0.0, 0.0, 1.0,
    ];

    static COS_45:f32 = std::f32::consts::SQRT_2 /2.0;

    /* Scale matrix     *       Rotation_z(90)      *    Rotation_x(45)
    [1.0                   [0.0 -1.0  0.0  0.0         [1.0
         2.0                1.0  0.0  0.0  0.0               COS  -SIN
            1.0             0.0  0.0  1.0  0.0               SIN   COS
               1.0]         0.0  0.0  0.0  1.0]                         1.0]
    */
    /// Transformation obtained from a Scaling and two Rotations transformation
    static MAT2 :[f32;16]=[
        0.0, - COS_45, COS_45, 0.0,
        2.0,    0.0,    0.0,   0.0,
        0.0,   COS_45, COS_45, 0.0,
        0.0,    0.0,    0.0,   1.0,
    ];

    static INVERSE_MAT2 : [f32;16] =[
        0.0, 0.5, 0.0, 0.0,
        - COS_45, 0.0, COS_45, 0.0,
        COS_45, 0.0, COS_45, 0.0,
        0.0, 0.0, 0.0, 1.0
    ];

    #[test]
    fn test_constants(){
        let result = fast_matrix_mul(&MAT1, &INVERSE_MAT1);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
        let result = fast_matrix_mul(&MAT2, &INVERSE_MAT2);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_transformation_constructor(){
        let trans = Transformation::new(MAT1);
        assert!(equal_matrices(trans.mat(), &MAT1));
        let it_mat = transpose_matrix(&INVERSE_MAT1);
        assert!(equal_matrices(trans.it_mat(), &it_mat));
        let it_mat = transpose_matrix(&trans.it_mat);
        let result = fast_matrix_mul(&it_mat, trans.mat());
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    #[should_panic(expected = "Wrong inputs in Scaling Matrix definition")]
    fn test_scaling_constructor(){
        let scaling_matrix = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 2.0, 0.0, 0.0,
            0.0, 0.0, 3.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let scale = Scaling::new([1.0,2.0,3.0]);
        assert!(equal_matrices(scale.mat(), &scaling_matrix));

        let it_scaling_matrix = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 0.5, 0.0, 0.0,
            0.0, 0.0, 1.0/3.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        assert!(equal_matrices(scale.it_mat(), &it_scaling_matrix));
        let it_mat = transpose_matrix(&scale.it_mat);
        let result = fast_matrix_mul(&it_mat, scale.mat());
        assert!(equal_matrices(&result, &IDENTITY_4X4));

        let _ = Scaling::new([0.0,2.0,3.0]);
    }

    #[test]
    fn test_translation_constructor(){
        let v = Vector::new(1.0,2.0,3.0);
        let translation = Translation::new(v);
        let expected : [f32;16] = [
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 2.0,
            0.0, 0.0, 1.0, 3.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(translation.mat(), &expected));
        let expected : [f32;16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -1.0, -2.0, -3.0, 1.0
        ];
        assert!(equal_matrices(translation.it_mat(), &expected));

        let it_mat = transpose_matrix(&translation.it_mat);
        let result = fast_matrix_mul(&it_mat, translation.mat());
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_rotation_x_constructor(){
        let angle = std::f32::consts::FRAC_PI_3;
        let rotation = XRotation::new(angle);
        let sin = angle.sin();
        let cos = angle.cos();
        let matrix : [f32;16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, cos, -sin, 0.0,
            0.0, sin, cos, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        assert!(equal_matrices(rotation.mat(), &matrix));
        assert!(equal_matrices(&rotation.it_mat, &matrix));
        let matrix = transpose_matrix(rotation.it_mat());
        let result = fast_matrix_mul(rotation.mat(),&matrix);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_rotation_y_constructor(){
        let theta = std::f32::consts::FRAC_PI_6;
        let rotation = YRotation::new(theta);
        let cos = theta.cos();
        let sin = theta.sin();
        let matrix = [
            cos, 0.0, sin, 0.0,
            0.0, 1.0, 0.0, 0.0,
            -sin, 0.0, cos, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];
        assert!(equal_matrices(rotation.mat(), &matrix));
        assert!(equal_matrices(&rotation.it_mat, &matrix));
        let matrix = transpose_matrix(rotation.it_mat());
        let result = fast_matrix_mul(rotation.mat(),&matrix);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_rotation_z_constructor(){
        let theta = std::f32::consts::FRAC_PI_3;
        let rotation = ZRotation::new(theta);
        let cos = theta.cos();
        let sin = theta.sin();
        let matrix = [
            cos, -sin, 0.0, 0.0,
            sin, cos, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];
        assert!(equal_matrices(rotation.mat(), &matrix));
        assert!(equal_matrices(&rotation.it_mat, &matrix));
        let matrix = transpose_matrix(rotation.it_mat());
        let result = fast_matrix_mul(rotation.mat(),&matrix);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }

    #[test]
    fn test_scaling_vector(){
        let v = Vector::new(1.0, 2.0, 3.0);
        let scale = Scaling::new([5.0, 6.0, -10.0]);
        let expected = Vector::new(5.0, 12.0, -30.0);
        assert_eq!(scale * v, expected);
    }

    #[test]
    fn test_scaling_point(){
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
        let expected = Normal::new(
            1.0 / 2.0,
            2.0 / 4.0,
            3.0 / 10.0,
        );

        let result = scale * n;

        assert!(are_close(result.x, expected.x));
        assert!(are_close(result.y, expected.y));
        assert!(are_close(result.z, expected.z));
    }

    //Many tests to be writte....

}




// -------------------------------------------------------------
//                            NOTES
// -------------------------------------------------------------
