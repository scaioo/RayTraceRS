//! Utility helper functions for the raytracer.
//!
//! This module provides small, reusable functions.
use endianness::ByteOrder;

/// Returns `true` if two floating-point numbers are close enough to be considered equal.
///
/// The comparison is performed using a fixed absolute tolerance:
/// `|x - y| < 1e-5`.
///
/// # Purpose
/// This function is used to handle **floating-point precision errors**.
/// Because computers store decimal numbers in binary, some values
/// (like 0.1) cannot be represented exactly.
/// This can lead to small rounding errors (e.g., 0.1 + 0.2 resulting in 0.30000000000000004).
/// The function ensures these tiny differences don’t affect program logic,
/// typically by rounding values or comparing them within a small tolerance.
///
/// # Notes
/// - This is a simple absolute comparison, not relative.
/// - It is suitable for small-magnitude values, but may be inaccurate
///   for very large numbers.
///
/// # Examples
/// ```rust
/// use rstrace::functions::are_close;
/// assert!(are_close(0.1 + 0.2, 0.3));
/// assert!(!are_close(1.0, 2.0));
/// ```
pub fn are_close(x: f32, y: f32) -> bool {
    let epsilon = 1e-5;
    (x - y).abs() < epsilon
}

/// Maps endianness to positive or negative floating-point
///
/// Returns:
/// - `-1.0` for [`ByteOrder::BigEndian`]
/// - `+1.0` for [`ByteOrder::LittleEndian`]
///
/// # Purpose
/// This function provides a compact numeric representation of endianness,
/// consistent with the PFM file format.
/// # Note
/// The mapping is arbitrary but consistent across the codebase.
pub fn endianness_number(endianness: &ByteOrder) -> f32 {
    match endianness {
        ByteOrder::LittleEndian => -1.0,
        ByteOrder::BigEndian => 1.0,
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn are_close_test() {
        let x = 0.11111;
        let y = 0.11112;

        if !are_close(x, y) {
            panic!("are_close is not working");
        }
    }

    #[test]
    fn test_endianness_number() {
        assert_eq!(-1.0, endianness_number(&ByteOrder::LittleEndian));
        assert_eq!(1.0, endianness_number(&ByteOrder::BigEndian));
    }
}
