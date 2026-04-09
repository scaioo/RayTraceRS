//! Utility functions used across the raytracer.
//!
//! This module provides small, reusable helpers that are not tied to a
//! specific subsystem but are used throughout the codebase.

use endianness::ByteOrder;

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
    let epsilon = 1e-5;
    (x - y).abs() < epsilon
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
