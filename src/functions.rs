//! Utility functions used across the raytracer.
//!
//! This module provides small, reusable helpers that are not tied to a
//! specific subsystem but are used throughout the codebase.

use endianness::ByteOrder;
//use crate::geometry::TDV;

pub static IDENTITY_4X4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// Returns `true` if two floating-point numbers are approximately equal.
///
/// The comparison uses a fixed absolute tolerance:
/// `|x - y| < 1e-5`.
/// This method is best suited for values with small magnitude, as it does
/// not perform a relative comparison.
///
/// # Examples
/// ```rust
/// use rstrace::functions::are_close;
///
/// assert!(are_close(0.1 + 0.2, 0.3));
/// assert!(!are_close(1.0, 2.0));
/// ```
pub fn are_close(x: f32, y: f32) -> bool {
    match (x.is_finite(), y.is_finite()) {
        (true, true) => (x - y).abs() < 1e-5,
        _ => x == y || (x.is_nan() && y.is_nan()),
    }
}

/// Converts an endianness value into a numeric representation.
///
/// Returns:
/// - `-1.0` for [`ByteOrder::LittleEndian`]
/// - `+1.0` for [`ByteOrder::BigEndian`]
///
/// # Examples
/// ```rust
/// use rstrace::functions::endianness_number;
/// use endianness::ByteOrder;
/// assert_eq!(-1.0, endianness_number(&ByteOrder::LittleEndian));
/// assert_eq!(1.0, endianness_number(&ByteOrder::BigEndian));
/// ```
pub fn endianness_number(endianness: &ByteOrder) -> f32 {
    match endianness {
        ByteOrder::LittleEndian => -1.0,
        ByteOrder::BigEndian => 1.0,
    }
}

/// Computationally fast multiplication between matrices written as [0.0f32;16] arrays
pub fn fast_matrix_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0; 16];

    // First row
    result[0] = a[0] * b[0] + a[1] * b[4] + a[2] * b[8] + a[3] * b[12];
    result[1] = a[0] * b[1] + a[1] * b[5] + a[2] * b[9] + a[3] * b[13];
    result[2] = a[0] * b[2] + a[1] * b[6] + a[2] * b[10] + a[3] * b[14];
    result[3] = a[0] * b[3] + a[1] * b[7] + a[2] * b[11] + a[3] * b[15];

    // Second row
    result[4] = a[4] * b[0] + a[5] * b[4] + a[6] * b[8] + a[7] * b[12];
    result[5] = a[4] * b[1] + a[5] * b[5] + a[6] * b[9] + a[7] * b[13];
    result[6] = a[4] * b[2] + a[5] * b[6] + a[6] * b[10] + a[7] * b[14];
    result[7] = a[4] * b[3] + a[5] * b[7] + a[6] * b[11] + a[7] * b[15];

    // Third row
    result[8] = a[8] * b[0] + a[9] * b[4] + a[10] * b[8] + a[11] * b[12];
    result[9] = a[8] * b[1] + a[9] * b[5] + a[10] * b[9] + a[11] * b[13];
    result[10] = a[8] * b[2] + a[9] * b[6] + a[10] * b[10] + a[11] * b[14];
    result[11] = a[8] * b[3] + a[9] * b[7] + a[10] * b[11] + a[11] * b[15];

    // Fourth row
    result[12] = a[12] * b[0] + a[13] * b[4] + a[14] * b[8] + a[15] * b[12];
    result[13] = a[12] * b[1] + a[13] * b[5] + a[14] * b[9] + a[15] * b[13];
    result[14] = a[12] * b[2] + a[13] * b[6] + a[14] * b[10] + a[15] * b[14];
    result[15] = a[12] * b[3] + a[13] * b[7] + a[14] * b[11] + a[15] * b[15];

    // Returns
    result
}

/// Hopefully fast computing of the inverse of a 4x4 matrix or returns Err
pub fn inverse_4x4(m: &[f32; 16]) -> [f32; 16] {
    let mut inv = [0.0; 16];

    inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];

    inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];

    inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];

    inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];

    inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];

    inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];

    inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];

    inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];

    inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];

    inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];

    inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];

    inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];

    inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];

    inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];

    inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];

    inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];

    let det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];

    if det.abs() < 1e-6 {
        panic!("det is 0.0!");
    }

    let inv_det = 1.0 / det;
    for item in &mut inv {
        *item *= inv_det;
    }

    inv
}


/// TODO
pub fn transpose_matrix(m: &[f32; 16]) -> [f32; 16] {
    let mut result: [f32; 16] = [0.0; 16];
    for i in 0..4 {
        for j in 0..4 {
            result[i * 4 + j] = m[j * 4 + i];
        }
    }
    result
}

/// TODO
pub fn equal_matrices(mat1: &[f32; 16], mat2: &[f32; 16]) -> bool {
    let mut result = true;
    for i in 0..16 {
        result = result && are_close(mat1[i], mat2[i]);
    }
    result
}

// tests
#[cfg(test)]
mod tests {
    use image::ExtendedColorType::A8;
    //use crate::geometry::{is_close, Vector};
    use super::*;

    static MAT: [f32; 16] = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, -9.0, -10.0, -11.0, -12.0, -13.0, -14.0, 15.0,
        -16.0,
    ];
    #[test]
    fn are_close_test() {
        let x = 0.11111;
        let y = 0.11112;

        if !are_close(x, y) {
            panic!("are_close is not working");
        }

        let x = f32::NAN;
        let y = f32::NAN;
        assert!(are_close(x, y));

        let x = f32::INFINITY;
        assert!(!are_close(x, y));

        let y = f32::NEG_INFINITY;
        assert!(!are_close(x, y));

        let y = f32::INFINITY;
        assert!(are_close(x, y));
    }

    #[test]
    fn test_endianness_number() {
        assert_eq!(-1.0, endianness_number(&ByteOrder::LittleEndian));
        assert_eq!(1.0, endianness_number(&ByteOrder::BigEndian));
    }

    #[test]
    fn test_matrix_product() {
        let mat1: [f32; 16] = [
            2.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let mat1_inverse: [f32; 16] = [
            0.5, 0.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let result = fast_matrix_mul(&mat1, &mat1_inverse);
        for i in 0..16 {
            assert_eq!(result[i], IDENTITY_4X4[i]);
        }
        let mat2: [f32; 16] = [
            1.0, 0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let mat2_inverse: [f32; 16] = [
            1.0, 0.0, 0.0, -10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let result1 = fast_matrix_mul(&mat2, &mat1_inverse);
        let result2 = fast_matrix_mul(&mat1, &mat2_inverse);
        let result = fast_matrix_mul(&result1, &result2);
        for i in 0..16 {
            assert_eq!(result[i], IDENTITY_4X4[i]);
        }
    }

    #[test]
    fn test_matrix_inverse() {
        let mat1: [f32; 16] = [
            2.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let mat1_inverse: [f32; 16] = [
            0.5, 0.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let result = inverse_4x4(&mat1);
        for i in 0..16 {
            assert!(are_close(result[i], mat1_inverse[i]));
        }
        let mat1: [f32; 16] = [
            1.0, 0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let mat1_inverse: [f32; 16] = [
            1.0, 0.0, 0.0, -10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let result = inverse_4x4(&mat1);
        for i in 0..16 {
            assert!(are_close(result[i], mat1_inverse[i]));
        }
    }

    #[test]
    fn test_transpose_matrix() {
        let mat = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 3.0, 0.0,
        ];
        let result = transpose_matrix(&mat);
        let expected = [
            1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 3.0, 1.0, 3.0, 0.0, 0.0, 0.0, 0.0,
        ];
        for i in 0..16 {
            assert!(are_close(result[i], expected[i]));
        }
    }

    #[test]
    fn test_transpose_matrices() {
        let mat = transpose_matrix(&MAT);
        let expected = [
            1.0, 5.0, -9.0, -13.0, 2.0, 6.0, -10.0, -14.0, 3.0, 7.0, -11.0, 15.0, 4.0, 8.0, -12.0,
            -16.0,
        ];
        assert!(equal_matrices(&mat, &expected), "{:?}\n{:?}", mat, expected);
    }

    #[test]
    fn test_equal_matrices() {
        let mat1: [f32; 16] = [
            1.0, 0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let mat1_inverse: [f32; 16] = [
            1.0, 0.0, 0.0, -10.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let result = fast_matrix_mul(&mat1, &mat1_inverse);
        assert!(equal_matrices(&result, &IDENTITY_4X4));
    }
}
