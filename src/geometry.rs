//! Geometry modules founding spacial description of the image

use std::fmt;
//use std::fmt::Display;
use crate::functions::are_close;
use std::ops::{Add, Div, Mul, Neg, Sub};

// =======================================================================
// VEC2D
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub fn new(x: f32, y: f32) -> Vec2D {
        Vec2D { x, y }
    }

    pub fn is_close(&self, other: &Vec2D) -> bool {
        are_close(self.x, other.x) && are_close(self.y, other.y)
    }
}

// =======================================================================
// CONSTANTS DEFINITIONS
// =======================================================================
pub static X_AXIS: Vector = Vector {
    x: 1.0,
    y: 0.0,
    z: 0.0,
};
pub static Y_AXIS: Vector = Vector {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
pub static Z_AXIS: Vector = Vector {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};

// =======================================================================
// FUNCTIONS DEFINITIONS
// =======================================================================

// We could make it a trait through all the project!
pub fn is_close<T: TDV>(a: T, b: T) -> bool {
    are_close(a.x(), b.x()) && are_close(a.y(), b.y()) && are_close(a.z(), b.z())
}

// =======================================================================
// TRAIT DEFINITIONS
// =======================================================================

/// Marker trait that signals the struct to be either Vector, Point or Normal
pub trait TDV {
    fn to_homogeneous(&self) -> [f32; 4];

    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;
}

// =======================================================================
// MACRO DEFINITIONS
// =======================================================================

#[macro_export]
macro_rules! impl_homogeneous {
    ($t:ty, $w:expr) => {
        // Note: this trait implementation works only
        // on structs that have 'x', 'y', 'z' arguments!
        impl TDV for $t {
            fn to_homogeneous(&self) -> [f32; 4] {
                [self.x, self.y, self.z, $w]
            }

            fn x(&self) -> f32 {
                self.x
            }
            fn y(&self) -> f32 {
                self.y
            }
            fn z(&self) -> f32 {
                self.z
            }
        }
    };
}

/// Vector module stored as three floating-point components.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Point module stored as three floating-point components.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Normal module stored as three floating-point components.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Normal {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ==========================================
// Dot and Cross Trait
// ==========================================
pub trait Dot<Rhs> {
    fn dot(&self, rhs: &Rhs) -> f32;
}
pub trait Cross<Rhs> {
    type Output;
    fn cross(&self, rhs: &Rhs) -> Self::Output;
}

// ==========================================
// MACRO DEFINITION
// ==========================================
/// Macro to implement the `is_close` method for 3D structs.
macro_rules! impl_is_close {
    ($type_name:ident) => {
        impl $type_name {
            /// Returns `true` when on each axis the difference
            /// of the coordinate is less than the defined epsilon.
            /// Useful for comparing results of floating-point math.
            pub fn is_close(&self, other: &$type_name) -> bool {
                are_close(self.x, other.x)
                    && are_close(self.y, other.y)
                    && are_close(self.z, other.z)
            }
        }
    };
}

/// Macro to implement the `new` constructor for 3D structs
/// containing `x`, `y`, and `z` f32 components.
macro_rules! impl_new {
    ($type_name:ident) => {
        impl $type_name {
            /// Creates a new instance with the given coordinates.
            pub fn new(x: f32, y: f32, z: f32) -> Self {
                Self { x, y, z }
            }
        }
    };
}

/// Macro to implement the `Display` trait for any 3D struct.
/// It uses `stringify!` to automatically print the correct struct name.
macro_rules! impl_display {
    ($type_name:ident) => {
        impl fmt::Display for $type_name {
            /// Formats the struct for user-facing output.
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}(x = {}, y = {}, z = {})",
                    stringify!($type_name),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }
    };
}

/// Macro to implement the `Neg` (unary minus) trait for 3D structs.
macro_rules! impl_neg {
    ($type_name:ident) => {
        impl Neg for $type_name {
            type Output = Self;

            /// Returns a new instance with all components negated.
            fn neg(self) -> Self {
                Self::new(-self.x, -self.y, -self.z)
            }
        }
    };
}

/// Macro to implement scalar multiplication in both directions:
/// (Struct * f32) AND (f32 * Struct)
macro_rules! impl_mul_scalar {
    ($type_name:ident) => {
        // 1. Implementation for: Struct * f32
        impl Mul<f32> for $type_name {
            type Output = Self;

            fn mul(self, scalar: f32) -> Self {
                Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
            }
        }

        // 2. Implementation for: f32 * Struct
        impl Mul<$type_name> for f32 {
            type Output = $type_name;

            fn mul(self, other: $type_name) -> Self::Output {
                $type_name::new(self * other.x, self * other.y, self * other.z)
            }
        }
    };
}

/// Macro to implement scalar division (Div<f32>) for 3D structs.
macro_rules! impl_div_scalar {
    ($type_name:ident) => {
        impl Div<f32> for $type_name {
            type Output = Self;

            /// Divides each component of the struct by a scalar value.
            fn div(self, scalar: f32) -> Self {
                Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
            }
        }
    };
}

/// Macro to implement the `Add` trait between two potentially different 3D structs.
/// It allows specifying the exact return type of the operation.
macro_rules! impl_add {
    // The macro takes three identifiers: Left side, Right side, and Output type
    ($type_self:ident, $type_other:ident, $type_out:ident) => {
        // Implement the Add trait for the left-hand side type
        impl Add<$type_other> for $type_self {
            // Define the output type dynamically based on the macro parameter
            type Output = $type_out;

            /// Performs the addition component by component.
            /// Note: 'self' and 'other' are passed by value, which is safe and efficient
            /// because our structs implement the Copy trait.
            fn add(self, other: $type_other) -> Self::Output {
                $type_out::new(self.x + other.x, self.y + other.y, self.z + other.z)
            }
        }
    };
}

/// Macro to implement the `Sub` trait between two 3D structs.
/// It allows specifying the exact return type of the operation.
macro_rules! impl_sub {
    // The macro takes three identifiers: Left side, Right side, and Output type
    ($type_self:ident, $type_other:ident, $type_out:ident) => {
        impl Sub<$type_other> for $type_self {
            type Output = $type_out;

            /// Performs the subtraction component by component.
            /// Standard behavior: (self.x - other.x, ...)
            fn sub(self, other: $type_other) -> Self::Output {
                $type_out::new(self.x - other.x, self.y - other.y, self.z - other.z)
            }
        }
    };
}

/// Macro to implement the `norm` and `squared_norm` method for 3D structs.
macro_rules! impl_norm {
    ($type_name:ident) => {
        impl $type_name {
            /// Calculates the squared length (magnitude) of the struct.
            pub fn squared_norm(&self) -> f32 {
                self.x * self.x + self.y * self.y + self.z * self.z
            }

            /// Calculates the actual length (magnitude) of the struct.
            /// Note: This uses a square root, which is relatively slow.
            pub fn norm(&self) -> f32 {
                self.squared_norm().sqrt()
            }
        }
    };
}

/// Macro to implement the `Dot Product` method for 3D structs.
macro_rules! impl_dot {
    ($type_self: ident, $type_other: ident) => {
        //dot product return a float (f32)
        impl Dot<$type_other> for $type_self {
            fn dot(&self, second_term: &$type_other) -> f32 {
                self.x * second_term.x + self.y * second_term.y + self.z * second_term.z
            }
        }
    };
}
/// Macro to implement the `Cross Product` method for 3D structs.
macro_rules! impl_cross {
    ($type_self: ident) => {
        impl Cross<$type_self> for $type_self {
            type Output = $type_self;
            // Cross product returns a custom type ($type_other)
            fn cross(&self, other: &$type_self) -> $type_self {
                $type_self {
                    x: self.y * other.z - self.z * other.y,
                    y: self.z * other.x - self.x * other.z,
                    z: self.x * other.y - self.y * other.x,
                }
            }
        }
    };
}

/// Macro to implement the `normalize` method for directional 3D structs.
macro_rules! impl_normalize {
    ($type_name:ident) => {
        impl $type_name {
            /// Returns a new instance with the same direction but a length (norm) of exactly 1.0.
            ///
            /// Warning: If the original norm is 0.0, this will result in a division by zero,
            /// generating NaN (Not a Number) values. In a production raytracer, you might
            /// want to add a check for zero-length vectors if that's a risk.
            pub fn normalize(&self) -> Self {
                let length = self.norm();
                *self / length
            }
        }
    };
}

/// Macro to implement the `From` trait to convert one 3D struct into another.
/// Implementing `From` automatically gives us the `.into()` method for free!
macro_rules! impl_from {
    // The macro takes two identifiers: the Source type and the Destination type
    ($from_type:ident, $to_type:ident) => {
        impl From<$from_type> for $to_type {
            /// Converts the source struct into the destination struct
            /// by copying its x, y, and z components.
            fn from(item: $from_type) -> Self {
                Self::new(item.x, item.y, item.z)
            }
        }
    };
}

// ==========================================
// MACRO USAGE
// ==========================================

// ----------------------------------------------------------------
// Automatically implement `TDV` for Vector, Point, and Normal
// ----------------------------------------------------------------
impl_homogeneous!(Vector, 0.0);
impl_homogeneous!(Point, 1.0);
impl_homogeneous!(Normal, 0.0);

// ----------------------------------------------------------------
// Automatically implement `is_close` for Vector, Point, and Normal
// ----------------------------------------------------------------
impl_is_close!(Vector);
impl_is_close!(Point);
impl_is_close!(Normal);

// -------------------------------------------------------------
// Automatically generate the `new` method for all three structs
// -------------------------------------------------------------
impl_new!(Vector);
impl_new!(Point);
impl_new!(Normal);

// -----------------------------------------------
// Automatically implement Display for all structs
// -----------------------------------------------
impl_display!(Vector);
impl_display!(Point);
impl_display!(Normal);

// ---------------------------------------------------------
// Automatically implement Neg for Vector, Point, and Normal
// ---------------------------------------------------------
impl_neg!(Vector);
impl_neg!(Point);
impl_neg!(Normal);

// -------------------------------------------------------------------------
// Automatically implement Mul<f32> (Multiplication) and Div<f32> (Division)
// -------------------------------------------------------------------------
impl_mul_scalar!(Vector);
impl_mul_scalar!(Normal);
impl_div_scalar!(Vector);
impl_div_scalar!(Normal);

// -----------------------------------------
// Implement Add (Sum) and Sub (Subtraction)
//------------------------------------------

// 1. Vector + Vector = Vector
impl_add!(Vector, Vector, Vector);
// 2. Point + Vector = Point (Moving a point by a vector)
impl_add!(Point, Vector, Point);
// 3. Vector + Point = Point (Commutative property for the above)
impl_add!(Vector, Point, Point);

// 1. Vector - Vector = Vector
impl_sub!(Vector, Vector, Vector);
// 2. Point - Point = Vector (Crucial for finding directions between points)
impl_sub!(Point, Point, Vector);
// 3. Point - Vector = Point (Moving a point backwards)
impl_sub!(Point, Vector, Point);

//--------------------------------------------------------------------
// Automatically implement squared_norm and norm for Vector and Normal
//--------------------------------------------------------------------
impl_norm!(Vector);
impl_norm!(Normal);

//----------------------------------------------------------------------
// Automatically implement Dot and Cross products between the 3D Structs
//----------------------------------------------------------------------
impl_dot!(Vector, Vector);
impl_cross!(Vector);
impl_dot!(Vector, Normal);
impl_dot!(Normal, Vector);

//-----------------------------------------------------
// Automatically implement normalization for 3D Structs
//-----------------------------------------------------
impl_normalize!(Vector);
impl_normalize!(Normal);

//--------------------------------------------------
// Automatically implement Conversion for 3D Structs
//--------------------------------------------------

impl_from!(Normal, Vector);
impl_from!(Vector, Normal);
impl_from!(Point, Vector);
impl_from!(Vector, Point);

// ==========================================
// TEST
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2d_is_close() {
        let v1 = Vec2D::new(1.0, 2.0);
        let v2 = Vec2D::new(1.000001, 2.0);
        assert!(v1.is_close(&v2));
        assert!(!v1.is_close(&Vec2D::new(1.00001, 2.0)));
    }

    #[test]
    fn test_is_close_method() {
        // Create a base vector
        let v1 = Vector::new(1.0, 2.0, 3.0);

        // Create a vector with a microscopic difference (within epsilon 1e-5)
        let v2 = Vector::new(1.000001, 2.0, 3.0);

        // Create a vector with a larger difference (outside epsilon)
        let v3 = Vector::new(1.0001, 2.0, 3.0);

        // This should be true because the difference is tiny
        assert!(v1.is_close(&v2), "v1 and v2 should be considered close");

        // We use ! (NOT) because this should be false
        assert!(
            !v1.is_close(&v3),
            "v1 and v3 should NOT be considered close"
        );

        // Test with a Normal just to be safe
        let n1 = Normal::new(0.0, 1.0, 0.0);
        let n2 = Normal::new(0.0, 0.999999, 0.0);
        assert!(n1.is_close(&n2), "n1 and n2 should be considered close");
    }
    #[test]
    fn test_constructors() {
        // Now you can instantiate them elegantly
        let v = Vector::new(1.0, 2.0, 3.0);
        let p = Point::new(4.0, 5.0, 6.0);
        let n = Normal::new(7.0, 8.0, 9.0);

        assert_eq!(v, Vector::new(1.0, 2.0, 3.0));
        assert_eq!(p, Point::new(4.0, 5.0, 6.0));
        assert_eq!(n, Normal::new(7.0, 8.0, 9.0));
    }
    #[test]
    fn test_display_trait() {
        // Instantiate the structs
        let v = Vector::new(1.0, 2.5, 3.0);
        let n = Normal::new(0.0, -1.0, 4.2);
        let p = Point::new(4.0, 5.0, 6.0);
        // Using format! allows us to capture the Display output as a String
        let formatted_vector = format!("{}", v);
        let formatted_normal = format!("{}", n);
        let formatted_point = format!("{}", p);
        // Check if the output matches the expected string
        assert_eq!(formatted_vector, "Vector(x = 1, y = 2.5, z = 3)");
        assert_eq!(formatted_normal, "Normal(x = 0, y = -1, z = 4.2)");
        assert_eq!(formatted_point, "Point(x = 4, y = 5, z = 6)");
    }
    #[test]
    fn test_negation() {
        // Create an initial normal
        let n = Normal::new(1.0, -2.5, 3.0);
        let v = Vector::new(-5.0, 0.0, 4.2);
        let p = Point::new(2.2, 1.9, 0.8);

        assert_eq!(-n, Normal::new(-1.0, 2.5, -3.0));
        assert_eq!(-v, Vector::new(5.0, 0.0, -4.2));
        assert_eq!(-p, Point::new(-2.2, -1.9, -0.8));
    }
    #[test]
    fn test_scalar_math() {
        let v = Vector::new(1.0, -2.0, 3.0);

        // Test scalar multiplication
        let multiplied = v * 2.0;
        assert_eq!(multiplied, Vector::new(2.0, -4.0, 6.0));

        // Test scalar division
        let divided = multiplied / 2.0;
        assert_eq!(divided, Vector::new(1.0, -2.0, 3.0));

        // Test with a Normal to ensure the macro works globally
        let n = Normal::new(10.0, 20.0, 30.0);
        assert_eq!(n * 0.5, Normal::new(5.0, 10.0, 15.0));
    }
    #[test]
    fn test_custom_addition() {
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(4.0, 5.0, 6.0);
        let p1 = Point::new(7.0, 8.0, 9.0);

        // Vector + Vector -> Vector
        let v3 = v1 + v2;
        assert_eq!(v3, Vector::new(5.0, 7.0, 9.0));

        // Point + Vector -> Point
        // Notice how the result is automatically a Point type!
        let p2 = p1 + v1;
        assert_eq!(p2, Point::new(8.0, 10.0, 12.0));

        // Vector + Point -> Point
        let p3 = v1 + p1;
        assert_eq!(p3, Point::new(8.0, 10.0, 12.0));
    }
    #[test]
    fn test_norms() {
        // 1. Test with a Vector using a Pythagorean triple (3, 4, 5)
        // 3^2 + 4^2 + 0^2 = 9 + 16 = 25
        // Sqrt(25) = 5
        let v = Vector::new(3.0, 4.0, 0.0);

        // We use assert! with the are_close function
        assert!(
            are_close(v.squared_norm(), 25.0),
            "Vector squared norm is incorrect!"
        );
        assert!(are_close(v.norm(), 5.0), "Vector norm is incorrect!");

        // 2. Test with a Normal using negative and decimal values
        // (-1)^2 + 1^2 + 1^2 = 1 + 1 + 1 = 3
        // Sqrt(3) ≈ 1.73205
        let n = Normal::new(-1.0, 1.0, 1.0);
        assert!(
            are_close(n.squared_norm(), 3.0),
            "Normal squared norm is incorrect!"
        );

        let expected_n_norm = 3.0_f32.sqrt();
        assert!(
            are_close(n.norm(), expected_n_norm),
            "Normal norm is incorrect!"
        );
    }
    #[test]
    fn test_dot_vector_vector() {
        // Create two vectors
        let v1 = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(4.0, 5.0, 6.0);

        // Calculate dot product: (1*4) + (2*5) + (3*6) = 4 + 10 + 18 = 32
        let result = v1.dot(&v2);

        // Verify the result
        assert_eq!(result, 32.0, "Dot product between Vector and Vector failed");
    }
    #[test]
    fn test_cross_vec_vec() {
        let v = Vector::new(1.0, 2.0, 3.0);
        let u = Vector::new(2.0, 3.0, 4.0);

        let ax = 2.0 * 4.0 - 3.0 * 3.0;
        let ay = 3.0 * 2.0 - 1.0 * 4.0;
        let az = 1.0 * 3.0 - 2.0 * 2.0;

        assert_eq!(
            v.cross(&u),
            Vector::new(ax, ay, az),
            "Cross product between Vectors failed"
        );
    }
    #[test]
    fn test_dot_normal_vec() {
        let v = Vector::new(1.0, 2.0, 3.0);
        let u = Normal::new(2.0, 3.0, 4.0);

        assert_eq!(v.dot(&u), 20.0);
        assert_eq!(u.dot(&v), 20.0);
    }

    #[test]
    fn test_normalization() {
        // Create a vector with length = 5 (3^2 + 4^2 = 25 -> sqrt(25) = 5)
        let v = Vector::new(3.0, 0.0, 4.0);

        // Normalize it
        let v_normalized = v.normalize();

        // 1. The new length MUST be exactly 1.0
        assert!(
            are_close(v_normalized.norm(), 1.0),
            "Normalized vector length is not 1.0!"
        );

        // 2. The components should be 3/5, 0/5, and 4/5
        assert!(are_close(v_normalized.x, 0.6), "X component incorrect");
        assert!(are_close(v_normalized.y, 0.0), "Y component incorrect");
        assert!(are_close(v_normalized.z, 0.8), "Z component incorrect");

        // 3. Test with a Normal as well
        let n = Normal::new(1.0, 1.0, 1.0);
        let n_norm = n.normalize();

        assert!(
            are_close(n_norm.norm(), 1.0),
            "Normalized normal length is not 1.0!"
        );
    }
}
