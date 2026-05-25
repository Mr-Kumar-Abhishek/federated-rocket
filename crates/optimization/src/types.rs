use serde::{Deserialize, Serialize};

/// Optimization goal / objective function type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationGoal {
    /// Maximize the apogee / peak altitude.
    MaximizeAltitude,
    /// Maximize the peak velocity during flight.
    MaximizeVelocity,
    /// Minimize the total rocket mass.
    MinimizeMass,
    /// Minimize the total aerodynamic drag.
    MinimizeDrag,
    /// Maximize the static stability margin.
    MaximizeStability,
    /// Maximize the safety margin of the ejection delay.
    MaximizeDelaySafety,
    /// Reach a specific target altitude (m).
    TargetAltitude(f64),
    /// Reach a specific target velocity (m/s).
    TargetVelocity(f64),
    /// A custom goal identified by a string label.
    Custom(String),
}

/// The type of a design parameter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    /// Continuous real-valued parameter.
    Continuous,
    /// Discrete set of allowable values.
    Discrete,
    /// Integer-valued parameter.
    Integer,
}

/// A design parameter that can be optimized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignParameter {
    /// Human-readable name of the parameter.
    pub name: String,
    /// Current value of the parameter.
    pub value: f64,
    /// Minimum allowable value.
    pub min: f64,
    /// Maximum allowable value.
    pub max: f64,
    /// Step size for discrete / integer parameters.
    pub step: f64,
    /// The type of the parameter.
    pub parameter_type: ParameterType,
}

impl DesignParameter {
    /// Creates a new continuous design parameter.
    pub fn new_continuous(name: &str, value: f64, min: f64, max: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            min,
            max,
            step: 0.0,
            parameter_type: ParameterType::Continuous,
        }
    }

    /// Creates a new discrete design parameter.
    pub fn new_discrete(name: &str, value: f64, min: f64, max: f64, step: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            min,
            max,
            step,
            parameter_type: ParameterType::Discrete,
        }
    }

    /// Creates a new integer design parameter.
    pub fn new_integer(name: &str, value: f64, min: f64, max: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            min,
            max,
            step: 1.0,
            parameter_type: ParameterType::Integer,
        }
    }

    /// Clamp the value to the valid range `[min, max]`.
    pub fn clamp_value(&mut self) {
        self.value = self.value.clamp(self.min, self.max);
    }

    /// Normalise the current value to the range `[0, 1]`.
    pub fn normalise(&self) -> f64 {
        if (self.max - self.min).abs() < f64::EPSILON {
            0.5
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
}

/// Result of an optimization run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// The goal that was optimised for.
    pub goal: OptimizationGoal,
    /// Objective value at the initial guess.
    pub initial_value: f64,
    /// Objective value at the optimum.
    pub final_value: f64,
    /// Improvement as a percentage.
    pub improvement: f64,
    /// Number of iterations performed.
    pub iterations: usize,
    /// Number of objective evaluations.
    pub evaluations: usize,
    /// Whether the optimizer converged.
    pub converged: bool,
    /// The optimised (or final) design parameters.
    pub parameters: Vec<DesignParameter>,
    /// History of `(parameter_value, objective_value)` pairs.
    pub history: Vec<(f64, f64)>,
}

impl OptimizationResult {
    /// Returns the absolute improvement (`initial_value - final_value`).
    pub fn absolute_improvement(&self) -> f64 {
        self.initial_value - self.final_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_goal_variants() {
        let goals = [
            OptimizationGoal::MaximizeAltitude,
            OptimizationGoal::MaximizeVelocity,
            OptimizationGoal::MinimizeMass,
            OptimizationGoal::MinimizeDrag,
            OptimizationGoal::MaximizeStability,
            OptimizationGoal::MaximizeDelaySafety,
            OptimizationGoal::TargetAltitude(1000.0),
            OptimizationGoal::TargetVelocity(200.0),
            OptimizationGoal::Custom("test".to_string()),
        ];

        // Verify all variants are distinct
        for i in 0..goals.len() {
            for j in (i + 1)..goals.len() {
                assert_ne!(goals[i], goals[j], "goal variants should differ");
            }
        }
    }

    #[test]
    fn test_optimization_goal_serialization() {
        let goal = OptimizationGoal::TargetAltitude(1000.0);
        let json = serde_json::to_string(&goal).unwrap();
        let deserialized: OptimizationGoal = serde_json::from_str(&json).unwrap();
        assert_eq!(goal, deserialized);
    }

    #[test]
    fn test_design_parameter_new_continuous() {
        let param = DesignParameter::new_continuous("Nose Length", 0.15, 0.05, 0.30);
        assert_eq!(param.name, "Nose Length");
        assert!((param.value - 0.15).abs() < 1e-12);
        assert!((param.min - 0.05).abs() < 1e-12);
        assert!((param.max - 0.30).abs() < 1e-12);
        assert_eq!(param.parameter_type, ParameterType::Continuous);
    }

    #[test]
    fn test_design_parameter_new_discrete() {
        let param = DesignParameter::new_discrete("Fin Count", 4.0, 2.0, 8.0, 1.0);
        assert_eq!(param.parameter_type, ParameterType::Discrete);
        assert!((param.step - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_design_parameter_new_integer() {
        let param = DesignParameter::new_integer("Stages", 2.0, 1.0, 5.0);
        assert_eq!(param.parameter_type, ParameterType::Integer);
        assert!((param.step - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_design_parameter_clamp_value() {
        let mut param = DesignParameter::new_continuous("Test", 10.0, 0.0, 5.0);
        param.clamp_value();
        assert!((param.value - 5.0).abs() < 1e-12);

        let mut param2 = DesignParameter::new_continuous("Test", -10.0, 0.0, 5.0);
        param2.clamp_value();
        assert!((param2.value - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_design_parameter_normalise() {
        let param = DesignParameter::new_continuous("Test", 0.15, 0.05, 0.30);
        let normalised = param.normalise();
        assert!((normalised - 0.4).abs() < 1e-12); // (0.15 - 0.05) / 0.25 = 0.4
    }

    #[test]
    fn test_design_parameter_zero_range_normalise() {
        let param = DesignParameter::new_continuous("Test", 0.5, 0.5, 0.5);
        let normalised = param.normalise();
        assert!((normalised - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_design_parameter_serialization() {
        let param = DesignParameter::new_continuous("Length", 1.0, 0.0, 2.0);
        let json = serde_json::to_string(&param).unwrap();
        let deserialized: DesignParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(param.name, deserialized.name);
        assert!((param.value - deserialized.value).abs() < 1e-12);
    }

    #[test]
    fn test_optimization_result_construction() {
        let result = OptimizationResult {
            goal: OptimizationGoal::MinimizeMass,
            initial_value: 10.0,
            final_value: 7.5,
            improvement: 25.0,
            iterations: 50,
            evaluations: 100,
            converged: true,
            parameters: vec![],
            history: vec![(0.0, 10.0), (1.0, 7.5)],
        };

        assert_eq!(result.goal, OptimizationGoal::MinimizeMass);
        assert!((result.absolute_improvement() - 2.5).abs() < 1e-12);
        assert!((result.improvement - 25.0).abs() < 1e-12);
        assert!(result.converged);
    }

    #[test]
    fn test_optimization_result_serialization() {
        let result = OptimizationResult {
            goal: OptimizationGoal::MaximizeAltitude,
            initial_value: 100.0,
            final_value: 250.0,
            improvement: 150.0,
            iterations: 30,
            evaluations: 60,
            converged: true,
            parameters: vec![DesignParameter::new_continuous("Test", 1.0, 0.0, 2.0)],
            history: vec![],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: OptimizationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.goal, deserialized.goal);
        assert_eq!(result.iterations, deserialized.iterations);
        assert_eq!(result.converged, deserialized.converged);
    }
}
