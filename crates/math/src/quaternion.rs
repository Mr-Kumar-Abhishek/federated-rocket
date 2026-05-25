use crate::matrix::Matrix3x3;
use crate::vector::Vector3D;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub},
};

/// A quaternion representing rotation in 3D space.
///
/// Stored as `w + xi + yj + zk`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    /// Creates a new `Quaternion` from its components.
    #[inline]
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    /// Returns the identity quaternion (no rotation).
    #[inline]
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0)
    }

    /// Returns the zero quaternion.
    #[inline]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a quaternion from an axis-angle representation.
    ///
    /// The `axis` must be a unit vector.
    pub fn from_axis_angle(axis: Vector3D, angle: f64) -> Self {
        let half = angle * 0.5;
        let s = half.sin();
        Self {
            w: half.cos(),
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
        }
    }

    /// Creates a quaternion from Euler angles (roll, pitch, yaw) using the ZYX convention.
    ///
    /// The angles are applied in order: yaw (Z), then pitch (Y), then roll (X).
    pub fn from_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        let half_roll = roll * 0.5;
        let half_pitch = pitch * 0.5;
        let half_yaw = yaw * 0.5;

        let cr = half_roll.cos();
        let sr = half_roll.sin();
        let cp = half_pitch.cos();
        let sp = half_pitch.sin();
        let cy = half_yaw.cos();
        let sy = half_yaw.sin();

        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    /// Creates a quaternion from a rotation matrix.
    ///
    /// Returns `None` if the matrix has trace < -0.999999 (near-180° rotation edge case handled).
    pub fn from_rotation_matrix(m: &Matrix3x3) -> Option<Self> {
        let trace = m.trace();
        let d = m.data;

        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0; // s = 4 * w
            let w = 0.25 * s;
            let x = (d[2][1] - d[1][2]) / s;
            let y = (d[0][2] - d[2][0]) / s;
            let z = (d[1][0] - d[0][1]) / s;
            Some(Self { w, x, y, z })
        } else if d[0][0] > d[1][1] && d[0][0] > d[2][2] {
            let s = (1.0 + d[0][0] - d[1][1] - d[2][2]).sqrt() * 2.0;
            let w = (d[2][1] - d[1][2]) / s;
            let x = 0.25 * s;
            let y = (d[0][1] + d[1][0]) / s;
            let z = (d[0][2] + d[2][0]) / s;
            Some(Self { w, x, y, z })
        } else if d[1][1] > d[2][2] {
            let s = (1.0 + d[1][1] - d[0][0] - d[2][2]).sqrt() * 2.0;
            let w = (d[0][2] - d[2][0]) / s;
            let x = (d[0][1] + d[1][0]) / s;
            let y = 0.25 * s;
            let z = (d[1][2] + d[2][1]) / s;
            Some(Self { w, x, y, z })
        } else {
            let s = (1.0 + d[2][2] - d[0][0] - d[1][1]).sqrt() * 2.0;
            let w = (d[1][0] - d[0][1]) / s;
            let x = (d[0][2] + d[2][0]) / s;
            let y = (d[1][2] + d[2][1]) / s;
            let z = 0.25 * s;
            Some(Self { w, x, y, z })
        }
    }

    /// Returns the norm (magnitude) of the quaternion.
    #[inline]
    pub fn norm(&self) -> f64 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns a normalized (unit) quaternion.
    ///
    /// # Panics
    /// Panics if the quaternion has zero length.
    #[inline]
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        assert!(n > 0.0, "Cannot normalize a zero quaternion");
        *self * (1.0 / n)
    }

    /// Returns the conjugate of the quaternion.
    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Returns the inverse of the quaternion.
    ///
    /// For a unit quaternion, this is equivalent to the conjugate.
    #[inline]
    pub fn inverse(&self) -> Self {
        let n = self.norm_squared();
        if n == 0.0 {
            Self::zero()
        } else {
            self.conjugate() * (1.0 / n)
        }
    }

    /// Rotates a vector by this quaternion.
    ///
    /// This computes `q * v * q^-1` where `v` is treated as a pure quaternion.
    #[inline]
    pub fn rotate(&self, v: Vector3D) -> Vector3D {
        let qv = Vector3D::new(self.x, self.y, self.z);
        let uv = qv.cross(&v);
        let uuv = qv.cross(&uv);
        // v + 2 * (w * uv + uuv)
        v + (uv * (2.0 * self.w)) + (uuv * 2.0)
    }

    /// Converts the quaternion to axis-angle representation.
    ///
    /// Returns `(axis, angle)` where `axis` is a unit vector and `angle` is in radians.
    pub fn to_axis_angle(&self) -> (Vector3D, f64) {
        let angle = 2.0 * self.w.acos();
        let s = (1.0 - self.w * self.w).sqrt();
        if s < 1e-12 {
            (Vector3D::unit_x(), angle)
        } else {
            let axis = Vector3D::new(self.x / s, self.y / s, self.z / s);
            (axis, angle)
        }
    }

    /// Converts the quaternion to a 3x3 rotation matrix.
    pub fn to_rotation_matrix(&self) -> Matrix3x3 {
        Matrix3x3::from_quaternion(self)
    }

    /// Converts the quaternion to Euler angles (roll, pitch, yaw) using the ZYX convention.
    ///
    /// Returns `(roll, pitch, yaw)` in radians.
    pub fn to_euler(&self) -> (f64, f64, f64) {
        let (w, x, y, z) = (self.w, self.x, self.y, self.z);

        // Roll (x-axis rotation)
        let sin_roll = 2.0 * (w * x + y * z);
        let cos_roll = 1.0 - 2.0 * (x * x + y * y);
        let roll = sin_roll.atan2(cos_roll);

        // Pitch (y-axis rotation)
        let sin_pitch = 2.0 * (w * y - z * x);
        let pitch = if sin_pitch.abs() >= 1.0 {
            std::f64::consts::FRAC_PI_2.copysign(sin_pitch)
        } else {
            sin_pitch.asin()
        };

        // Yaw (z-axis rotation)
        let sin_yaw = 2.0 * (w * z + x * y);
        let cos_yaw = 1.0 - 2.0 * (y * y + z * z);
        let yaw = sin_yaw.atan2(cos_yaw);

        (roll, pitch, yaw)
    }

    /// Computes the dot product of two quaternions.
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Performs spherical linear interpolation (slerp) between this quaternion
    /// and `other` by factor `t`.
    ///
    /// `t = 0.0` gives `self`, `t = 1.0` gives `other`.
    pub fn slerp(&self, other: &Self, t: f64) -> Self {
        let mut cos_theta = self.dot(other);
        let mut other_adjusted = *other;

        // Take the shorter path
        if cos_theta < 0.0 {
            cos_theta = -cos_theta;
            other_adjusted = -other_adjusted;
        }

        // Use linear interpolation for very small angles
        if cos_theta > 0.9995 {
            let result = *self + (other_adjusted - *self) * t;
            return result.normalize();
        }

        let theta = cos_theta.acos();
        let sin_theta = theta.sin();
        let scale0 = ((1.0 - t) * theta).sin() / sin_theta;
        let scale1 = (t * theta).sin() / sin_theta;

        *self * scale0 + other_adjusted * scale1
    }

    /// Returns the squared norm of the quaternion.
    #[inline]
    fn norm_squared(&self) -> f64 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Checks approximate equality component-wise within the given epsilon.
    #[inline]
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.w - other.w).abs() <= epsilon
            && (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.z - other.z).abs() <= epsilon
    }
}

// ---- Arithmetic trait implementations ----

/// Hamilton product of two quaternions.
impl Mul<Quaternion> for Quaternion {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Quaternion) -> Self {
        let (a1, b1, c1, d1) = (self.w, self.x, self.y, self.z);
        let (a2, b2, c2, d2) = (rhs.w, rhs.x, rhs.y, rhs.z);

        Self {
            w: a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            x: a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
            y: a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
            z: a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
        }
    }
}

impl Mul<f64> for Quaternion {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self {
            w: self.w * rhs,
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Add for Quaternion {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            w: self.w + rhs.w,
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Quaternion {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            w: self.w - rhs.w,
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Neg for Quaternion {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            w: -self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl AddAssign for Quaternion {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.w += rhs.w;
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl MulAssign<f64> for Quaternion {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.w *= rhs;
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

// ---- Scalar multiplication on the left side ----

impl Mul<Quaternion> for f64 {
    type Output = Quaternion;

    #[inline]
    fn mul(self, rhs: Quaternion) -> Quaternion {
        rhs * self
    }
}

impl fmt::Display for Quaternion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} + {}i + {}j + {}k)", self.w, self.x, self.y, self.z)
    }
}

impl Default for Quaternion {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    #[test]
    fn test_new_and_default() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 2.0);
        assert_eq!(q.y, 3.0);
        assert_eq!(q.z, 4.0);
        assert_eq!(Quaternion::default(), Quaternion::identity());
    }

    #[test]
    fn test_identity_and_zero() {
        assert_eq!(Quaternion::identity(), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(Quaternion::zero(), Quaternion::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_from_axis_angle() {
        let q = Quaternion::from_axis_angle(Vector3D::unit_z(), FRAC_PI_2);
        let v = q.rotate(Vector3D::unit_x());
        assert!(v.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_from_euler_zyx() {
        // 90° yaw (Z) should rotate X to Y
        let q = Quaternion::from_euler(0.0, 0.0, FRAC_PI_2);
        let v = q.rotate(Vector3D::unit_x());
        assert!(v.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_from_rotation_matrix() {
        let rot = Matrix3x3::rotation_z(FRAC_PI_2);
        let q = Quaternion::from_rotation_matrix(&rot).unwrap();
        let v = q.rotate(Vector3D::unit_x());
        assert!(v.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-14));
    }

    #[test]
    fn test_norm_and_normalize() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        assert!((q.norm() - (30.0_f64).sqrt()).abs() < 1e-15);
        let n = q.normalize();
        assert!((n.norm() - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_conjugate_and_inverse() {
        let q = Quaternion::from_axis_angle(Vector3D::unit_z(), FRAC_PI_4);
        let inv = q.inverse();
        let result = q * inv;
        assert!(result.approx_eq(&Quaternion::identity(), 1e-15));

        // Conjugate of a unit quaternion is its inverse
        let conj = q.conjugate();
        assert!(conj.approx_eq(&inv, 1e-15));
    }

    #[test]
    fn test_rotate_vector() {
        // Rotate (1, 0, 0) by 90° around Z → (0, 1, 0)
        let q = Quaternion::from_axis_angle(Vector3D::unit_z(), FRAC_PI_2);
        let v = q.rotate(Vector3D::new(1.0, 0.0, 0.0));
        assert!(v.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_to_axis_angle() {
        let q = Quaternion::from_axis_angle(Vector3D::unit_z(), FRAC_PI_2);
        let (axis, angle) = q.to_axis_angle();
        assert!(axis.approx_eq(&Vector3D::unit_z(), 1e-10));
        assert!((angle - FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn test_to_rotation_matrix() {
        let q = Quaternion::from_axis_angle(Vector3D::unit_z(), FRAC_PI_2);
        let m = q.to_rotation_matrix();
        let v = m * Vector3D::unit_x();
        assert!(v.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_to_euler() {
        // Pure yaw of PI/2 should give back (0, 0, PI/2)
        let q = Quaternion::from_euler(0.0, 0.0, FRAC_PI_2);
        let (roll, pitch, yaw) = q.to_euler();
        assert!((roll).abs() < 1e-10);
        assert!((pitch).abs() < 1e-10);
        assert!((yaw - FRAC_PI_2).abs() < 1e-10);

        // Pure roll of PI/4
        let q2 = Quaternion::from_euler(FRAC_PI_4, 0.0, 0.0);
        let (roll2, pitch2, yaw2) = q2.to_euler();
        assert!((roll2 - FRAC_PI_4).abs() < 1e-10);
        assert!((pitch2).abs() < 1e-10);
        assert!((yaw2).abs() < 1e-10);
    }

    #[test]
    fn test_dot() {
        let a = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let b = Quaternion::new(5.0, 6.0, 7.0, 8.0);
        assert_eq!(a.dot(&b), 70.0);
    }

    #[test]
    fn test_slerp() {
        let q1 = Quaternion::identity();
        let q2 = Quaternion::from_axis_angle(Vector3D::unit_z(), FRAC_PI_2);

        // slerp at t=0 should give q1
        let r0 = q1.slerp(&q2, 0.0);
        assert!(r0.approx_eq(&q1, 1e-15));

        // slerp at t=1 should give q2
        let r1 = q1.slerp(&q2, 1.0);
        assert!(r1.approx_eq(&q2, 1e-15));

        // slerp at t=0.5
        let r05 = q1.slerp(&q2, 0.5);
        assert!((r05.norm() - 1.0).abs() < 1e-15);

        // Rotating by the half-way quaternion should give 45°
        let v = r05.rotate(Vector3D::unit_x());
        assert!(v.approx_eq(
            &Vector3D::new((FRAC_PI_4).cos(), (FRAC_PI_4).sin(), 0.0),
            1e-15
        ));
    }

    #[test]
    fn test_hamilton_product() {
        // i * j = k
        let i = Quaternion::new(0.0, 1.0, 0.0, 0.0);
        let j = Quaternion::new(0.0, 0.0, 1.0, 0.0);
        let k = i * j;
        assert!(k.approx_eq(&Quaternion::new(0.0, 0.0, 0.0, 1.0), 1e-15));
    }

    #[test]
    fn test_arithmetic() {
        let a = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let b = Quaternion::new(5.0, 6.0, 7.0, 8.0);

        assert_eq!(a + b, Quaternion::new(6.0, 8.0, 10.0, 12.0));
        assert_eq!(a - b, Quaternion::new(-4.0, -4.0, -4.0, -4.0));
        assert_eq!(a * 2.0, Quaternion::new(2.0, 4.0, 6.0, 8.0));
        assert_eq!(2.0 * a, Quaternion::new(2.0, 4.0, 6.0, 8.0));
        assert_eq!(-a, Quaternion::new(-1.0, -2.0, -3.0, -4.0));

        let mut c = a;
        c += b;
        assert_eq!(c, Quaternion::new(6.0, 8.0, 10.0, 12.0));
        c *= 2.0;
        assert_eq!(c, Quaternion::new(12.0, 16.0, 20.0, 24.0));
    }

    #[test]
    fn test_display() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let s = format!("{q}");
        assert!(s.contains("1"));
        assert!(s.contains("i"));
    }

    #[test]
    fn test_approx_eq() {
        let a = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let b = Quaternion::new(1.0 + 1e-10, 2.0, 3.0, 4.0 + 1e-10);
        assert!(a.approx_eq(&b, 1e-9));
        assert!(!a.approx_eq(&b, 1e-11));
    }

    #[test]
    fn test_rotation_roundtrip() {
        // axis-angle → quaternion → axis-angle
        let axis = Vector3D::new(1.0, 2.0, 3.0).normalize();
        let angle = 0.723;
        let q = Quaternion::from_axis_angle(axis, angle);
        let (axis2, angle2) = q.to_axis_angle();
        assert!(axis.approx_eq(&axis2, 1e-10));
        assert!((angle - angle2).abs() < 1e-10);
    }

    #[test]
    fn test_euler_roll_pitch_yaw() {
        // Test that converting to euler and back preserves rotation
        let original_roll = 0.3;
        let original_pitch = -0.2;
        let original_yaw = 1.5;
        let q = Quaternion::from_euler(original_roll, original_pitch, original_yaw);
        let (r, p, y) = q.to_euler();
        // The conversion may not be exact due to singularities, but should be close
        assert!((r - original_roll).abs() < 1e-10);
        assert!((p - original_pitch).abs() < 1e-10);
        assert!((y - original_yaw).abs() < 1e-10);
    }
}
