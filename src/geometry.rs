//! Geometry modules founding spacial description of the image

use std::fmt;
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        if self.squared_norm() < 1.0e-06 { // Is this control too computationally heavy?
            panic!("ERROR: normalize() is impossibile for Vec(0,0,0)!");
        }
        self / self.norm()
    }

    /// Returns `true` when on each ax the difference
    /// of the coordinate is less than 0.00001.
    pub fn is_close(&self, other: &Vector) -> bool{
        are_close(self.x, other.x)
            && are_close(self.y, other.y)
            && are_close(self.z, other.z)
    }

    pub fn to_normal(&self) -> Normal {
        Normal{
            x : self.x,
            y : self.y,
            z : self.z,
        }
    }
}

/* TODO: [function][test]
- [X][X] Constructor
- [X][X] Conversion to String
- [X][X] Comparison between vectors
- [X][X] Sum between vectors
- [X][X] difference between vectors
- [X][X] Product by a scalar
- [X][X] Negation
- [X][x] Dot product between two vectors and cross product
- [X][X] Calculation of squared_norm and norm
- [X][X] Function that normalizes the vector
- [X][ ] Function that converts a Vec into a Normal
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

impl Point {
    /// Creates a new `Point` with the given components.
    ///
    /// This function performs no validation. Values such as `NaN`,
    /// `INFINITY`, and `NEG_INFINITY` are allowed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rstrace::geometry::Point;
    ///
    /// let p = Point::new(1.0, 2.0, 3.0);
    /// assert_eq!(p.x, 1.0);
    /// assert_eq!(p.y, 2.0);
    /// assert_eq!(p.z, 3.0);
    /// ```
    pub fn new(x : f32, y : f32, z : f32) -> Point {
        Point{x, y, z}
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point(x = {}, y = {}, z = {})", self.x, self.y, self.z)
    }
}

// What should we write in the doc? TODO
// NOTE: maybe explicit the NAN, INFINITY and NEG_INFINITY handling
impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, other: Vector) -> Self {
        Point{
            x : self.x + other.x,
            y : self.y + other.y,
            z : self.z + other.z
        }
    }
}
impl Add<Point> for Vector {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point{
            x : self.x + other.x,
            y : self.y + other.y,
            z : self.z + other.z
        }
    }
}

/// Subtraction of two points, returning a vector.
///
/// From an intuitive physics view, the first point to appear in the subtraction si
/// the point reached, the first the starting point.
///
/// i.e.: in `b - a` the Vector has origin in `a` and cap in `b`.
impl Sub for Point {
    type Output = Vector;
    fn sub(self, other: Self) -> Vector {
        Vector{
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

impl Sub<Vector> for Point {
    type Output = Point;
    fn sub(self, other: Vector) -> Point {
        self + (-other)
    }
}

impl Point {
    pub fn to_vec(self) -> Vector {
        Vector{
            x : self.x,
            y : self.y,
            z : self.z
        }
    }
}

/* TODO: [function][test]
- [X][X] Constructor
- [X][X] Conversion to String
- [X][X] Sum Point + Vector -> Point
- [X][X] Difference between two Points, returning a Vec;
- [X][X] Difference between Point and Vec, returning a Point
- [X][X] Conversion from Point to Vec (Point.to_vec())
- [ ][ ] ...
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
impl Normal {
    /// Creates a new `Normal` with the given components.
    ///
    /// This function performs no validation. Values such as `NaN`,
    /// `INFINITY`, and `NEG_INFINITY` are allowed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rstrace::geometry::Normal;
    ///
    /// let normal = Normal::new(1.0, 2.0, 3.0);
    /// assert_eq!(normal.x, 1.0);
    /// assert_eq!(normal.y, 2.0);
    /// assert_eq!(normal.z, 3.0);
    /// ```
    pub fn new(x : f32, y : f32, z : f32) -> Normal {
        Normal{x, y, z}
    }
}

impl Display for Normal{
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Normal(x = {}, y = {}, z = {})",self.x,self.y,self.z)
    }
}
impl Normal{
    /// Returns `true` when on each ax the difference
    /// of the coordinate is less than 0.00001.
    pub fn is_close(&self, other: &Normal) -> bool {
        are_close(self.x, other.x)
            && are_close(self.y, other.y)
            && are_close(self.z, other.z)
    }
}

impl Neg for Normal {
    type Output = Self;
    fn neg(self) -> Self {
        Normal::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f32> for Normal {
    type Output = Self;
    fn mul(self, other: f32) -> Self {
        Normal{
            x : self.x * other,
            y : self.y * other,
            z : self.z * other
        }
    }
}

impl Mul<Normal> for f32 {
    type Output = Normal;
    fn mul(self, other: Normal) -> Normal {
        other * self
    }
}

/// Divides a `Normal` by a scalar.
///
/// No checks are performed for division by zero. If `other == 0.0`,
/// the result will contain `INFINITY` or `NaN` values according to
/// IEEE-754 floating-point rules.
///
/// # Example
/// ```rust
/// use rstrace::geometry::Normal;
///
/// let n = Normal::new(10.0, 1.0, 4.0);
/// assert_eq!(n / 2.0, Normal::new(5.0, 0.5, 2.0));
/// let n = n / 0.0;
/// assert!(n.x.is_infinite());
/// ```
impl Div<f32> for Normal {
    type Output = Self;
    fn div(self, other: f32) -> Self {
        self * (1.0 / other)
    }
}
// -------------------
// Dot and Cross Trait
// -------------------


pub trait Dot<Rhs> {
    fn dot(&self, rhs: &Rhs) -> f32;
}
pub trait Cross<Rhs> {
    type Output;
    fn cross(&self, rhs: &Rhs) -> Self::Output;
}

// ---------------------------------
// Macros for dot and cross products
// ---------------------------------
macro_rules! impl_dot {
    ($type_self: ident, $type_other: ident) => {
        //dot product return a float (f32)
        impl Dot<$type_other> for $type_self{
            fn dot(&self, second_term: &$type_other) -> f32 {
                self.x*second_term.x + self.y*second_term.y +self.z*second_term.z
            }
        }
    };
}
macro_rules! impl_cross {
    ($type_self: ident, $type_other: ident,$type_out: ident) =>{
        impl Cross<$type_other> for $type_self{
            type Output = $type_out;
            // Cross product returns a custom type ($type_other)
            fn cross(&self,other: &$type_other) -> $type_out {
                $type_out{
                    x: self.y * other.z - self.z * other.y,
                    y: self.z * other.x - self.x * other.z,
                    z: self.x * other.y - self.y * other.x,
                }
            }
        }
    };
}

// Implementation of dot and cross for Vector and Norm
impl_dot!(Vector,Vector);
impl_cross!(Vector,Vector,Vector);
impl_dot!(Normal,Vector);
impl_dot!(Vector,Normal);

// Calculation of squared norm and norm

/* TODO
- [X][X] Constructor
- [X][X] Conversion to String
- [X][X] Comparison between normals (for tests)
- [X][X] Operator -normale
- [X][X] Multiplication by a scalar
- [X][X] Dot product Vec·Normal and cross product Vec×Normal and Normal×Normal
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
        let v = Vector::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
        assert_eq!(v.y, f32::INFINITY);
        assert_eq!(v.z, f32::NEG_INFINITY);
        assert!(v.x.is_nan());
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
    fn test_vector_negation(){
        let v = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(-v, Vector::new(-1.0, -2.0, -3.0));
    }
    #[test]
    fn test_vector_norm(){
        let v = Vector::new(4.0, 0.0, -3.0);
        assert_eq!(v.norm(), 5.0);
    }
    #[test]
    fn test_vector_squared_norm(){
        let v = Vector::new(4.0, 0.0, -3.0);
        assert_eq!(v.squared_norm(), 25.0);
    }
    #[test]
    #[should_panic(expected = "ERROR: normalize() is impossibile for Vec(0,0,0)!")]
    fn test_vector_normalize(){
        let v = Vector::new(3.0, -4.0, 0.0);
        assert_eq!(v.normalize(), Vector::new(
            3.0/5.0,
            -4.0/5.0,
            0.0/5.0)
        );

        let v = Vector::new(0.0, 0.0, 0.000001);
        let _ = v.normalize();
    }
    #[test]
    fn test_vector_is_close(){
        let v = Vector::new(1.0, 2.0, 3.0);
        let v2 = Vector::new(1.0000001, 2.000000003, 3.0000000005);
        assert_eq!(v.is_close(&v2), true);
        let v2 = Vector::new(10.0, 10.0, 0.0);
        assert_eq!(v.is_close(&v2), false);
        let v2 = Vector::new(1.00001, 2.000000003, 3.0000000005);
        assert_eq!(v.is_close(&v2), false); // x1 - x2 > 0.00001!!!
    }
    #[test]
    fn test_to_normal(){
        let v = Vector::new(1.0, 2.0, f32::INFINITY);
        assert_eq!(v.to_normal(), Normal::new(1.0, 2.0, f32::INFINITY));
    }

    //======================= Point ==========================

    #[test]
    fn test_point_constructor(){
        let p = Point::new(1.0, 2.0, 3.0);
        assert_eq!(p, Point::new(1.0, 2.0, 3.0));
        let p = Point::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
        assert_eq!(p.y, f32::INFINITY);
        assert_eq!(p.z, f32::NEG_INFINITY);
        assert!(p.x.is_nan());
    }
    #[test]
    fn test_point_display(){
        let p = Point::new(1.0, 2.0, 3.0);
        assert_eq!(format!("{}", p), "Point(x = 1, y = 2, z = 3)");
        let p = Point::new(1.0, 2.201, -3.0);
        assert_eq!(format!("{}", p), "Point(x = 1, y = 2.201, z = -3)");
    }
    #[test]
    fn test_point_vector_sum(){
        let v = Vector::new(1.0, 2.0, 3.0);
        let p = Point::new(0.0, 1.0, 5.0);
        let result = Point::new(1.0, 3.0, 8.0);
        assert_eq!(p + v, result);
        assert_eq!(v + p, result);
    }
    #[test]
    fn test_point_subtraction(){
        let p_final = Point::new(1.0, 2.0, 3.0);
        let p_initial = Point::new(-1.0, -2.0, 3.0);
        let v = Vector::new(2.0,4.0,0.0);
        assert_eq!(p_final - p_initial, v);
        assert_eq!(p_initial - p_final, -v);
    }
    #[test]
    fn test_point_minus_vector(){
        let p = Point::new(-1.0, -2.0, -3.0);
        let v = Vector::new(2.0,4.0,0.0);
        assert_eq!(p - v, Point::new(-3.0,-6.0,-3.0));
    }
    #[test]
    fn test_point_to_vec(){
        let p = Point::new(1.0, 2.0, 3.0);
        assert_eq!(p.to_vec(), Vector::new(1.0, 2.0, 3.0));
    }

    //======================= Normal ==========================
    #[test]
    fn test_normal_constructor(){
        let n = Normal::new(1.0, 2.0, 3.0);
        assert_eq!(n, Normal::new(1.0, 2.0, 3.0));
        let n = Normal::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
        assert_eq!(n.y, f32::INFINITY);
        assert_eq!(n.z, f32::NEG_INFINITY);
        assert!(n.x.is_nan());
    }
    #[test]
    fn test_normal_display(){
        let n = Normal::new(1.0, 2.0, 3.0);
        assert_eq!(format!("{}", n), "Normal(x = 1, y = 2, z = 3)");
    }
    #[test]
    fn test_normal_is_close(){
        let n = Normal::new(1.0, 2.0, 3.0);
        let n2 = Normal::new(1.0000001, 2.000000003, 3.0000000005);
        assert_eq!(n.is_close(&n2), true);
        let n2 = Normal::new(10.0, 10.0, 0.0);
        assert_eq!(n.is_close(&n2), false);
        let n2 = Normal::new(1.00001, 2.000000003, 3.0000000005);
        assert_eq!(n.is_close(&n2), false); // x1 - x2 > 0.00001!!!
    }
    #[test]
    fn test_normal_neg(){
        let n = Normal::new(1.0, 2.0, 3.0);
        assert_eq!(-n, Normal::new(-1.0, -2.0, -3.0));
    }
    #[test]
    fn test_normal_scalar_multiplication() {
        let n = Normal::new(1.0, 2.0, 3.0);
        assert_eq!(n * 2.0, Normal::new(2.0, 4.0, 6.0));
        assert_eq!(0.5 * n, Normal::new(0.5, 1.0, 1.5));
        assert_eq!(n / 3.0, Normal::new(1.0 / 3.0, 2.0 / 3.0, 3.0 / 3.0));
        let n = n/-0.0;
        assert!(n.x.is_infinite());
        let n = n/f32::NAN;
        assert!(n.x.is_nan());
    }
    #[test]
    fn test_dot_vec_vec(){
        let v = Vector::new(1.0,2.0,3.0);
        let u = Vector::new(2.0,3.0,4.0);
        assert_eq!(v.dot(u),20.0)
    }
    #[test]
    fn test_cross_vec_vec(){
        let v = Vector::new(1.0,2.0,3.0);
        let u = Vector::new(2.0,3.0,4.0);
        let ax = 2.0 * 4.0 - 3.0 * 3.0;
        let ay = 3.0 * 2.0 - 1.0 * 4.0;
        let az = 1.0 * 3.0 - 2.0 * 2.0;
        assert_eq!(v.cross(u).x,ax);
        assert_eq!(v.cross(u).y,ay);
        assert_eq!(v.cross(u).z,az);
    }

    #[test]
    fn test_dot_normal_vec(){
        let v = Vector::new(1.0,2.0,3.0);
        let u = Normal::new(2.0,3.0,4.0);
        assert_eq!(u.dot(&v),20.0);
    }
}