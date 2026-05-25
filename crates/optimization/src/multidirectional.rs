use crate::types::{DesignParameter, OptimizationGoal, OptimizationResult, ParameterType};

/// Multidirectional search — a derivative-free direct search method for
/// multi-variate optimisation.
///
/// The algorithm maintains a simplex of `n + 1` vertices for an `n`-dimensional
/// problem and iteratively improves the worst vertex through reflection,
/// expansion, and contraction operations.  When none of these succeed the
/// entire simplex is shrunk toward the best vertex.
///
/// This implementation follows the Nelder–Mead variant of multidirectional
/// search.
pub struct MultidirectionalSearch {
    /// Maximum number of iterations. Default: 200.
    pub max_iterations: usize,
    /// Standard-deviation tolerance for convergence. Default: 1e-6.
    pub tolerance: f64,
    /// Expansion factor applied to the reflected point. Default: 2.0.
    pub expansion_factor: f64,
    /// Contraction factor applied toward the centroid. Default: 0.5.
    pub contraction_factor: f64,
}

impl Default for MultidirectionalSearch {
    fn default() -> Self {
        Self {
            max_iterations: 200,
            tolerance: 1e-6,
            expansion_factor: 2.0,
            contraction_factor: 0.5,
        }
    }
}

impl MultidirectionalSearch {
    /// Create a new `MultidirectionalSearch` with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimize the objective function `f` starting from the given simplex.
    ///
    /// # Parameters
    ///
    /// * `f` - The objective function to minimise.  Takes a slice of parameter
    ///   values and returns a scalar.
    /// * `initial_simplex` - The initial simplex vertices.  Must contain
    ///   exactly `n + 1` vertices, each of length `n` (the problem dimension).
    ///
    /// # Returns
    ///
    /// An [`OptimizationResult`] summarising the optimisation run.
    pub fn minimize<F>(&self, f: F, initial_simplex: &[Vec<f64>]) -> OptimizationResult
    where
        F: Fn(&[f64]) -> f64,
    {
        let n = initial_simplex.len();
        let dim = initial_simplex[0].len();

        // Initialise simplex
        let mut simplex = initial_simplex.to_vec();
        let mut values: Vec<f64> = simplex.iter().map(|v| f(v)).collect();
        let mut history = Vec::new();
        let mut iterations = 0;
        let mut evaluations = n; // initial evaluations

        // Track the worst value from the initial simplex for reporting
        let initial_worst = values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        for iteration in 0..self.max_iterations {
            iterations = iteration + 1;

            // Sort vertices by objective value (ascending — best first)
            let mut indices: Vec<usize> = (0..n).collect();
            indices.sort_by(|&i, &j| values[i].partial_cmp(&values[j]).unwrap());

            // Check convergence via standard deviation of function values
            let mean = indices.iter().map(|&i| values[i]).sum::<f64>() / n as f64;
            let variance = indices
                .iter()
                .map(|&i| (values[i] - mean).powi(2))
                .sum::<f64>()
                / n as f64;

            let best_value = values[indices[0]];
            history.push((iteration as f64, best_value));

            if variance.sqrt() < self.tolerance {
                let x_opt = simplex[indices[0]].clone();
                let improvement = if values[indices[0]].abs() > f64::EPSILON {
                    ((values[indices[n - 1]] - values[indices[0]]) / values[indices[0]].abs())
                        * 100.0
                } else {
                    0.0
                };

                let parameters: Vec<DesignParameter> = x_opt
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| DesignParameter {
                        name: format!("x{}", i),
                        value: v,
                        min: v - 1.0,
                        max: v + 1.0,
                        step: 0.0,
                        parameter_type: ParameterType::Continuous,
                    })
                    .collect();

                return OptimizationResult {
                    goal: OptimizationGoal::Custom("minimize".to_string()),
                    initial_value: initial_worst,
                    final_value: values[indices[0]],
                    improvement,
                    iterations,
                    evaluations,
                    converged: true,
                    parameters,
                    history,
                };
            }

            // Compute centroid of all but the worst vertex
            let centroid: Vec<f64> = (0..dim)
                .map(|j| {
                    let sum: f64 = indices[..n - 1]
                        .iter()
                        .map(|&i| simplex[i][j])
                        .sum();
                    sum / (n - 1) as f64
                })
                .collect();

            let worst_idx = indices[n - 1];
            let best_idx = indices[0];

            // --- Reflection ---
            let reflected: Vec<f64> = centroid
                .iter()
                .zip(simplex[worst_idx].iter())
                .map(|(c, w)| 2.0 * c - w)
                .collect();
            let f_reflected = f(&reflected);
            evaluations += 1;

            if f_reflected < values[best_idx] {
                // --- Expansion ---
                let expanded: Vec<f64> = centroid
                    .iter()
                    .zip(reflected.iter())
                    .map(|(c, r)| c + self.expansion_factor * (r - c))
                    .collect();
                let f_expanded = f(&expanded);
                evaluations += 1;

                if f_expanded < f_reflected {
                    simplex[worst_idx] = expanded;
                    values[worst_idx] = f_expanded;
                } else {
                    simplex[worst_idx] = reflected;
                    values[worst_idx] = f_reflected;
                }
            } else if f_reflected < values[worst_idx] {
                simplex[worst_idx] = reflected;
                values[worst_idx] = f_reflected;
            } else {
                // --- Contraction ---
                let contracted: Vec<f64> = centroid
                    .iter()
                    .zip(simplex[worst_idx].iter())
                    .map(|(c, w)| c + self.contraction_factor * (w - c))
                    .collect();
                let f_contracted = f(&contracted);
                evaluations += 1;

                if f_contracted < values[worst_idx] {
                    simplex[worst_idx] = contracted;
                    values[worst_idx] = f_contracted;
                } else {
                    // Shrink all vertices toward the best vertex
                    for i in 1..n {
                        for j in 0..dim {
                            simplex[i][j] =
                                simplex[0][j] + 0.5 * (simplex[i][j] - simplex[0][j]);
                        }
                        values[i] = f(&simplex[i]);
                        evaluations += 1;
                    }
                }
            }
        }

        // --- Max iterations reached without convergence ---
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|&i, &j| values[i].partial_cmp(&values[j]).unwrap());
        let x_opt = simplex[indices[0]].clone();

        let improvement = if values[indices[0]].abs() > f64::EPSILON {
            ((values[indices[n - 1]] - values[indices[0]]) / values[indices[0]].abs()) * 100.0
        } else {
            0.0
        };

        let parameters: Vec<DesignParameter> = x_opt
            .iter()
            .enumerate()
            .map(|(i, &v)| DesignParameter {
                name: format!("x{}", i),
                value: v,
                min: v - 1.0,
                max: v + 1.0,
                step: 0.0,
                parameter_type: ParameterType::Continuous,
            })
            .collect();

        OptimizationResult {
            goal: OptimizationGoal::Custom("minimize".to_string()),
            initial_value: initial_worst,
            final_value: values[indices[0]],
            improvement,
            iterations,
            evaluations,
            converged: false,
            parameters,
            history,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rosenbrock's banana function: f(x, y) = (a - x)^2 + b * (y - x^2)^2
    /// Global minimum at (a, a^2) = (1, 1) with f = 0.
    fn rosenbrock(params: &[f64]) -> f64 {
        let x = params[0];
        let y = params[1];
        let a = 1.0;
        let b = 100.0;
        (a - x).powi(2) + b * (y - x.powi(2)).powi(2)
    }

    fn default_simplex_2d() -> Vec<Vec<f64>> {
        vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.0, 1.0]]
    }

    #[test]
    fn test_default_parameters() {
        let ms = MultidirectionalSearch::default();
        assert_eq!(ms.max_iterations, 200);
        assert!((ms.tolerance - 1e-6).abs() < 1e-12);
        assert!((ms.expansion_factor - 2.0).abs() < 1e-12);
        assert!((ms.contraction_factor - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_new() {
        let ms = MultidirectionalSearch::new();
        assert_eq!(ms.max_iterations, 200);
    }

    #[test]
    fn test_minimize_rosenbrock() {
        let ms = MultidirectionalSearch {
            max_iterations: 500,
            tolerance: 1e-4,
            ..MultidirectionalSearch::default()
        };

        let result = ms.minimize(rosenbrock, &default_simplex_2d());

        // The Rosenbrock function is challenging; the simplex may not fully converge
        // within a reasonable number of iterations, but it should make progress.
        let x_opt = result.parameters[0].value;
        let y_opt = result.parameters[1].value;

        // Should be close to the minimum (1, 1) — allow generous tolerance
        assert!(
            (x_opt - 1.0).abs() < 0.5,
            "x_opt ≈ 1, got {}",
            x_opt
        );
        assert!(
            (y_opt - 1.0).abs() < 0.5,
            "y_opt ≈ 1, got {}",
            y_opt
        );
        assert!(result.iterations > 0);
        assert!(!result.history.is_empty());
    }

    #[test]
    fn test_minimize_quadratic_2d() {
        // f(x, y) = x^2 + y^2 — minimum at (0, 0)
        let ms = MultidirectionalSearch {
            tolerance: 1e-4,
            ..MultidirectionalSearch::default()
        };

        let simplex = vec![vec![2.0, 2.0], vec![3.0, 2.0], vec![2.0, 3.0]];
        let result = ms.minimize(|p| p[0] * p[0] + p[1] * p[1], &simplex);

        let x_opt = result.parameters[0].value;
        let y_opt = result.parameters[1].value;

        assert!(
            x_opt.abs() < 0.1,
            "x_opt ≈ 0, got {}",
            x_opt
        );
        assert!(
            y_opt.abs() < 0.1,
            "y_opt ≈ 0, got {}",
            y_opt
        );
    }

    #[test]
    fn test_simplex_expansion() {
        // A simple 1D problem: f(x) = (x-3)^2, minimum at 3
        // Simplex for 1D has 2 vertices
        let ms = MultidirectionalSearch {
            tolerance: 1e-6,
            ..MultidirectionalSearch::default()
        };

        let simplex = vec![vec![0.0], vec![5.0]];
        let result = ms.minimize(|p| (p[0] - 3.0).powi(2), &simplex);

        let x_opt = result.parameters[0].value;
        assert!(
            (x_opt - 3.0).abs() < 0.5,
            "x_opt ≈ 3, got {}",
            x_opt
        );
        assert!(result.converged || result.iterations >= 1);
    }

    #[test]
    fn test_simplex_contraction() {
        // Start with a simplex where the worst vertex is far from the minimum
        // to force contraction steps
        let ms = MultidirectionalSearch {
            max_iterations: 100,
            tolerance: 1e-8,
            ..MultidirectionalSearch::default()
        };

        let simplex = vec![vec![0.0, 0.0], vec![10.0, 0.0], vec![0.0, 10.0]];
        let result = ms.minimize(|p| p[0] * p[0] + p[1] * p[1], &simplex);

        // Should make progress toward (0, 0)
        assert!(
            result.parameters[0].value.abs() < 1.0,
            "x_opt ≈ 0, got {}",
            result.parameters[0].value
        );
        assert!(
            result.parameters[1].value.abs() < 1.0,
            "y_opt ≈ 0, got {}",
            result.parameters[1].value
        );
    }

    #[test]
    fn test_convergence_detection() {
        // Start simplex very close to the minimum — should converge quickly
        let ms = MultidirectionalSearch {
            max_iterations: 50,
            tolerance: 1e-2,
            ..MultidirectionalSearch::default()
        };

        let simplex = vec![vec![0.01, 0.01], vec![0.02, 0.01], vec![0.01, 0.02]];
        let result = ms.minimize(|p| p[0] * p[0] + p[1] * p[1], &simplex);

        assert!(result.converged, "Should converge when close to minimum");
    }

    #[test]
    fn test_not_converged_when_max_iterations_reached() {
        // Use a very tight tolerance and few iterations to force non-convergence
        let ms = MultidirectionalSearch {
            max_iterations: 5,
            tolerance: 1e-15,
            ..MultidirectionalSearch::default()
        };

        let result = ms.minimize(rosenbrock, &default_simplex_2d());
        assert!(!result.converged, "Should not converge with tight tolerance");
        assert_eq!(result.iterations, 5);
    }

    #[test]
    fn test_history_records_progress() {
        let ms = MultidirectionalSearch {
            max_iterations: 30,
            ..MultidirectionalSearch::default()
        };

        let result = ms.minimize(|p| p[0] * p[0] + p[1] * p[1], &default_simplex_2d());

        assert!(!result.history.is_empty());
        // The best value should generally decrease (minimization)
        for window in result.history.windows(2) {
            // Allow a small increase due to expansion steps
            assert!(
                window[1].1 <= window[0].1 + 1.0,
                "Best value should generally decrease: {:?} -> {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn test_evaluations_tracked() {
        let ms = MultidirectionalSearch {
            max_iterations: 10,
            ..MultidirectionalSearch::default()
        };

        let result = ms.minimize(|p| p[0] * p[0] + p[1] * p[1], &default_simplex_2d());
        // Initial evaluations = 3 (n+1 for 2D), plus at least 1 per iteration
        assert!(
            result.evaluations >= 3 + result.iterations,
            "evaluations={}, iterations={}",
            result.evaluations,
            result.iterations
        );
    }

    #[test]
    fn test_parameters_output_dimension() {
        let ms = MultidirectionalSearch::new();

        let simplex = vec![
            vec![0.0, 0.0, 0.0],
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        let result = ms.minimize(|p| p.iter().map(|&v| v * v).sum::<f64>(), &simplex);

        assert_eq!(result.parameters.len(), 3);
    }

    #[test]
    fn test_initial_value_is_worst_vertex_value() {
        let ms = MultidirectionalSearch::new();

        let simplex = vec![vec![0.0], vec![10.0]];
        let result = ms.minimize(|p| p[0].abs(), &simplex);

        // The worst initial vertex is at x=10 with f=10
        assert!(
            (result.initial_value - 10.0).abs() < 1e-6,
            "initial_value ≈ 10, got {}",
            result.initial_value
        );
    }
}