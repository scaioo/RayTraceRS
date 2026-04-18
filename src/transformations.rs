use std::process::Output;
use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::functions::{fast_matrix_mul, inverse_4x4, transpose_matrix};
use anyhow::Result;
use crate::geometry::{Vector, Point, Normal};
// =======================================================================
// TRAIT DEFINITIONS
// =======================================================================
/// This trait is the Marker Trait for Transformations
pub trait IsHomogeneousMatrix {
    fn mat(&self) -> &[f32; 16];

    fn it_mat(&self) -> &[f32; 16];
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

            fn it_mat(&self) -> &[f32; 16] {
                &self.it_mat
            }
        }

        // -----------------------   Matrix * Matrix    -------------------------

        // Do we want to use the * symbol for the matrix-rhs product?
        // option 1: yes
        impl<RHS: IsHomogeneousMatrix> Mul<RHS> for $t {
            type Output = GenericTransformation;

            fn mul(self, rhs: RHS) -> GenericTransformation {
                let array = fast_matrix_mul(&self.mat, rhs.mat());
                let inverse  = inverse_4x4(&array);
                GenericTransformation {
                    mat: array,
                    it_mat: transpose_matrix(&inverse),
                }
            }
        }

        // option 2: no
        impl $t {
            pub fn times_transformation<H: IsHomogeneousMatrix>(&self, matrix : H) -> GenericTransformation {
                let mat : [f32; 16] = fast_matrix_mul(&self.mat, matrix.mat());
                let inverse  = inverse_4x4(&mat);
                GenericTransformation{
                    mat: mat,
                    it_mat: transpose_matrix(&inverse),
                }
            }
        }

        // -----------------------   Matrix * Vector    -------------------------

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
    };
}

// =======================================================================
// STRUCT DEFINITIONS
// =======================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GenericTransformation {
    // 0..3 are the first row,
    // 4..7 the second row...
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scaling {
    pub mat: [f32; 16],
    pub it_mat: [f32; 16]
}

impl_matrix_operations!(Scaling);

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
    use crate::transformations::Scaling;

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

    //Many tests to be writte....

}




// -------------------------------------------------------------
//                            NOTES
// -------------------------------------------------------------


/*
Draft:
- For each transformation there is a struct that generates
    a 4x4 matrix with one array type and its inverse.
- Create macros to implement the various operations with vectors, normals and points.


Else:
Gemini suggested me this:
    "nalgebra.org: La documentazione è bellissima
    e spiega come creare matrici specializzate (2x2, 3x3, 4x4)
    che sono ottimizzate per la velocità.
consider switching to this!

 */