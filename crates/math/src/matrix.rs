use crate::vector::Vector3D;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// A 3x3 matrix stored in row-major order.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Matrix3x3 {
    pub data: [[f64; 3]; 3],
}

impl Matrix3x3 {
    /// Creates a new `Matrix3x3` from a row-major `[[f64; 3]; 3]`.
    #[inline]
    pub fn new(data: [[f64; 3]; 3]) -> Self {
        Self { data }
    }

    /// Returns the identity matrix.
    #[inline]
    pub fn identity() -> Self {
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Returns the zero matrix.
    #[inline]
    pub fn zero() -> Self {
        Self {
            data: [[0.0; 3]; 3],
        }
    }

    /// Creates a matrix from three row vectors.
    #[inline]
    pub fn from_rows(r1: Vector3D, r2: Vector3D, r3: Vector3D) -> Self {
        Self {
            data: [[r1.x, r1.y, r1.z], [r2.x, r2.y, r2.z], [r3.x, r3.y, r3.z]],
        }
    }

    /// Creates a matrix from three column vectors.
    #[inline]
    pub fn from_columns(c1: Vector3D, c2: Vector3D, c3: Vector3D) -> Self {
        Self {
            data: [
                [c1.x, c2.x, c3.x],
                [c1.y, c2.y, c3.y],
                [c1.z, c2.z, c3.z],
            ],
        }
    }

    /// Creates a rotation matrix around the X axis by `angle` (radians).
    #[inline]
    pub fn rotation_x(angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            data: [
                [1.0, 0.0, 0.0],
                [0.0, cos_a, -sin_a],
                [0.0, sin_a, cos_a],
            ],
        }
    }

    /// Creates a rotation matrix around the Y axis by `angle` (radians).
    #[inline]
    pub fn rotation_y(angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            data: [
                [cos_a, 0.0, sin_a],
                [0.0, 1.0, 0.0],
                [-sin_a, 0.0, cos_a],
            ],
        }
    }

    /// Creates a rotation matrix around the Z axis by `angle` (radians).
    #[inline]
    pub fn rotation_z(angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            data: [
                [cos_a, -sin_a, 0.0],
                [sin_a, cos_a, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    /// Creates a rotation matrix around the given `axis` by `angle` (radians)
    /// using Rodrigues' rotation formula.
    ///
    /// The `axis` must be a unit vector.
    pub fn rotation(axis: &Vector3D, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = 1.0 - cos_a;
        let (x, y, z) = (axis.x, axis.y, axis.z);

        Self {
            data: [
                [
                    cos_a + x * x * one_minus_cos,
                    x * y * one_minus_cos - z * sin_a,
                    x * z * one_minus_cos + y * sin_a,
                ],
                [
                    y * x * one_minus_cos + z * sin_a,
                    cos_a + y * y * one_minus_cos,
                    y * z * one_minus_cos - x * sin_a,
                ],
                [
                    z * x * one_minus_cos - y * sin_a,
                    z * y * one_minus_cos + x * sin_a,
                    cos_a + z * z * one_minus_cos,
                ],
            ],
        }
    }

    /// Creates a rotation matrix from a quaternion.
    pub fn from_quaternion(q: &crate::quaternion::Quaternion) -> Self {
        let (w, x, y, z) = (q.w, q.x, q.y, q.z);
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);

        Self {
            data: [
                [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)],
                [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)],
                [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy)],
            ],
        }
    }

    /// Returns the transpose of the matrix.
    #[inline]
    pub fn transpose(&self) -> Self {
        let d = self.data;
        Self {
            data: [
                [d[0][0], d[1][0], d[2][0]],
                [d[0][1], d[1][1], d[2][1]],
                [d[0][2], d[1][2], d[2][2]],
            ],
        }
    }

    /// Computes the determinant of the matrix.
    #[inline]
    pub fn determinant(&self) -> f64 {
        let d = self.data;
        d[0][0] * (d[1][1] * d[2][2] - d[1][2] * d[2][1])
            - d[0][1] * (d[1][0] * d[2][2] - d[1][2] * d[2][0])
            + d[0][2] * (d[1][0] * d[2][1] - d[1][1] * d[2][0])
    }

    /// Computes the inverse of the matrix. Returns `None` if the matrix is singular.
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }
        let inv_det = 1.0 / det;
        let d = self.data;

        // Cofactor matrix transposed = adjugate
        let inv = Self {
            data: [
                [
                    (d[1][1] * d[2][2] - d[1][2] * d[2][1]) * inv_det,
                    (d[0][2] * d[2][1] - d[0][1] * d[2][2]) * inv_det,
                    (d[0][1] * d[1][2] - d[0][2] * d[1][1]) * inv_det,
                ],
                [
                    (d[1][2] * d[2][0] - d[1][0] * d[2][2]) * inv_det,
                    (d[0][0] * d[2][2] - d[0][2] * d[2][0]) * inv_det,
                    (d[0][2] * d[1][0] - d[0][0] * d[1][2]) * inv_det,
                ],
                [
                    (d[1][0] * d[2][1] - d[1][1] * d[2][0]) * inv_det,
                    (d[0][1] * d[2][0] - d[0][0] * d[2][1]) * inv_det,
                    (d[0][0] * d[1][1] - d[0][1] * d[1][0]) * inv_det,
                ],
            ],
        };

        Some(inv)
    }

    /// Returns the trace (sum of diagonal elements).
    #[inline]
    pub fn trace(&self) -> f64 {
        self.data[0][0] + self.data[1][1] + self.data[2][2]
    }

    /// Returns `true` if the matrix is approximately the identity within `tol`.
    #[inline]
    pub fn is_identity(&self, tol: f64) -> bool {
        self.approx_eq(&Self::identity(), tol)
    }

    /// Returns `true` if the matrix is approximately orthogonal within `tol`.
    /// An orthogonal matrix satisfies `M * M^T = I`.
    #[inline]
    pub fn is_orthogonal(&self, tol: f64) -> bool {
        let product = *self * self.transpose();
        product.approx_eq(&Self::identity(), tol)
    }

    /// Returns the i-th row as a `Vector3D` (0-indexed).
    ///
    /// # Panics
    /// Panics if `i >= 3`.
    #[inline]
    pub fn get_row(&self, i: usize) -> Vector3D {
        assert!(i < 3, "Row index out of bounds: {i}");
        Vector3D::new(self.data[i][0], self.data[i][1], self.data[i][2])
    }

    /// Returns the j-th column as a `Vector3D` (0-indexed).
    ///
    /// # Panics
    /// Panics if `j >= 3`.
    #[inline]
    pub fn get_column(&self, j: usize) -> Vector3D {
        assert!(j < 3, "Column index out of bounds: {j}");
        Vector3D::new(self.data[0][j], self.data[1][j], self.data[2][j])
    }

    /// Sets the i-th row from a `Vector3D` (0-indexed).
    ///
    /// # Panics
    /// Panics if `i >= 3`.
    #[inline]
    pub fn set_row(&mut self, i: usize, v: Vector3D) {
        assert!(i < 3, "Row index out of bounds: {i}");
        self.data[i] = [v.x, v.y, v.z];
    }

    /// Sets the j-th column from a `Vector3D` (0-indexed).
    ///
    /// # Panics
    /// Panics if `j >= 3`.
    #[inline]
    pub fn set_column(&mut self, j: usize, v: Vector3D) {
        assert!(j < 3, "Column index out of bounds: {j}");
        self.data[0][j] = v.x;
        self.data[1][j] = v.y;
        self.data[2][j] = v.z;
    }

    /// Returns a slice view of the matrix's data (row-major order, 9 elements).
    #[inline]
    pub fn to_slice(&self) -> &[f64] {
        // SAFETY: Matrix3x3 is repr(C) compatible with [f64; 9]
        unsafe { std::slice::from_raw_parts(&self.data[0][0] as *const f64, 9) }
    }

    /// Checks approximate equality component-wise within the given epsilon.
    #[inline]
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        for i in 0..3 {
            for j in 0..3 {
                if (self.data[i][j] - other.data[i][j]).abs() > epsilon {
                    return false;
                }
            }
        }
        true
    }
}

// ---- Arithmetic trait implementations ----

impl Add for Matrix3x3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] = self.data[i][j] + rhs.data[i][j];
            }
        }
        Self { data }
    }
}

impl Sub for Matrix3x3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] = self.data[i][j] - rhs.data[i][j];
            }
        }
        Self { data }
    }
}

/// Matrix-matrix multiplication.
impl Mul<Matrix3x3> for Matrix3x3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Matrix3x3) -> Self {
        let mut data = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] = self.data[i][0] * rhs.data[0][j]
                    + self.data[i][1] * rhs.data[1][j]
                    + self.data[i][2] * rhs.data[2][j];
            }
        }
        Self { data }
    }
}

/// Matrix-vector multiplication (transforms the vector).
impl Mul<Vector3D> for Matrix3x3 {
    type Output = Vector3D;

    #[inline]
    fn mul(self, rhs: Vector3D) -> Vector3D {
        let d = self.data;
        Vector3D::new(
            d[0][0] * rhs.x + d[0][1] * rhs.y + d[0][2] * rhs.z,
            d[1][0] * rhs.x + d[1][1] * rhs.y + d[1][2] * rhs.z,
            d[2][0] * rhs.x + d[2][1] * rhs.y + d[2][2] * rhs.z,
        )
    }
}

impl Mul<f64> for Matrix3x3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        let mut data = self.data;
        for i in 0..3 {
            for j in 0..3 {
                data[i][j] *= rhs;
            }
        }
        Self { data }
    }
}

impl Neg for Matrix3x3 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        self * (-1.0)
    }
}

impl AddAssign for Matrix3x3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..3 {
            for j in 0..3 {
                self.data[i][j] += rhs.data[i][j];
            }
        }
    }
}

impl SubAssign for Matrix3x3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..3 {
            for j in 0..3 {
                self.data[i][j] -= rhs.data[i][j];
            }
        }
    }
}

impl MulAssign<f64> for Matrix3x3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        for i in 0..3 {
            for j in 0..3 {
                self.data[i][j] *= rhs;
            }
        }
    }
}

impl Index<usize> for Matrix3x3 {
    type Output = [f64; 3];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl fmt::Display for Matrix3x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;
        for i in 0..3 {
            write!(f, "  [")?;
            for j in 0..3 {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.data[i][j])?;
            }
            writeln!(f, "]")?;
        }
        write!(f, "]")
    }
}

impl Default for Matrix3x3 {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

// ---- Scalar multiplication on the left side ----

impl Mul<Matrix3x3> for f64 {
    type Output = Matrix3x3;

    #[inline]
    fn mul(self, rhs: Matrix3x3) -> Matrix3x3 {
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
        let m = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        assert_eq!(m.data[0][0], 1.0);
        assert_eq!(m.data[2][2], 9.0);
        assert_eq!(Matrix3x3::default(), Matrix3x3::identity());
    }

    #[test]
    fn test_identity_and_zero() {
        let i = Matrix3x3::identity();
        let z = Matrix3x3::zero();
        assert_eq!(i.data[0][0], 1.0);
        assert_eq!(i.data[1][1], 1.0);
        assert_eq!(i.data[2][2], 1.0);

        for row in 0..3 {
            for col in 0..3 {
                assert_eq!(z.data[row][col], 0.0);
            }
        }
    }

    #[test]
    fn test_from_rows_columns() {
        let r1 = Vector3D::new(1.0, 2.0, 3.0);
        let r2 = Vector3D::new(4.0, 5.0, 6.0);
        let r3 = Vector3D::new(7.0, 8.0, 9.0);
        let m = Matrix3x3::from_rows(r1, r2, r3);
        assert_eq!(m.get_row(0), r1);
        assert_eq!(m.get_row(1), r2);
        assert_eq!(m.get_row(2), r3);

        let c1 = Vector3D::new(1.0, 2.0, 3.0);
        let c2 = Vector3D::new(4.0, 5.0, 6.0);
        let c3 = Vector3D::new(7.0, 8.0, 9.0);
        let m2 = Matrix3x3::from_columns(c1, c2, c3);
        assert_eq!(m2.get_column(0), c1);
        assert_eq!(m2.get_column(1), c2);
        assert_eq!(m2.get_column(2), c3);
    }

    #[test]
    fn test_transpose() {
        let m = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let mt = m.transpose();
        assert_eq!(mt.data[0][1], 4.0);
        assert_eq!(mt.data[1][0], 2.0);
        // Double transpose is identity
        assert_eq!(m.transpose().transpose(), m);
    }

    #[test]
    fn test_determinant() {
        let m = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]]);
        assert_eq!(m.determinant(), -3.0);
    }

    #[test]
    fn test_inverse() {
        let m = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]]);
        let inv = m.inverse().unwrap();
        let prod = m * inv;
        assert!(prod.is_identity(1e-10));

        // Singular matrix
        let singular = Matrix3x3::new([[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [3.0, 6.0, 9.0]]);
        assert!(singular.inverse().is_none());
    }

    #[test]
    fn test_trace() {
        let m = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        assert_eq!(m.trace(), 15.0);
    }

    #[test]
    fn test_is_identity_and_orthogonal() {
        assert!(Matrix3x3::identity().is_identity(1e-10));
        assert!(Matrix3x3::identity().is_orthogonal(1e-10));

        let rot = Matrix3x3::rotation_x(FRAC_PI_2);
        assert!(rot.is_orthogonal(1e-10));
    }

    #[test]
    fn test_get_set_row_column() {
        let mut m = Matrix3x3::zero();
        m.set_row(0, Vector3D::new(1.0, 2.0, 3.0));
        m.set_column(1, Vector3D::new(4.0, 5.0, 6.0));
        assert_eq!(m.get_row(0), Vector3D::new(1.0, 4.0, 3.0));
        assert_eq!(m.get_column(1), Vector3D::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_rotation_matrices() {
        let rx = Matrix3x3::rotation_x(FRAC_PI_2);
        let v = rx * Vector3D::new(0.0, 1.0, 0.0);
        assert!(v.approx_eq(&Vector3D::new(0.0, 0.0, 1.0), 1e-15));

        let ry = Matrix3x3::rotation_y(FRAC_PI_2);
        let v2 = ry * Vector3D::new(1.0, 0.0, 0.0);
        assert!(v2.approx_eq(&Vector3D::new(0.0, 0.0, -1.0), 1e-15));

        let rz = Matrix3x3::rotation_z(FRAC_PI_2);
        let v3 = rz * Vector3D::new(1.0, 0.0, 0.0);
        assert!(v3.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_rotation_axis() {
        let axis = Vector3D::unit_z();
        let rot = Matrix3x3::rotation(&axis, FRAC_PI_2);
        let v = rot * Vector3D::new(1.0, 0.0, 0.0);
        assert!(v.approx_eq(&Vector3D::new(0.0, 1.0, 0.0), 1e-15));
    }

    #[test]
    fn test_matrix_multiplication() {
        let a = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let b = Matrix3x3::new([[9.0, 8.0, 7.0], [6.0, 5.0, 4.0], [3.0, 2.0, 1.0]]);
        let c = a * b;
        assert_eq!(c.data[0][0], 30.0);
        assert_eq!(c.data[1][1], 69.0);
        assert_eq!(c.data[2][2], 90.0);
    }

    #[test]
    fn test_matrix_vector_mul() {
        let m = Matrix3x3::identity();
        let v = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(m * v, v);

        let rot = Matrix3x3::rotation_x(FRAC_PI_2);
        let result = rot * Vector3D::new(0.0, 1.0, 0.0);
        assert!(result.approx_eq(&Vector3D::new(0.0, 0.0, 1.0), 1e-15));
    }

    #[test]
    fn test_scalar_mul() {
        let m = Matrix3x3::identity();
        let m2 = m * 2.0;
        assert_eq!(m2.data[0][0], 2.0);
        assert_eq!(m2.data[1][1], 2.0);

        let m3 = 3.0 * Matrix3x3::identity();
        assert_eq!(m3.data[0][0], 3.0);
    }

    #[test]
    fn test_add_sub_neg() {
        let a = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let b = Matrix3x3::new([[9.0, 8.0, 7.0], [6.0, 5.0, 4.0], [3.0, 2.0, 1.0]]);
        let sum = a + b;
        assert_eq!(sum.data[0][0], 10.0);
        assert_eq!(sum.data[1][1], 10.0);

        let diff = a - b;
        assert_eq!(diff.data[0][0], -8.0);
        assert_eq!(diff.data[1][1], 0.0);

        let neg_a = -a;
        assert_eq!(neg_a.data[0][0], -1.0);
        assert_eq!(neg_a.data[1][1], -5.0);
    }

    #[test]
    fn test_assign_ops() {
        let mut m = Matrix3x3::identity();
        m += Matrix3x3::identity();
        assert_eq!(m.data[0][0], 2.0);
        m -= Matrix3x3::identity();
        assert_eq!(m.data[0][0], 1.0);
        m *= 3.0;
        assert_eq!(m.data[0][0], 3.0);
    }

    #[test]
    fn test_display() {
        let m = Matrix3x3::identity();
        let s = format!("{m}");
        assert!(s.contains("1"));
    }

    #[test]
    fn test_to_slice() {
        let m = Matrix3x3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let sl = m.to_slice();
        assert_eq!(sl.len(), 9);
        assert_eq!(sl[0], 1.0);
        assert_eq!(sl[4], 5.0);
        assert_eq!(sl[8], 9.0);
    }

    #[test]
    fn test_index() {
        let m = Matrix3x3::identity();
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][2], 0.0);
    }

    #[test]
    fn test_approx_eq() {
        let a = Matrix3x3::identity();
        let b = Matrix3x3::new(
            [[1.0 + 1e-10, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0 + 1e-10]],
        );
        assert!(a.approx_eq(&b, 1e-9));
        assert!(!a.approx_eq(&b, 1e-11));
    }
}
