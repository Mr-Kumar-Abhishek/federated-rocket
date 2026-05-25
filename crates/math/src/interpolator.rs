/// Interpolation method selection.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationMethod {
    /// Simple linear interpolation between nearest points.
    Linear,
    /// Natural cubic spline interpolation (second derivatives zero at boundaries).
    CubicSpline,
    /// Akima spline interpolation.
    Akima,
    /// Polynomial interpolation of the given degree.
    Polynomial(u8),
}

/// An interpolator for 1D data points `(x, y)`.
///
/// Supports multiple interpolation methods: Linear, CubicSpline, Akima, and Polynomial.
#[derive(Debug, Clone)]
pub struct Interpolator {
    points: Vec<(f64, f64)>,
    method: InterpolationMethod,
    // Pre-computed coefficients for cubic spline or polynomial
    coeffs: Vec<f64>,
    // Second derivatives for cubic spline (natural spline)
    second_derivs: Vec<f64>,
    prepared: bool,
}

impl Interpolator {
    /// Creates a new `Interpolator` with the specified method and no points.
    #[inline]
    pub fn new(method: InterpolationMethod) -> Self {
        Self {
            points: Vec::new(),
            coeffs: Vec::new(),
            second_derivs: Vec::new(),
            prepared: false,
            method,
        }
    }

    /// Creates a new `Interpolator` with the given points and method.
    pub fn with_points(points: Vec<(f64, f64)>, method: InterpolationMethod) -> Self {
        let mut interp = Self::new(method);
        interp.points = points;
        interp
    }

    /// Adds a single data point to the interpolator.
    ///
    /// Resets the prepared state; you must call [`prepare`](Self::prepare) again before interpolating
    /// with methods that require pre-computation (CubicSpline, Polynomial).
    #[inline]
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
        self.prepared = false;
    }

    /// Clears all data points and resets prepared state.
    #[inline]
    pub fn clear(&mut self) {
        self.points.clear();
        self.coeffs.clear();
        self.second_derivs.clear();
        self.prepared = false;
    }

    /// Returns the number of data points.
    #[inline]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns `true` if the interpolator has no data points.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Pre-computes coefficients for the selected interpolation method.
    ///
    /// Must be called after adding points for CubicSpline and Polynomial methods.
    /// Automatically called by [`interpolate`](Self::interpolate) and [`derivative`](Self::derivative)
    /// if not already prepared.
    pub fn prepare(&mut self) {
        self.points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let n = self.points.len();

        match self.method {
            InterpolationMethod::Linear => {
                // No pre-computation needed
                self.prepared = true;
            }
            InterpolationMethod::CubicSpline => {
                self.compute_natural_cubic_spline(n);
                self.prepared = true;
            }
            InterpolationMethod::Akima => {
                self.compute_akima_coefficients(n);
                self.prepared = true;
            }
            InterpolationMethod::Polynomial(_degree) => {
                self.compute_polynomial_coefficients(n);
                self.prepared = true;
            }
        }
    }

    /// Interpolates the value at `x`.
    ///
    /// Returns `None` if there are fewer than 2 points or if `x` is outside the data range.
    pub fn interpolate(&mut self, x: f64) -> Option<f64> {
        if !self.prepared {
            self.prepare();
        }
        if self.points.len() < 2 {
            return None;
        }

        // Clamp to the data range
        let x_min = self.points.first()?.0;
        let x_max = self.points.last()?.0;
        if x < x_min || x > x_max {
            return None;
        }

        match self.method {
            InterpolationMethod::Linear => self.interpolate_linear(x),
            InterpolationMethod::CubicSpline => self.interpolate_cubic_spline(x),
            InterpolationMethod::Akima => self.interpolate_akima(x),
            InterpolationMethod::Polynomial(_) => self.interpolate_polynomial(x),
        }
    }

    /// Approximates the derivative at `x`.
    ///
    /// Returns `None` if there are fewer than 2 points or if `x` is outside the data range.
    pub fn derivative(&mut self, x: f64) -> Option<f64> {
        if !self.prepared {
            self.prepare();
        }
        if self.points.len() < 2 {
            return None;
        }

        let x_min = self.points.first()?.0;
        let x_max = self.points.last()?.0;
        if x < x_min || x > x_max {
            return None;
        }

        match self.method {
            InterpolationMethod::Linear => {
                // Derivative of linear interpolation is the slope of the segment
                let (i, _) = self.find_segment(x)?;
                let (x1, y1) = self.points[i];
                let (x2, y2) = self.points[i + 1];
                Some((y2 - y1) / (x2 - x1))
            }
            InterpolationMethod::CubicSpline => {
                // Derivative of cubic spline: d/dx of S_i(x) = b_i + 2*c_i*(x-x_i) + 3*d_i*(x-x_i)^2
                // where S_i(x) = y_i + b_i*(x-x_i) + c_i*(x-x_i)^2 + d_i*(x-x_i)^3
                let (i, _) = self.find_segment(x)?;
                let (x1, _) = self.points[i];
                let dx = x - x1;

                // Coefficients stored as: [b0, c0, d0, b1, c1, d1, ...]
                let idx = i * 3;
                if idx + 2 < self.coeffs.len() {
                    let b = self.coeffs[idx];
                    let c = self.coeffs[idx + 1];
                    let d = self.coeffs[idx + 2];
                    Some(b + 2.0 * c * dx + 3.0 * d * dx * dx)
                } else {
                    None
                }
            }
            InterpolationMethod::Akima => {
                // Use central difference on the Akima interpolant
                let h = 1e-6;
                let fp = self.interpolate_akima(x + h)?;
                let fm = self.interpolate_akima(x - h)?;
                Some((fp - fm) / (2.0 * h))
            }
            InterpolationMethod::Polynomial(_) => {
                // Use central difference on the polynomial interpolant
                let h = 1e-6;
                let fp = self.interpolate_polynomial(x + h)?;
                let fm = self.interpolate_polynomial(x - h)?;
                Some((fp - fm) / (2.0 * h))
            }
        }
    }

    // ---------------------------------------------------------------
    // Linear interpolation
    // ---------------------------------------------------------------

    fn interpolate_linear(&self, x: f64) -> Option<f64> {
        let (i, _) = self.find_segment(x)?;
        let (x1, y1) = self.points[i];
        let (x2, y2) = self.points[i + 1];
        let t = (x - x1) / (x2 - x1);
        Some(y1 + t * (y2 - y1))
    }

    // ---------------------------------------------------------------
    // Natural cubic spline
    // ---------------------------------------------------------------

    fn compute_natural_cubic_spline(&mut self, n: usize) {
        if n < 2 {
            return;
        }

        let n = self.points.len();
        let mut h = vec![0.0; n - 1];
        let mut alpha = vec![0.0; n - 1];

        for i in 0..n - 1 {
            h[i] = self.points[i + 1].0 - self.points[i].0;
        }

        for i in 1..n - 1 {
            // RHS = 6 * (delta_{i+1} - delta_i) where delta_i = (y_i - y_{i-1}) / h_{i-1}
            alpha[i] = 6.0 * (
                (self.points[i + 1].1 - self.points[i].1) / h[i]
                - (self.points[i].1 - self.points[i - 1].1) / h[i - 1]
            );
        }

        // Tridiagonal system solve for second derivatives M_i (natural spline: M_0 = M_{n-1} = 0)
        // h_{i-1}*M_{i-1} + 2*(h_{i-1}+h_i)*M_i + h_i*M_{i+1} = alpha[i]
        let mut second_deriv = vec![0.0; n];
        let mut l = vec![1.0; n];
        let mut mu = vec![0.0; n];
        let mut z = vec![0.0; n];

        for i in 1..n - 1 {
            l[i] = 2.0 * (self.points[i + 1].0 - self.points[i - 1].0) - h[i - 1] * mu[i - 1];
            mu[i] = h[i] / l[i];
            z[i] = (alpha[i] - h[i - 1] * z[i - 1]) / l[i];
        }

        // Back substitution (M_{n-1} = 0 by natural boundary condition)
        for j in (0..n - 1).rev() {
            second_deriv[j] = z[j] - mu[j] * second_deriv[j + 1];
        }

        self.second_derivs = second_deriv;

        // Compute cubic coefficients for each segment
        // S_i(x) = y_i + b_i*(x-x_i) + (M_i/2)*(x-x_i)^2 + ((M_{i+1}-M_i)/(6*h_i))*(x-x_i)^3
        self.coeffs = Vec::with_capacity((n - 1) * 3);
        for i in 0..n - 1 {
            let sd_i = self.second_derivs[i];
            let sd_ip1 = self.second_derivs[i + 1];
            let b = (self.points[i + 1].1 - self.points[i].1) / h[i]
                - h[i] * (sd_ip1 + 2.0 * sd_i) / 6.0;
            let c = sd_i / 2.0;
            let d = (sd_ip1 - sd_i) / (6.0 * h[i]);
            self.coeffs.push(b);
            self.coeffs.push(c);
            self.coeffs.push(d);
        }
    }

    fn interpolate_cubic_spline(&self, x: f64) -> Option<f64> {
        let (i, _) = self.find_segment(x)?;
        let (x1, y1) = self.points[i];
        let dx = x - x1;

        let idx = i * 3;
        if idx + 2 < self.coeffs.len() {
            let b = self.coeffs[idx];
            let c = self.coeffs[idx + 1];
            let d = self.coeffs[idx + 2];
            Some(y1 + b * dx + c * dx * dx + d * dx * dx * dx)
        } else {
            None
        }
    }

    // ---------------------------------------------------------------
    // Akima spline
    // ---------------------------------------------------------------

    fn compute_akima_coefficients(&mut self, n: usize) {
        if n < 2 {
            return;
        }

        // Compute slopes
        let m_len = n - 1;
        let mut m = vec![0.0; m_len];
        for i in 0..m_len {
            m[i] = (self.points[i + 1].1 - self.points[i].1)
                / (self.points[i + 1].0 - self.points[i].0);
        }

        // Akima's method requires 4 extra slopes at each end
        let mut slopes = Vec::with_capacity(n + 8);
        // Estimate slopes before the first point
        slopes.push(3.0 * m[0] - 2.0 * m[1]);   // m_{-4}
        slopes.push(2.0 * m[0] - m[1]);          // m_{-3}
        slopes.push(m[0]);                       // m_{-2}
        slopes.push(m[0]);                       // m_{-1}
        // Actual slopes
        for i in 0..n - 1 {
            slopes.push(m[i]);
        }
        // Estimate slopes after the last point
        slopes.push(m[n - 2]);                   // m_{n-1}
        slopes.push(m[n - 2]);                   // m_{n}
        slopes.push(2.0 * m[n - 2] - m[n - 3]); // m_{n+1}
        slopes.push(3.0 * m[n - 2] - 2.0 * m[n - 3]); // m_{n+2}

        // Compute the derivative at each data point using Akima's formula
        let mut t = vec![0.0; n];
        for i in 0..n {
            let m1 = slopes[i + 4];
            let m2 = slopes[i + 5];
            let m3 = slopes[i + 6];
            let _m4 = slopes[i + 7];

            let w1 = (m3 - m2).abs();
            let w2 = (m1 - m2).abs();

            if w1 + w2 > 0.0 {
                t[i] = (w1 * m2 + w2 * m3) / (w1 + w2);
            } else {
                t[i] = (m2 + m3) / 2.0;
            }
        }

        // Compute polynomial coefficients for each segment
        // p_i(x) = p0 + p1*(x-x_i) + p2*(x-x_i)^2 + p3*(x-x_i)^3
        self.coeffs = Vec::with_capacity((n - 1) * 4);
        for i in 0..n - 1 {
            let x1 = self.points[i].0;
            let x2 = self.points[i + 1].0;
            let y1 = self.points[i].1;
            let y2 = self.points[i + 1].1;
            let dx = x2 - x1;

            let p0 = y1;
            let p1 = t[i];
            let p2 = (3.0 * (y2 - y1) / dx - 2.0 * t[i] - t[i + 1]) / dx;
            let p3 = (t[i] + t[i + 1] - 2.0 * (y2 - y1) / dx) / (dx * dx);

            self.coeffs.push(p0);
            self.coeffs.push(p1);
            self.coeffs.push(p2);
            self.coeffs.push(p3);
        }
    }

    fn interpolate_akima(&self, x: f64) -> Option<f64> {
        let (i, _) = self.find_segment(x)?;
        let x1 = self.points[i].0;
        let dx = x - x1;

        let idx = i * 4;
        if idx + 3 < self.coeffs.len() {
            let p0 = self.coeffs[idx];
            let p1 = self.coeffs[idx + 1];
            let p2 = self.coeffs[idx + 2];
            let p3 = self.coeffs[idx + 3];
            Some(p0 + p1 * dx + p2 * dx * dx + p3 * dx * dx * dx)
        } else {
            None
        }
    }

    // ---------------------------------------------------------------
    // Polynomial interpolation
    // ---------------------------------------------------------------

    fn compute_polynomial_coefficients(&mut self, n: usize) {
        if n < 2 {
            return;
        }

        let degree = match self.method {
            InterpolationMethod::Polynomial(d) => d as usize,
            _ => return,
        };

        // Use Lagrange polynomial coefficients
        // We'll use the barycentric form for evaluation instead, which is more numerically stable
        self.coeffs = vec![0.0; degree + 1];
        // Store the actual points used for barycentric weights
        // We'll compute the weighted barycentric form
        let n_used = degree.min(n);
        // We select the n_used+1 points closest to the center of the range
        // For simplicity, use the first n_used+1 points

        // Actually, for polynomial interpolation, we'll use Neville's algorithm
        // during evaluation. The coeffs vector stores the barycentric weights.
        let m = n_used + 1;
        let mut weights = vec![1.0; m];
        for i in 0..m {
            for j in 0..m {
                if i != j {
                    weights[i] /= self.points[i].0 - self.points[j].0;
                }
            }
        }
        self.coeffs = weights;
    }

    fn interpolate_polynomial(&self, x: f64) -> Option<f64> {
        let degree = match self.method {
            InterpolationMethod::Polynomial(d) => d as usize,
            _ => return None,
        };

        let n = self.points.len();
        let m = (degree + 1).min(n);

        if m < 2 {
            return None;
        }

        // Use Neville's algorithm for evaluation
        // First, find the m points closest to x
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            let da = (self.points[*a].0 - x).abs();
            let db = (self.points[*b].0 - x).abs();
            da.partial_cmp(&db).unwrap()
        });
        indices.truncate(m);
        indices.sort(); // restore original order

        // Neville's algorithm
        let mut p = vec![0.0; m];
        for i in 0..m {
            p[i] = self.points[indices[i]].1;
        }

        for k in 1..m {
            for i in 0..m - k {
                let xi = self.points[indices[i]].0;
                let xik = self.points[indices[i + k]].0;
                p[i] = ((x - xik) * p[i] + (xi - x) * p[i + 1]) / (xi - xik);
            }
        }

        Some(p[0])
    }

    // ---------------------------------------------------------------
    // Utility: find the segment containing x
    // ---------------------------------------------------------------

    /// Finds the index `i` such that `points[i].0 <= x <= points[i+1].0`.
    /// Returns `(i, t)` where `t` is the fraction along the segment.
    #[inline]
    fn find_segment(&self, x: f64) -> Option<(usize, f64)> {
        let n = self.points.len();
        if n < 2 {
            return None;
        }

        let x_min = self.points[0].0;
        let x_max = self.points[n - 1].0;

        if x < x_min || x > x_max {
            return None;
        }

        // Binary search
        let mut lo = 0;
        let mut hi = n - 1;
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if x >= self.points[mid].0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        Some((lo, 0.0))
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let points = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let mut interp = Interpolator::with_points(points, InterpolationMethod::Linear);

        assert_eq!(interp.interpolate(0.0), Some(0.0));
        assert_eq!(interp.interpolate(1.0), Some(1.0));
        assert_eq!(interp.interpolate(2.0), Some(0.0));

        // Midpoints
        assert_eq!(interp.interpolate(0.5), Some(0.5));
        assert_eq!(interp.interpolate(1.5), Some(0.5));

        // Out of range
        assert_eq!(interp.interpolate(-0.1), None);
        assert_eq!(interp.interpolate(2.1), None);
    }

    #[test]
    fn test_linear_derivative() {
        let points = vec![(0.0, 0.0), (2.0, 4.0), (4.0, 8.0)];
        let mut interp = Interpolator::with_points(points, InterpolationMethod::Linear);

        // Slope is always 2.0
        assert_eq!(interp.derivative(1.0), Some(2.0));
        assert_eq!(interp.derivative(3.0), Some(2.0));
    }

    #[test]
    fn test_cubic_spline_interpolation() {
        // Test with a known function: f(x) = x^2 on [0, 2]
        let points = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 4.0)];
        let mut interp = Interpolator::with_points(points, InterpolationMethod::CubicSpline);

        // At the data points, should match exactly (cubic spline passes through knots)
        assert!((interp.interpolate(0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((interp.interpolate(1.0).unwrap() - 1.0).abs() < 1e-10);
        assert!((interp.interpolate(2.0).unwrap() - 4.0).abs() < 1e-10);

        // At x=0.5, natural cubic spline interpolation of x^2 with 3 points
        let val = interp.interpolate(0.5).unwrap();
        assert!((val - 0.25).abs() < 0.5, "cubic spline at 0.5: {val}");
    }

    #[test]
    fn test_cubic_spline_derivative() {
        // f(x) = x^3 has derivative 3x^2
        let points = vec![(0.0, 0.0), (0.5, 0.125), (1.0, 1.0), (1.5, 3.375), (2.0, 8.0)];
        let mut interp = Interpolator::with_points(points, InterpolationMethod::CubicSpline);

        let deriv = interp.derivative(1.0).unwrap();
        let expected = 3.0; // 3 * 1^2 = 3
        assert!(
            (deriv - expected).abs() < 2.0,
            "cubic spline derivative at 1.0: {deriv}, expected {expected}"
        );
    }

    #[test]
    fn test_akima_interpolation() {
        let points = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 4.0), (3.0, 9.0), (4.0, 16.0)];
        let mut interp = Interpolator::with_points(points, InterpolationMethod::Akima);

        // At data points
        assert!((interp.interpolate(0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((interp.interpolate(2.0).unwrap() - 4.0).abs() < 1e-10);
        assert!((interp.interpolate(4.0).unwrap() - 16.0).abs() < 1e-10);

        // At x=0.5, should be reasonable
        let val = interp.interpolate(0.5).unwrap();
        assert!((val - 0.25).abs() < 0.5, "akima at 0.5: {val}");
    }

    #[test]
    fn test_polynomial_interpolation() {
        // f(x) = 2x + 1 is linear, degree 1 should recover it exactly
        let points = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 5.0)];
        let mut interp = Interpolator::with_points(points, InterpolationMethod::Polynomial(1));

        // Since the data is actually linear, degree 1 should interpolate exactly
        let v1 = interp.interpolate(0.5).unwrap();
        assert!((v1 - 2.0).abs() < 1e-10, "polynomial at 0.5: {v1}");
    }

    #[test]
    fn test_interpolate_known_function() {
        // Test linear interpolation of sin(x) on [0, pi/2]
        let n = 10;
        let points: Vec<(f64, f64)> = (0..=n)
            .map(|i| {
                let x = std::f64::consts::FRAC_PI_2 * i as f64 / n as f64;
                (x, x.sin())
            })
            .collect();

        let mut interp = Interpolator::with_points(points, InterpolationMethod::Linear);

        let x_test: f64 = 1.0;
        let expected = x_test.sin();
        let actual = interp.interpolate(x_test).unwrap();
        let error = (actual - expected).abs();
        // With only 10 points on [0, pi/2], linear interpolation error at x=1
        // should be moderate
        assert!(error < 0.05, "linear interpolation of sin(1): {actual}, expected {expected}");
    }

    #[test]
    fn test_cubic_spline_known_function() {
        // Test cubic spline interpolation of sin(x) on [0, pi]
        let n = 8;
        let points: Vec<(f64, f64)> = (0..=n)
            .map(|i| {
                let x = std::f64::consts::PI * i as f64 / n as f64;
                (x, x.sin())
            })
            .collect();

        let mut interp = Interpolator::with_points(points.clone(), InterpolationMethod::CubicSpline);

        let x_test: f64 = 1.5;
        let expected = x_test.sin();
        let actual = interp.interpolate(x_test).unwrap();
        let error = (actual - expected).abs();
        assert!(error < 0.01, "cubic spline of sin(1.5): {actual}, expected {expected}, error {error}");
    }

    #[test]
    fn test_add_point_and_clear() {
        let mut interp = Interpolator::new(InterpolationMethod::Linear);
        assert!(interp.is_empty());

        interp.add_point(0.0, 0.0);
        interp.add_point(1.0, 1.0);
        assert_eq!(interp.len(), 2);
        assert_eq!(interp.interpolate(0.5), Some(0.5));

        interp.clear();
        assert!(interp.is_empty());
        assert_eq!(interp.interpolate(0.5), None);
    }

    #[test]
    fn test_insufficient_points() {
        let mut interp = Interpolator::new(InterpolationMethod::Linear);
        interp.add_point(0.0, 0.0);
        assert_eq!(interp.interpolate(0.5), None);
    }

    #[test]
    fn test_linear_vs_cubic_on_linear_data() {
        // Both methods should recover linear data exactly
        let points = vec![(0.0, 2.0), (1.0, 4.0), (2.0, 6.0)];

        let mut linear = Interpolator::with_points(points.clone(), InterpolationMethod::Linear);
        let mut cubic = Interpolator::with_points(points, InterpolationMethod::CubicSpline);

        assert_eq!(linear.interpolate(0.5), Some(3.0));
        assert!((cubic.interpolate(0.5).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_prepare_recalculates() {
        let mut interp = Interpolator::new(InterpolationMethod::CubicSpline);
        interp.add_point(0.0, 0.0);
        interp.add_point(1.0, 1.0);
        interp.add_point(2.0, 0.0);

        // First interpolation should work
        let _v1 = interp.interpolate(0.5);

        // Add a point and interpolate again
        interp.add_point(3.0, 1.0);
        let v2 = interp.interpolate(2.5);
        assert!(v2.is_some());
    }
}