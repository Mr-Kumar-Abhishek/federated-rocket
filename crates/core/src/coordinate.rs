use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 3-dimensional coordinate with `x`, `y`, `z` components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Coordinate {
    /// Creates a new `Coordinate` from `x`, `y`, `z` values.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Coordinate { x, y, z }
    }

    /// Returns the origin (0, 0, 0).
    pub const fn origin() -> Self {
        Coordinate {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Computes the Euclidean distance from this coordinate to another.
    pub fn distance_to(&self, other: &Coordinate) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Computes the angle (in radians) between the vectors from the origin to
    /// this coordinate and to another coordinate.
    pub fn angle_to(&self, other: &Coordinate) -> f64 {
        let dot = self.dot(other);
        let norm_product = self.norm() * other.norm();
        if norm_product == 0.0 {
            0.0
        } else {
            (dot / norm_product).acos()
        }
    }

    /// Rotates this coordinate around the given axis by the specified angle (radians).
    ///
    /// Uses Rodrigues' rotation formula.
    pub fn rotate(&self, axis: &Coordinate, angle: f64) -> Self {
        let k = axis.normalize();
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // v_rot = v * cos_a + (k × v) * sin_a + k * (k · v) * (1 - cos_a)
        let v = self;
        let k_cross_v = k.cross(v);
        let k_dot_v = k.dot(v);

        Coordinate {
            x: v.x * cos_a + k_cross_v.x * sin_a + k.x * k_dot_v * (1.0 - cos_a),
            y: v.y * cos_a + k_cross_v.y * sin_a + k.y * k_dot_v * (1.0 - cos_a),
            z: v.z * cos_a + k_cross_v.z * sin_a + k.z * k_dot_v * (1.0 - cos_a),
        }
    }

    /// Applies a 3×3 transformation matrix to this coordinate.
    pub fn transform(&self, matrix: &[[f64; 3]; 3]) -> Self {
        Coordinate {
            x: self.x * matrix[0][0] + self.y * matrix[0][1] + self.z * matrix[0][2],
            y: self.x * matrix[1][0] + self.y * matrix[1][1] + self.z * matrix[1][2],
            z: self.x * matrix[2][0] + self.y * matrix[2][1] + self.z * matrix[2][2],
        }
    }

    /// Returns the Euclidean norm (magnitude) of this coordinate vector.
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns a normalized (unit) vector in the same direction.
    ///
    /// Returns `Coordinate::origin()` if the norm is zero.
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n == 0.0 {
            Coordinate::origin()
        } else {
            Coordinate {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            }
        }
    }

    /// Computes the cross product of this coordinate with another.
    pub fn cross(&self, other: &Coordinate) -> Self {
        Coordinate {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Computes the dot product of this coordinate with another.
    pub fn dot(&self, other: &Coordinate) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the zero vector (same as `origin()`).
    pub fn zero() -> Self {
        Coordinate::origin()
    }
}

// --- Arithmetic operations ---

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f64> for Coordinate {
    type Output = Coordinate;

    fn mul(self, scalar: f64) -> Coordinate {
        Coordinate {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Div<f64> for Coordinate {
    type Output = Coordinate;

    fn div(self, scalar: f64) -> Coordinate {
        Coordinate {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl Neg for Coordinate {
    type Output = Coordinate;

    fn neg(self) -> Coordinate {
        Coordinate {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    #[test]
    fn test_new_and_origin() {
        let c = Coordinate::new(1.0, 2.0, 3.0);
        assert!((c.x - 1.0).abs() < EPS);
        assert!((c.y - 2.0).abs() < EPS);
        assert!((c.z - 3.0).abs() < EPS);

        let o = Coordinate::origin();
        assert!((o.x - 0.0).abs() < EPS);
        assert!((o.y - 0.0).abs() < EPS);
        assert!((o.z - 0.0).abs() < EPS);
    }

    #[test]
    fn test_distance_to() {
        let a = Coordinate::new(0.0, 0.0, 0.0);
        let b = Coordinate::new(3.0, 4.0, 0.0);
        assert!((a.distance_to(&b) - 5.0).abs() < EPS);

        let c = Coordinate::new(1.0, 2.0, 3.0);
        let d = Coordinate::new(1.0, 2.0, 3.0);
        assert!((c.distance_to(&d) - 0.0).abs() < EPS);
    }

    #[test]
    fn test_angle_to() {
        let a = Coordinate::new(1.0, 0.0, 0.0);
        let b = Coordinate::new(0.0, 1.0, 0.0);
        assert!((a.angle_to(&b) - std::f64::consts::FRAC_PI_2).abs() < EPS);

        let c = Coordinate::new(1.0, 0.0, 0.0);
        let d = Coordinate::new(1.0, 0.0, 0.0);
        assert!((c.angle_to(&d) - 0.0).abs() < EPS);
    }

    #[test]
    fn test_norm() {
        let c = Coordinate::new(3.0, 4.0, 0.0);
        assert!((c.norm() - 5.0).abs() < EPS);

        let o = Coordinate::origin();
        assert!((o.norm() - 0.0).abs() < EPS);
    }

    #[test]
    fn test_normalize() {
        let c = Coordinate::new(3.0, 4.0, 0.0);
        let n = c.normalize();
        assert!((n.norm() - 1.0).abs() < EPS);
        assert!((n.x - 0.6).abs() < EPS);
        assert!((n.y - 0.8).abs() < EPS);
        assert!((n.z - 0.0).abs() < EPS);

        let o = Coordinate::origin();
        let nz = o.normalize();
        assert!((nz.norm() - 0.0).abs() < EPS);
    }

    #[test]
    fn test_cross() {
        let a = Coordinate::new(1.0, 0.0, 0.0);
        let b = Coordinate::new(0.0, 1.0, 0.0);
        let c = a.cross(&b);
        assert!((c.x - 0.0).abs() < EPS);
        assert!((c.y - 0.0).abs() < EPS);
        assert!((c.z - 1.0).abs() < EPS);
    }

    #[test]
    fn test_dot() {
        let a = Coordinate::new(1.0, 2.0, 3.0);
        let b = Coordinate::new(4.0, 5.0, 6.0);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((a.dot(&b) - 32.0).abs() < EPS);
    }

    #[test]
    fn test_rotate_x_axis() {
        let v = Coordinate::new(0.0, 1.0, 0.0);
        let axis = Coordinate::new(1.0, 0.0, 0.0);
        let rotated = v.rotate(&axis, std::f64::consts::FRAC_PI_2);
        // Rotating (0,1,0) around x-axis by 90° should give (0,0,1)
        assert!((rotated.x - 0.0).abs() < EPS);
        assert!((rotated.y - 0.0).abs() < EPS);
        assert!((rotated.z - 1.0).abs() < EPS);
    }

    #[test]
    fn test_rotate_z_axis() {
        let v = Coordinate::new(1.0, 0.0, 0.0);
        let axis = Coordinate::new(0.0, 0.0, 1.0);
        let rotated = v.rotate(&axis, std::f64::consts::FRAC_PI_2);
        // Rotating (1,0,0) around z-axis by 90° should give (0,1,0)
        assert!((rotated.x - 0.0).abs() < EPS);
        assert!((rotated.y - 1.0).abs() < EPS);
        assert!((rotated.z - 0.0).abs() < EPS);
    }

    #[test]
    fn test_transform() {
        // Identity matrix
        let m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let v = Coordinate::new(1.0, 2.0, 3.0);
        let result = v.transform(&m);
        assert!((result.x - 1.0).abs() < EPS);
        assert!((result.y - 2.0).abs() < EPS);
        assert!((result.z - 3.0).abs() < EPS);

        // Scaling matrix
        let scale = [[2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]];
        let scaled = v.transform(&scale);
        assert!((scaled.x - 2.0).abs() < EPS);
        assert!((scaled.y - 4.0).abs() < EPS);
        assert!((scaled.z - 6.0).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_add() {
        let a = Coordinate::new(1.0, 2.0, 3.0);
        let b = Coordinate::new(4.0, 5.0, 6.0);
        let c = a + b;
        assert!((c.x - 5.0).abs() < EPS);
        assert!((c.y - 7.0).abs() < EPS);
        assert!((c.z - 9.0).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_sub() {
        let a = Coordinate::new(5.0, 7.0, 9.0);
        let b = Coordinate::new(1.0, 2.0, 3.0);
        let c = a - b;
        assert!((c.x - 4.0).abs() < EPS);
        assert!((c.y - 5.0).abs() < EPS);
        assert!((c.z - 6.0).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_mul() {
        let a = Coordinate::new(1.0, 2.0, 3.0);
        let b = a * 2.0;
        assert!((b.x - 2.0).abs() < EPS);
        assert!((b.y - 4.0).abs() < EPS);
        assert!((b.z - 6.0).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_div() {
        let a = Coordinate::new(2.0, 4.0, 6.0);
        let b = a / 2.0;
        assert!((b.x - 1.0).abs() < EPS);
        assert!((b.y - 2.0).abs() < EPS);
        assert!((b.z - 3.0).abs() < EPS);
    }

    #[test]
    fn test_neg() {
        let a = Coordinate::new(1.0, -2.0, 3.0);
        let b = -a;
        assert!((b.x + 1.0).abs() < EPS);
        assert!((b.y - 2.0).abs() < EPS);
        assert!((b.z + 3.0).abs() < EPS);
    }

    #[test]
    fn test_display() {
        let c = Coordinate::new(1.5, -2.0, 3.0);
        assert_eq!(format!("{}", c), "(1.5, -2, 3)");
    }

    #[test]
    fn test_zero() {
        let z = Coordinate::zero();
        assert!((z.x - 0.0).abs() < EPS);
        assert!((z.y - 0.0).abs() < EPS);
        assert!((z.z - 0.0).abs() < EPS);
    }
}