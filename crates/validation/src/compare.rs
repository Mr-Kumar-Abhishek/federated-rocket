use crate::reference::*;
use federated_rocket_simulation::engine::SimulationResult;

/// Result of comparing two simulation outputs
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub test_name: String,
    pub passed: bool,
    pub comparisons: Vec<MetricComparison>,
    pub max_deviation: f64,
    pub max_deviation_metric: String,
    pub overall_error: f64,
}

/// Comparison of a single metric
#[derive(Debug, Clone)]
pub struct MetricComparison {
    pub metric_name: String,
    pub reference_value: f64,
    pub actual_value: f64,
    pub absolute_error: f64,
    pub relative_error_percent: f64,
    pub within_tolerance: bool,
    pub tolerance: f64,
}

/// Compares federated-rocket results against OpenRocket reference data
pub struct SimulationComparator {
    pub tolerances: ValidationTolerances,
}

impl SimulationComparator {
    pub fn new() -> Self {
        Self {
            tolerances: ValidationTolerances::default(),
        }
    }

    pub fn with_tolerances(tolerances: ValidationTolerances) -> Self {
        Self { tolerances }
    }

    /// Compare full simulation results against reference
    pub fn compare(
        &self,
        name: &str,
        result: &SimulationResult,
        reference: &ReferenceSimulation,
    ) -> ValidationResult {
        let mut comparisons = Vec::new();
        let mut max_deviation = 0.0_f64;
        let mut max_deviation_metric = String::new();

        // 1. Compare key metrics
        let metrics = [
            (
                "Max Altitude",
                result.max_altitude,
                reference.max_altitude,
                self.tolerances.altitude_tolerance,
            ),
            (
                "Max Velocity",
                result.max_velocity,
                reference.max_velocity,
                self.tolerances.velocity_tolerance,
            ),
            (
                "Max Acceleration",
                result.max_acceleration,
                reference.max_acceleration,
                self.tolerances.acceleration_tolerance,
            ),
            (
                "Flight Time",
                result.flight_time,
                reference.flight_time,
                self.tolerances.event_time_tolerance,
            ),
            (
                "Apogee Time",
                result.apogee_time,
                reference.apogee_time,
                self.tolerances.event_time_tolerance,
            ),
        ];

        for (metric_name, actual, ref_val, tol) in &metrics {
            let (abs_err, rel_err) = compute_errors(*actual, *ref_val);
            let within = rel_err.abs() <= *tol || abs_err.abs() <= 1e-6;
            let dev = rel_err.abs();

            if dev > max_deviation {
                max_deviation = dev;
                max_deviation_metric = metric_name.to_string();
            }

            comparisons.push(MetricComparison {
                metric_name: metric_name.to_string(),
                reference_value: *ref_val,
                actual_value: *actual,
                absolute_error: abs_err,
                relative_error_percent: rel_err,
                within_tolerance: within,
                tolerance: *tol,
            });
        }

        // 2. Compare trajectory points (at matching time steps)
        for ref_point in &reference.trajectory {
            // Find closest point in our trajectory
            if let Some(actual) = find_closest_point(&result.trajectory, ref_point.time) {
                let alt_err = relative_error(actual.altitude(), ref_point.altitude);
                let vel_err = relative_error(actual.speed(), ref_point.velocity);

                if alt_err.abs() > self.tolerances.altitude_tolerance {
                    comparisons.push(MetricComparison {
                        metric_name: format!("Trajectory Altitude @ {:.2}s", ref_point.time),
                        reference_value: ref_point.altitude,
                        actual_value: actual.altitude(),
                        absolute_error: actual.altitude() - ref_point.altitude,
                        relative_error_percent: alt_err,
                        within_tolerance: false,
                        tolerance: self.tolerances.altitude_tolerance,
                    });
                }

                if vel_err.abs() > self.tolerances.velocity_tolerance {
                    comparisons.push(MetricComparison {
                        metric_name: format!("Trajectory Velocity @ {:.2}s", ref_point.time),
                        reference_value: ref_point.velocity,
                        actual_value: actual.speed(),
                        absolute_error: actual.speed() - ref_point.velocity,
                        relative_error_percent: vel_err,
                        within_tolerance: false,
                        tolerance: self.tolerances.velocity_tolerance,
                    });
                }
            }
        }

        // 3. Overall pass/fail
        let any_failed = comparisons.iter().any(|c| !c.within_tolerance);
        let overall_error = comparisons
            .iter()
            .map(|c| c.relative_error_percent.abs())
            .fold(0.0_f64, f64::max);

        ValidationResult {
            test_name: name.to_string(),
            passed: !any_failed,
            comparisons,
            max_deviation,
            max_deviation_metric,
            overall_error,
        }
    }
}

impl Default for SimulationComparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute absolute and relative errors
fn compute_errors(actual: f64, reference: f64) -> (f64, f64) {
    let abs_err = actual - reference;
    let rel_err = if reference.abs() > 1e-12 {
        100.0 * abs_err / reference
    } else {
        abs_err * 100.0
    };
    (abs_err, rel_err)
}

/// Relative error as percentage
fn relative_error(actual: f64, reference: f64) -> f64 {
    if reference.abs() > 1e-12 {
        100.0 * (actual - reference) / reference
    } else {
        actual * 100.0
    }
}

/// Find the closest trajectory state to a given time
fn find_closest_point<'a>(
    trajectory: &'a [federated_rocket_simulation::state::FlightState],
    time: f64,
) -> Option<&'a federated_rocket_simulation::state::FlightState> {
    trajectory.iter().min_by(|a, b| {
        let da = (a.time - time).abs();
        let db = (b.time - time).abs();
        da.partial_cmp(&db).unwrap()
    })
}
