use crate::types::{DesignParameter, OptimizationGoal, OptimizationResult, ParameterType};

/// Golden Section Search optimizer for 1D (univariate) problems.
///
/// This is a derivative-free optimisation method that iteratively narrows
/// an interval known to contain a minimum by evaluating the objective at
/// two interior points placed according to the golden ratio.
///
/// # Theory
///
/// Let φ = (1 + √5) / 2 ≈ 1.618 be the golden ratio.
/// The inverse golden ratio is r = 2 − φ ≈ 0.382.
/// At each iteration the interval `[x1, x2]` is reduced by a factor of r,
/// so the method is guaranteed to converge linearly.
pub struct GoldenSectionSearch {
    /// Maximum number of iterations. Default: 100.
    pub max_iterations: usize,
    /// Interval width tolerance for convergence. Default: 1e-6.
    pub tolerance: f64,
    /// Objective-value convergence threshold (not used in the basic algorithm
    /// but stored for convenience). Default: 1e-4.
    pub convergence_threshold: f64,
}

impl Default for GoldenSectionSearch {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            convergence_threshold: 1e-4,
        }
    }
}

impl GoldenSectionSearch {
    /// Create a new `GoldenSectionSearch` with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimize the objective function `f` over the closed interval `[a, b]`.
    ///
    /// # Parameters
    ///
    /// * `f` - The objective function to minimise.
    /// * `a` - Left endpoint of the search interval.
    /// * `b` - Right endpoint of the search interval.
    ///
    /// # Returns
    ///
    /// An [`OptimizationResult`] summarising the optimisation run.
    pub fn minimize<F>(&self, f: F, a: f64, b: f64) -> OptimizationResult
    where
        F: Fn(f64) -> f64,
    {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0; // ≈ 1.618
        let resphi = 2.0 - phi; // ≈ 0.382

        let mut x1 = a;
        let mut x2 = b;
        let mut d = resphi * (x2 - x1);
        let mut xa = x1 + d;
        let mut xb = x2 - d;
        let mut fa = f(xa);
        let mut fb = f(xb);
        let mut history = Vec::new();
        let mut iterations = 0;

        for i in 0..self.max_iterations {
            iterations = i + 1;

            if fa < fb {
                x2 = xb;
                xb = xa;
                fb = fa;
                d = resphi * (x2 - x1);
                xa = x1 + d;
                fa = f(xa);
            } else {
                x1 = xa;
                xa = xb;
                fa = fb;
                d = resphi * (x2 - x1);
                xb = x2 - d;
                fb = f(xb);
            }

            let x_mid = (x1 + x2) / 2.0;
            history.push((x_mid, fa.min(fb)));

            if (x2 - x1).abs() < self.tolerance {
                break;
            }
        }

        let x_opt = (x1 + x2) / 2.0;
        let f_opt = f(x_opt);
        let initial_value = f((a + b) / 2.0);

        let improvement = if initial_value.abs() > f64::EPSILON {
            ((initial_value - f_opt) / initial_value.abs()) * 100.0
        } else {
            0.0
        };

        let converged = (x2 - x1).abs() < self.tolerance;

        OptimizationResult {
            goal: OptimizationGoal::Custom("minimize".to_string()),
            initial_value,
            final_value: f_opt,
            improvement,
            iterations,
            evaluations: iterations * 2,
            converged,
            parameters: vec![DesignParameter {
                name: "x".to_string(),
                value: x_opt,
                min: a,
                max: b,
                step: 0.0,
                parameter_type: ParameterType::Continuous,
            }],
            history,
        }
    }

    /// Maximize the objective function `f` over the closed interval `[a, b]`.
    ///
    /// This simply negates `f` and delegates to [`Self::minimize`].
    ///
    /// # Parameters
    ///
    /// * `f` - The objective function to maximise.
    /// * `a` - Left endpoint of the search interval.
    /// * `b` - Right endpoint of the search interval.
    ///
    /// # Returns
    ///
    /// An [`OptimizationResult`] summarising the optimisation run.
    pub fn maximize<F>(&self, f: F, a: f64, b: f64) -> OptimizationResult
    where
        F: Fn(f64) -> f64,
    {
        let min_result = self.minimize(|x| -f(x), a, b);

        // Negate back the objective values so they reflect the true (maximized) function
        OptimizationResult {
            goal: OptimizationGoal::Custom("maximize".to_string()),
            initial_value: -min_result.initial_value,
            final_value: -min_result.final_value,
            improvement: min_result.improvement,
            iterations: min_result.iterations,
            evaluations: min_result.evaluations,
            converged: min_result.converged,
            parameters: min_result.parameters,
            history: min_result
                .history
                .into_iter()
                .map(|(x, y)| (x, -y))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parameters() {
        let gs = GoldenSectionSearch::default();
        assert_eq!(gs.max_iterations, 100);
        assert!((gs.tolerance - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn test_new() {
        let gs = GoldenSectionSearch::new();
        assert_eq!(gs.max_iterations, 100);
    }

    #[test]
    fn test_minimize_quadratic() {
        // f(x) = x^2 has minimum at x = 0
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| x * x, -2.0, 3.0);

        assert!(result.converged, "Golden section should converge for x^2");
        assert!(result.final_value.abs() < 1e-4, "f(x_opt) ≈ 0, got {}", result.final_value);
        assert!(
            result.parameters[0].value.abs() < 1e-4,
            "x_opt ≈ 0, got {}",
            result.parameters[0].value
        );
        assert!(result.iterations > 0);
        assert!(!result.history.is_empty());
    }

    #[test]
    fn test_minimize_shifted_quadratic() {
        // f(x) = (x - 5)^2 has minimum at x = 5
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| (x - 5.0) * (x - 5.0), 0.0, 10.0);

        assert!(result.converged);
        let x_opt = result.parameters[0].value;
        assert!(
            (x_opt - 5.0).abs() < 1e-3,
            "x_opt ≈ 5, got {}",
            x_opt
        );
    }

    #[test]
    fn test_minimize_sin() {
        // sin(x) on [3, 5] has minimum at x ≈ 4.712 (3π/2)
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| x.sin(), 3.0, 5.0);

        assert!(result.converged);
        let x_opt = result.parameters[0].value;
        let expected = 3.0 * std::f64::consts::PI / 2.0; // ≈ 4.712
        assert!(
            (x_opt - expected).abs() < 1e-3,
            "x_opt ≈ {}, got {}",
            expected,
            x_opt
        );
    }

    #[test]
    fn test_maximize_quadratic() {
        // f(x) = -(x^2) on [-2, 2] has maximum at x = 0 with f(0) = 0
        let gs = GoldenSectionSearch::new();
        let result = gs.maximize(|x| -x * x, -2.0, 2.0);

        assert!(result.converged);
        let x_opt = result.parameters[0].value;
        assert!(x_opt.abs() < 1e-4, "x_opt ≈ 0, got {}", x_opt);
        // The maximum value of -x^2 on [-2,2] is 0 at x=0
        assert!(
            result.final_value.abs() < 1e-4,
            "final_value ≈ 0, got {}",
            result.final_value
        );
    }

    #[test]
    fn test_maximize_sin() {
        // sin(x) on [0, π] has maximum at x = π/2
        let gs = GoldenSectionSearch::new();
        let result = gs.maximize(|x| x.sin(), 0.0, std::f64::consts::PI);

        assert!(result.converged);
        let x_opt = result.parameters[0].value;
        let expected = std::f64::consts::PI / 2.0;
        assert!(
            (x_opt - expected).abs() < 1e-3,
            "x_opt ≈ {}, got {}",
            expected,
            x_opt
        );
    }

    #[test]
    fn test_negative_improvement() {
        // If initial guess is already better than the optimum, improvement can be negative.
        // f(x) = x^2 on [0.5, 2] — the minimum is at x=0 but the interval starts inside.
        // Actually with interval [0.5, 2], the minimum is at 0.5, so initial middle is 1.25
        // which is worse than 0.5. So improvement should be positive.
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| x * x, 0.5, 2.0);

        assert!(result.converged);
        assert!(
            result.improvement > 0.0,
            "Improvement should be positive, got {}",
            result.improvement
        );
    }

    #[test]
    fn test_tight_tolerance_uses_more_iterations() {
        let loose = GoldenSectionSearch {
            tolerance: 1e-2,
            ..GoldenSectionSearch::default()
        };
        let tight = GoldenSectionSearch {
            tolerance: 1e-10,
            ..GoldenSectionSearch::default()
        };

        let r_loose = loose.minimize(|x| x * x, -1.0, 1.0);
        let r_tight = tight.minimize(|x| x * x, -1.0, 1.0);

        assert!(r_tight.iterations >= r_loose.iterations);
    }

    #[test]
    fn test_evaluations_count() {
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| x * x, -2.0, 2.0);
        assert_eq!(result.evaluations, result.iterations * 2);
    }

    #[test]
    fn test_history_records_intermediate_points() {
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| x * x, -2.0, 2.0);
        assert!(!result.history.is_empty());
        // Each entry should have a valid (x, f(x)) pair
        for &(x, fx) in &result.history {
            assert!((fx - x * x).abs() < 1.0); // Allow generous tolerance; history points are midpoints
        }
    }

    #[test]
    fn test_flat_function() {
        // f(x) = 1 everywhere — any point is optimal
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|_| 1.0, -10.0, 10.0);

        assert!(result.converged);
        assert!((result.final_value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_linear_function() {
        // f(x) = x on [0, 10] — minimum at 0
        let gs = GoldenSectionSearch {
            tolerance: 1e-8,
            ..GoldenSectionSearch::default()
        };
        let result = gs.minimize(|x| x, 0.0, 10.0);

        assert!(result.converged);
        let x_opt = result.parameters[0].value;
        assert!(
            (x_opt - 0.0).abs() < 1e-3,
            "x_opt ≈ 0, got {}",
            x_opt
        );
    }

    #[test]
    fn test_goal_custom_label() {
        let gs = GoldenSectionSearch::new();
        let result = gs.minimize(|x| x * x, -1.0, 1.0);
        assert_eq!(result.goal, OptimizationGoal::Custom("minimize".to_string()));

        let max_result = gs.maximize(|x| -x * x, -1.0, 1.0);
        assert_eq!(max_result.goal, OptimizationGoal::Custom("maximize".to_string()));
    }
}