//! Geometry modules founding spacial description of the image

use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::functions::are_close;

/// Vector module stored as three floating-point components.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector{
    pub x : f32,
    pub y : f32,
    pub z : f32,
}

impl Vector{
    /// Creates a new `Vector` with the given components.
    ///
    /// This function performs no validation. Values such as `NaN`,
    /// `INFINITY`, and `NEG_INFINITY` are allowed.
    ///
    /// # Examples
    /// ```rust
    /// use rstrace::geometry::Vector;
    ///
    /// let v = Vector::new(1.0, 0.0, -5.0);
    /// assert_eq!(v.x, 1.0);
    /// assert_eq!(v.y, 0.0);
    /// assert_eq!(v.z, -5.0);
    /// ```
    pub fn new(x : f32, y : f32, z : f32) -> Vector {
        Vector{x, y, z}
    }
}

/// Formats the vector as `Vec(x = ..., y = ..., z = ...)`.
impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec(x = {}, y = {}, z = {})", self.x, self.y, self.z)
    }
}

/// Component-wise addition of two vectors
impl Add for Vector {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}

// Component-wise subtraction of two vectors
impl Sub for Vector {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Self;
    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other
        }
    }
}


impl Mul<Vector> for f32 {
    type Output = Vector;
    fn mul(self, other: Vector) -> Vector {
        Vector{
            x: self * other.x,
            y: self * other.y,
            z: self * other.z
        }
    }
}

/// Fraction of a Vector by a scalar
///
/// No control is present avoiding /0.0 operation
impl Div<f32> for Vector {
    type Output = Self;
    fn div(self, other: f32) -> Self {
        Vector {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other
        }
    }
}


impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self {
        Vector{
            x : - self.x,
            y : - self.y,
            z : - self.z
        }
    }
}


impl Vector {
    /// Dot product of vectors
    ///
    /// # Example
    /// ```rust, no_run
    /// use rstrace::geometry::Vector;
    /// let vector_1 = Vector::new(1.0, 2.0, 3.0);
    /// let vector_2 = Vector::new(-1.0, -2.0, 3.0);
    /// assert_eq!(vector_1.dot(vector_2), 4.0);
    ///```
    // Consider adding the habits for NaN and INFINITY.
    pub fn dot(self, other: Self) -> f32 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    /// Cross product of vectors
    ///
    /// # Example
    /// ```rust, no_run
    /// use rstrace::geometry::Vector;
    /// let vector_1 = Vector::new(1.0, 2.0, 3.0);
    /// let vector_2 = Vector::new(-1.0, -2.0, 3.0);
    /// let expected_v = Vector::new(12.0, -6.0, 0.0);
    /// assert_eq!(vector_1.cross(vector_2), expected_v);
    ///```
    // Consider adding the habits for NaN and INFINITY.
    pub fn cross(self, other: Self) -> Self {
        Vector{
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    /// Return the norm (Euclidean length) of a vector
    pub fn norm(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Return the squared norm (Euclidean length) of a vector
    pub fn squared_norm(self) -> f32 {
        self.dot(self)
    }

    /// Normalization of a Vector
    ///
    /// Return a vector with its norm equal to 1
    pub fn normalize(self) -> Self {
        if self.squared_norm() == 0.0 { // Is this control too computationally heavy?
            panic!("ERROR: normalize() is impossibile for Vec(0,0,0)!");
        }
        self / self.norm()
    }

    /// Returns `true` when on each ax the difference
    /// of the coordinate is less than 0.00001.
    pub fn is_closed(&self, other: &Vector) -> bool{
        are_close(self.x, other.x)
            && are_close(self.y, other.y)
            && are_close(self.z, other.z)
    }
}

/* TODO: [function][test]
- [X][X] Constructor
- [X][X] Conversion to String
- [ ][ ] Comparison between vectors
- [X][X] Sum between vectors
- [X][X] difference between vectors
- [X][X] Product by a scalar
- [X][X] Negation
- [X][x] Dot product between two vectors and cross product
- [X][X] Calculation of squared_norm and norm
- [X][X] Function that normalizes the vector
- [ ][ ] Function that converts a Vec into a Normal
- [ ][ ] ...
*/

// ===========================================================================
// ===========================================================================
/// Point module stored as three floating-point components.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point{
    pub x : f32,
    pub y : f32,
    pub z : f32,
}

/* TODO: [function][test]
- [ ][ ] Constructor
- [ ][ ] Conversion to String
- [ ][ ] Sum Point + Vector -> Vector
- [ ][ ] Difference between two Points, returning a Vec;
- [ ][ ] Difference between Point and Vec, returning a Point
- [ ][ ] Conversion from Point to Vec (Point.to_vec())
- [ ][ ] Altro
 */

// ===========================================================================
// ===========================================================================

/// Normal module stored as three floating-point components.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Normal {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/* TODO
- [ ][ ] Constructor
- [ ][ ] Comparison between normals (for tests)
- [ ][ ] Operatore -normale
- [ ][ ] Multiplication by a scalar
- [ ][ ] Dot product Vec·Normal and cross product Vec×Normal and Normal×Normal
- [ ][ ] Calculation of squared_norm and norm
- [ ][ ] Function that normalizes the normal
- [ ][ ] Altro
 */
// -------------------------------------------------------------
//                            Tests
// -------------------------------------------------------------


#[cfg(test)]
mod test {
    use super::*;

    //======================= Vector ==========================
    #[test]
    fn test_vector_constructor(){
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vector_display(){
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(format!("{}", v), "Vec(x = 1, y = 2, z = 3)");
        let v = Vector::new(1.0, 2.201, -3.0);
        assert_eq!(format!("{}", v), "Vec(x = 1, y = 2.201, z = -3)");
    }

    #[test]
    fn test_vector_addition(){
        let v1 =
            Vector::new(1.0, 2.0, 3.0)
                + Vector::new(20.0, 300.0, -4.0);
        let v2 = Vector::new(21.0, 302.0, -1.0);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_vector_subtraction(){
        let v1 =
            Vector::new(1.0, 2.0, 3.0)
                - Vector::new(20.0, 300.0, -4.0);
        let v2 = Vector::new(-19.0, -298.0, 7.0);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_vector_multiplication(){
        let v = Vector::new(1.0, 2.0, 3.0);
        // Test scalar * vector
        let v1 = 0.5 * v;
        assert_eq!(v1, Vector::new(0.5, 1.0, 1.5));

        // Test vector * scalar
        let v1 = v * 5.0;
        assert_eq!(v1, Vector::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_vector_division(){
        let v1 = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(v1/2.0, Vector::new(0.5, 1.0, 1.5));
    }

    #[test]
    fn test_vector_dot_product(){
        let vector_1 = Vector::new(1.0, 2.0, 3.0);
        let vector_2 = Vector::new(-1.0, -2.0, 3.0);
        assert_eq!(vector_1.dot(vector_2), 4.0);
    }

    #[test]
    fn test_vector_cross_product(){
        let vector_1 = Vector::new(1.0, 0.0, 0.0);
        let vector_2 = Vector::new(0.0, 1.0, 0.0);
        assert_eq!(vector_1.cross(vector_2), Vector::new(0.0, 0.0, 1.0));
        let vector_1 = Vector::new(1.0, 1.0, 0.0);
        let vector_2 = Vector::new(10.0, 10.0, 0.0);
        assert_eq!(vector_1.cross(vector_2), Vector::new(0.0,0.0,0.0));
        let vector_1 = Vector::new(1.0, 2.0, 3.0);
        let vector_2 = Vector::new(-1.0, -2.0, 3.0);
        let vector_expected = Vector::new(12.0, -6.0, 0.0);
        assert_eq!(vector_1.cross(vector_2), vector_expected);
        assert_eq!(vector_2.cross(vector_1), -1.0 * vector_expected);
    }

    #[test]
    fn test_negation(){
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(-v, Vector::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_norm(){
        let v = Vector::new(4.0, 0.0, -3.0);
        assert_eq!(v.norm(), 5.0);
    }

    #[test]
    fn test_squared_norm(){
        let v = Vector::new(4.0, 0.0, -3.0);
        assert_eq!(v.squared_norm(), 25.0);
    }

    #[test]
    #[should_panic(expected = "ERROR: normalize() is impossibile for Vec(0,0,0)!")]
    fn test_normalize(){
        let v = Vector::new(3.0, -4.0, 0.0);
        assert_eq!(v.normalize(), Vector::new(
            3.0/5.0, 
            -4.0/5.0, 
            0.0/5.0)
        );
        
        let v = Vector::new(0.0, 0.0, 0.0);
        let _ = v.normalize();
    }

    #[test]
    fn test_is_closed(){
        let v = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(1.0000001, 2.000000003, 3.0000000005);
        assert_eq!(v.is_closed(&v2), true);
        let v2 = Vector::new(10.0, 10.0, 0.0);
        assert_eq!(v.is_closed(&v2), false);
        let v2 = Vector::new(1.00001, 2.000000003, 3.0000000005);
        assert_eq!(v.is_closed(&v2), false); // x1 - x2 > 0.00001!!!
    }
}