use std::process::Output;
use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::functions::{fast_matrix_mul, inverse_4x4};
use anyhow::Result;
use crate::geometry::{Vector, Point, Normal};
// =======================================================================
// TRAIT DEFINITIONS
// =======================================================================

// =======================================================================
// MACRO DEFINITIONS
// =======================================================================

#[macro_export]
macro_rules! impl_matrix_operations {
    ($t: ident) => {
        // Note that there might be a w as last coordinate of the homogeneous vector!
        // If none the standard is given by the type!

        // -----------------------   Matrix times scalar   -------------------------
        impl Mul<f32> for $t {
            type Output = $t;

            fn mul(self, rhs: f32) -> $t {
                let mut new_mat = self.mat;
                let mut new_inv = self.inverse;

                for i in 0..16 {
                    new_mat[i] *= rhs;
                    new_inv[i] *= rhs;
                }

                $t {
                    mat: new_mat,
                    inverse: new_inv,
                }
            }
        }

        impl Mul<$t> for f32 {
            type Output = $t;

            fn mul(self, rhs: $t) -> $t {
                rhs * self
            }
        }

        impl Div<f32> for $t {
            type Output = $t;
            fn div(self, rhs: f32) -> $t {
                if rhs == 0.0 || rhs.is_nan(){
                    panic!("Invalid quotient!");
                }
                self * (1.0 / rhs)
            }
        }

        // -----------------------   Matrix * Matrix    -------------------------

        // Do we want to use the * symbol for the matrix-rhs product?
        // option 1: yes
        impl Mul<$t> for $t {
            type Output = GenericTransformation;
            fn mul(self, rhs: $t) -> GenericTransformation {
                let array = fast_matrix_mul(self.mat, rhs.mat);
                GenericTransformation{
                    mat: array,
                    inverse: inverse_4x4(array),
                }
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
    pub inverse: [f32; 16]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scaling {
    pub mat: [f32; 16],
    pub inverse: [f32; 16]
}

impl_matrix_operations!(Scaling);

impl Scaling {
    pub fn new(diagonal : [f32;3]) -> Scaling {
        if diagonal.iter().any(|a| *a == 0.0) {
            panic!("Wrong inputs in Scaling Matrix definition\
            {:?}", diagonal);
        }
        let mut array = [0f32; 16];
        let mut inverse = [0f32; 16];
        for i in 0..3 {
            array[i*5] = diagonal[i];
            inverse[i*5] = 1.0/ array[i*5];
        }
        array[15] = 1.0;
        inverse[15] = 1.0;

        Scaling{
            mat : array,
            inverse
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
        let mut inverse_mat = [0.0; 16];
        for i in 0..4{
            inverse_mat[i*5] = 1.0 / mat[i*5] ;
        }
        assert_eq!(mat, scale.mat);
        assert_eq!(inverse_mat, scale.inverse);
        let _ = Scaling::new([0.0,2.0,3.0]);
    }

    #[test]
    #[should_panic(expected = "Invalid quotient!")]
    fn test_scaling_scalar_operations(){
        let scale = Scaling::new([1.0, 2.0, 3.0]);
        let scale2 = scale * 5.0;
        let scale3 = -2.0  * scale;
        let scale4 = scale /2.0;
        for i in 0..16 {
            assert_eq!(scale2.mat[i], scale.mat[i] * 5.0);
            assert_eq!(scale3.mat[i], -scale.mat[i] * 2.0);
            assert_eq!(scale4.mat[i], scale.mat[i]/ 2.0);
        }
        let _ = scale / 0.0;
    }

    #[test]
    #[should_panic(expected = "Invalid quotient!")]
    fn test_scaling_division_nan(){
        let scale = Scaling::new([1.0, 2.0, 3.0]);
        let _ = scale / f32::NAN;
    }
    
    #[test]
    fn test_matrix_matrix(){
        panic!("WRITE THE TEST!");
    }

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