use crate::geometry::{Vector, Point, Normal};
// =======================================================================
// TRAIT DEFINITIONS
// =======================================================================



// =======================================================================
// MACRO DEFINITIONS
// =======================================================================


// =======================================================================
// STRUCT DEFINITIONS
// =======================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GenericTranfsormation {
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