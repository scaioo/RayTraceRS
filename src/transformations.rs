use std::process::Output;
use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::functions::{fast_matrix_mul, inverse_4x4, transpose_matrix, IDENTITY_4X4};
use crate::geometry::{Vector, Point, Normal};
// =======================================================================
// TRAIT DEFINITIONS
// =======================================================================
/// This trait is the Marker Trait for Transformations
pub trait IsHomogeneousMatrix {
    fn mat(&self) -> &[f32; 16];

    fn it(&self) -> &[f32; 16];

    fn is_homogeneous(&self) -> bool;
}

// =======================================================================
// MACRO DEFINITIONS
// =======================================================================

#[macro_export]
macro_rules! impl_matrix_operations {
    ($t: ident) => {
        // Note that there might be a w as last coordinate of the homogeneous vector!
        // If none the standard is given by the type!
        // -----------------------       Marker trait      -------------------------
        impl IsHomogeneousMatrix for $t {
            fn mat(&self) -> &[f32; 16] {
                &self.mat
            }

            fn is_homogeneous(&self) -> bool {
                true
            }

            fn it(&self) -> &[f32; 16] {
                &self.it_mat
            }


        }

        // -----------------------   Matrix * Matrix    -------------------------


        // Do we want to use the * symbol for the matrix-rhs product?
        // option 1: yes
        impl<RHS: IsHomogeneousMatrix> Mul<RHS> for $t {
            type Output = Transformation;

            fn mul(self, rhs: RHS) -> Transformation {
                let array = fast_matrix_mul(&self.mat, rhs.mat());
                let total_it = fast_matrix_mul(&self.it(), &rhs.it());

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
                let total_it = fast_matrix_mul(&self.it(), &rhs.it());

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transformation {
    // 0..3 are the first row,
    // 4..7 the second row...
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

impl Transformation {
    pub fn new(mat : [f32;16]) -> Transformation {
        let matrix = inverse_4x4(&mat);
        Transformation{
            mat,
            it_mat: transpose_matrix(&matrix)
        }
    }
}

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

        let w = self.it_mat[12] * rhs.x + self.it_mat[13] * rhs.y + self.it_mat[14] * rhs.z;                // For debugging - to be canceled later
        // For debugging
        if w.abs() > 1.0e-8 {
            panic!("Invalid transformation!");
        }

        nor
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scaling {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

impl Scaling {
    pub fn new(diagonal : [f32;3]) -> Scaling {
        if diagonal.iter().any(|a| *a == 0.0) {
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


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Translation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

impl Translation {
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


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct XRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

impl XRotation {
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

impl_matrix_operations!(XRotation);
impl_mul_xrot!(Vector, mat);
impl_mul_xrot!(Normal, it_mat);
impl_mul_xrot!(Point, mat);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct YRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
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

impl_matrix_operations!(YRotation);
impl_mul_yrot!(Vector, mat);
impl_mul_yrot!(Normal, it_mat);
impl_mul_yrot!(Point, mat);


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ZRotation {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
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

impl_matrix_operations!(ZRotation);
impl_mul_zrot!(Vector, mat);
impl_mul_zrot!(Normal, it_mat);
impl_mul_zrot!(Point, mat);


// =======================================================================
// FUNCTIONS DEFINITIONS
// =======================================================================

pub fn is_consistent<T : IsHomogeneousMatrix>(matrix : &T) -> bool {
    let mat = fast_matrix_mul(&matrix.mat(), &matrix.it());
    let mut result = true;
    for i in 0..16{
        result = result && mat[i] == IDENTITY_4X4[i];
    }
    result
}

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
    use crate::functions::are_close;
    use crate::geometry::{Vector, Point, Normal};
    use crate::transformations::{Transformation, Scaling};

    #[test]
    fn test_transformation_constructor(){
        panic!("WRITE TEST!");
    }


    #[test]
    #[should_panic(expected = "Wrong inputs in Scaling Matrix definition")]
    fn test_scaling_constructor(){
        let scale = Scaling::new([1.0, 2.0, 3.0]);
        let mat = [
                1.0, 0.0, 0.0, 0.0,
                0.0, 2.0, 0.0, 0.0,
                0.0, 0.0, 3.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ];
        let mut it_mat = [0.0; 16];
        for i in 0..4{
            it_mat[i*5] = 1.0 / mat[i*5] ;
        }
        assert_eq!(mat, scale.mat);
        assert_eq!(it_mat, scale.it_mat);
        let _ = Scaling::new([0.0,2.0,3.0]);
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


/*
// -----------------------   Old code in macro: Matrix * Vector    -------------------------

        impl Mul<Vector> for $t {
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

        impl Mul<Point> for $t {
            type Output = Point;
            fn mul(self, rhs: Point) -> Point {
                // Note that a homogeneous vector is [vx, vy, vz, 1]
                let mut point = Point::new(0.0, 0.0, 0.0);

                // Ugly but fasts - we use properties of the homogeneous space
                point.x = self.mat[0] * rhs.x + self.mat[1] * rhs.y + self.mat[2] * rhs.z;
                point.y = self.mat[4] * rhs.x + self.mat[5] * rhs.y + self.mat[6] * rhs.z;
                point.z = self.mat[8] * rhs.x + self.mat[9] * rhs.y + self.mat[10] * rhs.z;

                let w = self.mat[12] * rhs.x + self.mat[13] * rhs.y + self.mat[14] * rhs.z;
                // For debugging - to be canceled later
                if (w - 1.0).abs() > 1.0e-8 {
                    panic!("Invalid transformation!");
                }

                point
            }
        }

        impl Mul<Normal> for $t {
            type Output = Normal;
            fn mul(self, rhs: Normal) -> Normal {
                // Note that a homogeneous vector is [vx, vy, vz, 0]
                let mut nor = Normal::new(0.0, 0.0, 0.0);

                // Ugly but fasts - we use properties of the homogeneous space
                nor.x = self.it_mat[0] * rhs.x + self.it_mat[1] * rhs.y + self.it_mat[2] * rhs.z;
                nor.y = self.it_mat[4] * rhs.x + self.it_mat[5] * rhs.y + self.it_mat[6] * rhs.z;
                nor.z = self.it_mat[8] * rhs.x + self.it_mat[9] * rhs.y + self.it_mat[10] * rhs.z;

                let w = self.it_mat[12] * rhs.x + self.it_mat[13] * rhs.y + self.it_mat[14] * rhs.z;                // For debugging - to be canceled later
                // For debugging
                if w.abs() > 1.0e-8 {
                    panic!("Invalid transformation!");
                }

                nor
            }
        }


impl Mul<Vector> for XRotation {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        Vector{
            x : rhs.x,
            y : self.mat[5] * rhs.y + self.mat[6] * rhs.z,
            z : self.mat[9] * rhs.y + self.mat[10] * rhs.z
        }
    }
}

impl Mul<Point> for XRotation {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point{
            x : rhs.x,
            y : self.mat[5] * rhs.y + self.mat[6] * rhs.z,
            z : self.mat[9] * rhs.y + self.mat[10] * rhs.z
        }
    }
}

impl Mul<Normal> for XRotation {
    type Output = Normal;
    fn mul(self, rhs: Normal) -> Normal {
        Normal{
            x : rhs.x,
            y : self.mat[5] * rhs.y + self.mat[6] * rhs.z,
            z : self.mat[9] * rhs.y + self.mat[10] * rhs.z
        }
    }
}
 */