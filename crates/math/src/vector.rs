use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// A 3D vector with f64 components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    /// Creates a new `Vector3D` from its components.
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the zero vector (0, 0, 0).
    #[inline]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Returns the unit vector along the X axis.
    #[inline]
    pub fn unit_x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Returns the unit vector along the Y axis.
    #[inline]
    pub fn unit_y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    /// Returns the unit vector along the Z axis.
    #[inline]
    pub fn unit_z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Returns the Euclidean norm (magnitude) of the vector.
    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Alias for [`norm`](Self::norm).
    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.norm()
    }

    /// Returns the squared Euclidean norm.
    #[inline]
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns a normalized (unit) vector.
    ///
    /// # Panics
    /// Panics if the vector has zero length.
    #[inline]
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        assert!(n > 0.0, "Cannot normalize a zero vector");
        *self / n
    }

    /// Tries to normalize the vector. Returns `None` if the vector is zero.
    #[inline]
    pub fn try_normalize(&self) -> Option<Self> {
        let n = self.norm();
        if n > 0.0 {
            Some(*self / n)
        } else {
            None
        }
    }

    /// Computes the dot product with another vector.
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Computes the cross product with another vector.
    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Returns the angle (in radians) between this vector and another.
    #[inline]
    pub fn angle_to(&self, other: &Self) -> f64 {
        let cos_theta = (self.dot(other) / (self.norm() * other.norm())).clamp(-1.0, 1.0);
        cos_theta.acos()
    }

    /// Returns `true` if all components are finite.
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Returns `true` if the vector is normalized within the given tolerance.
    #[inline]
    pub fn is_normalized(&self, tol: f64) -> bool {
        (self.norm_squared() - 1.0).abs() <= tol
    }

    /// Computes the Euclidean distance to another vector.
    #[inline]
    pub fn distance_to(&self, other: &Self) -> f64 {
        (*self - *other).norm()
    }

    /// Performs linear interpolation between this vector and `other` by factor `t`.
    /// `t = 0.0` gives `self`, `t = 1.0` gives `other`.
    #[inline]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        *self + (*other - *self) * t
    }

    /// Rotates the vector around the X axis by the given angle (in radians).
    #[inline]
    pub fn rotate_x(&self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }

    /// Rotates the vector around the Y axis by the given angle (in radians).
    #[inline]
    pub fn rotate_y(&self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }

    /// Rotates the vector around the Z axis by the given angle (in radians).
    #[inline]
    pub fn rotate_z(&self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }

    /// Rotates the vector around the given `axis` by the given `angle` (in radians)
    /// using Rodrigues' rotation formula.
    ///
    /// The `axis` must be a unit vector.
    pub fn rotate(&self, axis: &Self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let dot = self.dot(axis);
        let cross = axis.cross(self);
        *self * cos_a + *axis * dot * (1.0 - cos_a) + cross * sin_a
    }

    /// Reflects the vector about a surface with the given `normal`.
    ///
    /// The `normal` must be a unit vector.
    #[inline]
    pub fn reflect(&self, normal: &Self) -> Self {
        *self - *normal * 2.0 * self.dot(normal)
    }

    /// Clamps the magnitude of the vector to `max`.
    #[inline]
    pub fn clamp_magnitude(&self, max: f64) -> Self {
        let n = self.norm_squared();
        if n > max * max {
            *self * (max / n.sqrt())
        } else {
            *self
        }
    }

    /// Returns a vector with the absolute values of each component.
    #[inline]
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    /// Returns the component-wise minimum of two vectors.
    #[inline]
    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    /// Returns the component-wise maximum of two vectors.
    #[inline]
    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    /// Clamps each component between `min` and `max`.
    #[inline]
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
            z: self.z.clamp(min.z, max.z),
        }
    }

    /// Converts the vector to a fixed-size array `[f64; 3]`.
    #[inline]
    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Converts the vector to a tuple `(f64, f64, f64)`.
    #[inline]
    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }

    /// Creates a `Vector3D` from a fixed-size array `[f64; 3]`.
    #[inline]
    pub fn from_array(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }

    /// Creates a `Vector3D` from a tuple `(f64, f64, f64)`.
    #[inline]
    pub fn from_tuple(t: (f64, f64, f64)) -> Self {
        Self::new(t.0, t.1, t.2)
    }

    /// Returns a slice view of the vector's components.
    #[inline]
    pub fn as_slice(&self) -> &[f64] {
        // SAFETY: Vector3D is repr(C) compatible with [f64; 3]
        unsafe { std::slice::from_raw_parts(&self.x as *const f64, 3) }
    }

    /// Checks approximate equality component-wise within the given epsilon.
    #[inline]
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.z - other.z).abs() <= epsilon
    }
}

// ---- Arithmetic trait implementations ----

impl Add for Vector3D {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector3D {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vector3D {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// Component-wise multiplication.
impl Mul<Vector3D> for Vector3D {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Vector3D) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Div<f64> for Vector3D {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vector3D {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl AddAssign for Vector3D {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vector3D {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl MulAssign<f64> for Vector3D {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl DivAssign<f64> for Vector3D {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Index<usize> for Vector3D {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vector3D index out of bounds: {index}"),
        }
    }
}

impl IndexMut<usize> for Vector3D {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vector3D index out of bounds: {index}"),
        }
    }
}

impl fmt::Display for Vector3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Default for Vector3D {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl From<[f64; 3]> for Vector3D {
    #[inline]
    fn from(arr: [f64; 3]) -> Self {
        Self::from_array(arr)
    }
}

impl From<(f64, f64, f64)> for Vector3D {
    #[inline]
    fn from(t: (f64, f64, f64)) -> Self {
        Self::from_tuple(t)
    }
}

// ---- Scalar multiplication on the left side ----

impl Mul<Vector3D> for f64 {
    type Output = Vector3D;

    #[inline]
    fn mul(self, rhs: Vector3D) -> Vector3D {
        rhs * self
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn test_new_and_default() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);

        let z = Vector3D::zero();
        assert_eq!(z, Vector3D::new(0.0, 0.0, 0.0));

        assert_eq!(Vector3D::default(), Vector3D::zero());
    }

    #[test]
    fn test_unit_vectors() {
        assert_eq!(Vector3D::unit_x(), Vector3D::new(1.0, 0.0, 0.0));
        assert_eq!(Vector3D::unit_y(), Vector3D::new(0.0, 1.0, 0.0));
        assert_eq!(Vector3D::unit_z(), Vector3D::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_norm_and_magnitude() {
        let v = Vector3D::new(3.0, 4.0, 0.0);
        assert_eq!(v.norm(), 5.0);
        assert_eq!(v.magnitude(), 5.0);
        assert_eq!(v.norm_squared(), 25.0);
    }

    #[test]
    fn test_normalize() {
        let v = Vector3D::new(3.0, 0.0, 0.0);
        assert_eq!(v.normalize(), Vector3D::unit_x());

        let v2 = Vector3D::new(1.0, 2.0, 2.0);
        let n = v2.normalize();
        assert!((n.norm() - 1.0).abs() < 1e-15);
    }

    #[test]
    #[should_panic(expected = "Cannot normalize")]
    fn test_normalize_panic() {
        Vector3D::zero().normalize();
    }

    #[test]
    fn test_try_normalize() {
        assert!(Vector3D::zero().try_normalize().is_none());
        assert!(Vector3D::unit_x().try_normalize().is_some());
    }

    #[test]
    fn test_dot() {
        let a = Vector3D::new(1.0, 2.0, 3.0);
        let b = Vector3D::new(4.0, 5.0, 6.0);
        assert_eq!(a.dot(&b), 32.0);
    }

    #[test]
    fn test_cross() {
        let a = Vector3D::unit_x();
        let b = Vector3D::unit_y();
        assert_eq!(a.cross(&b), Vector3D::unit_z());

        // anti-commutative
        assert_eq!(a.cross(&b), -b.cross(&a));
    }

    #[test]
    fn test_angle_to() {
        let a = Vector3D::unit_x();
        let b = Vector3D::unit_y();
        assert!((a.angle_to(&b) - FRAC_PI_2).abs() < 1e-15);

        let c = Vector3D::new(1.0, 1.0, 0.0).normalize();
        assert!((a.angle_to(&c) - std::f64::consts::FRAC_PI_4).abs() < 1e-15);
    }

    #[test]
    fn test_is_finite() {
        assert!(Vector3D::new(1.0, 2.0, 3.0).is_finite());
        assert!(!Vector3D::new(f64::NAN, 2.0, 3.0).is_finite());
        assert!(!Vector3D::new(f64::INFINITY, 2.0, 3.0).is_finite());
    }

    #[test]
    fn test_is_normalized() {
        assert!(Vector3D::unit_x().is_normalized(1e-10));
        assert!(!Vector3D::new(2.0, 0.0, 0.0).is_normalized(1e-10));
    }

    #[test]
    fn test_distance() {
        let a = Vector3D::zero();
        let b = Vector3D::new(3.0, 4.0, 0.0);
        assert_eq!(a.distance_to(&b), 5.0);
    }

    #[test]
    fn test_lerp() {
        let a = Vector3D::zero();
        let b = Vector3D::unit_x();
        assert_eq!(a.lerp(&b, 0.5), Vector3D::new(0.5, 0.0, 0.0));
        assert_eq!(a.lerp(&b, 0.0), a);
        assert_eq!(a.lerp(&b, 1.0), b);
    }

    #[test]
    fn test_rotate_x() {
        let v = Vector3D::new(0.0, 1.0, 0.0);
        let rotated = v.rotate_x(FRAC_PI_2);
        assert!(rotated.approx_eq(&Vector3D::new(0.0, 0.0, 1.0), 1e-15));
    }

    #[test]
    fn test_rotate_y() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let rotated = v.rotate_y(FRAC_PI_2);
        assert!(rotated.approx_eq(&Vector3D::new(0.0, 0.0, -1.0), 1e-15));
    }

    #[test]
    fn test_rotate_z() {
        let v = Vector3D::new(1.0, 0.0, 0.0);
        let rotated = v.rotate_z(FRAC_PI_2);
        assert!(rotated.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_rotate_axis() {
        let v = Vector3D::new(0.0, 1.0, 0.0);
        let axis = Vector3D::unit_z();
        let rotated = v.rotate(&axis, FRAC_PI_2);
        assert!(rotated.approx_eq(&Vector3D::new(-1.0, 0.0, 0.0), 1e-15));
    }

    #[test]
    fn test_reflect() {
        let v = Vector3D::new(1.0, -1.0, 0.0);
        let n = Vector3D::unit_y();
        let r = v.reflect(&n);
        assert_eq!(r, Vector3D::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn test_clamp_magnitude() {
        let v = Vector3D::new(10.0, 0.0, 0.0);
        assert_eq!(v.clamp_magnitude(5.0), Vector3D::new(5.0, 0.0, 0.0));

        let v2 = Vector3D::new(1.0, 0.0, 0.0);
        assert_eq!(v2.clamp_magnitude(5.0), v2);
    }

    #[test]
    fn test_abs_min_max_clamp() {
        let a = Vector3D::new(-1.0, 2.0, -3.0);
        assert_eq!(a.abs(), Vector3D::new(1.0, 2.0, 3.0));

        let b = Vector3D::new(0.0, 1.0, -2.0);
        assert_eq!(a.min(&b), Vector3D::new(-1.0, 1.0, -3.0));
        assert_eq!(a.max(&b), Vector3D::new(0.0, 2.0, -2.0));

        let lo = Vector3D::new(-0.5, -0.5, -0.5);
        let hi = Vector3D::new(0.5, 0.5, 0.5);
        let clamped = a.clamp(&lo, &hi);
        assert_eq!(clamped, Vector3D::new(-0.5, 0.5, -0.5));
    }

    #[test]
    fn test_conversions() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(v.to_array(), [1.0, 2.0, 3.0]);
        assert_eq!(v.to_tuple(), (1.0, 2.0, 3.0));
        assert_eq!(Vector3D::from_array([1.0, 2.0, 3.0]), v);
        assert_eq!(Vector3D::from_tuple((1.0, 2.0, 3.0)), v);
        assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);

        let from_arr: Vector3D = [1.0, 2.0, 3.0].into();
        assert_eq!(from_arr, v);
        let from_tup: Vector3D = (1.0, 2.0, 3.0).into();
        assert_eq!(from_tup, v);
    }

    #[test]
    fn test_arithmetic() {
        let a = Vector3D::new(1.0, 2.0, 3.0);
        let b = Vector3D::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vector3D::new(5.0, 7.0, 9.0));
        assert_eq!(a - b, Vector3D::new(-3.0, -3.0, -3.0));
        assert_eq!(a * 2.0, Vector3D::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * a, Vector3D::new(2.0, 4.0, 6.0));
        assert_eq!(a * b, Vector3D::new(4.0, 10.0, 18.0)); // component-wise
        assert_eq!(a / 2.0, Vector3D::new(0.5, 1.0, 1.5));
        assert_eq!(-a, Vector3D::new(-1.0, -2.0, -3.0));

        // Assign
        let mut c = a;
        c += b;
        assert_eq!(c, Vector3D::new(5.0, 7.0, 9.0));
        c -= a;
        assert_eq!(c, Vector3D::new(4.0, 5.0, 6.0));
        c *= 2.0;
        assert_eq!(c, Vector3D::new(8.0, 10.0, 12.0));
        c /= 2.0;
        assert_eq!(c, Vector3D::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_index() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 2.0);
        assert_eq!(v[2], 3.0);

        let mut w = v;
        w[0] = 10.0;
        assert_eq!(w.x, 10.0);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_out_of_bounds() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        let _ = v[3];
    }

    #[test]
    fn test_display() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(format!("{v}"), "(1, 2, 3)");
    }

    #[test]
    fn test_approx_eq() {
        let a = Vector3D::new(1.0, 2.0, 3.0);
        let b = Vector3D::new(1.0 + 1e-10, 2.0 + 1e-10, 3.0 + 1e-10);
        assert!(a.approx_eq(&b, 1e-9));
        assert!(!a.approx_eq(&b, 1e-11));
    }
}