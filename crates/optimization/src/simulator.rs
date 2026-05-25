use federated_rocket_core::component_tree::ComponentTree;
use federated_rocket_physics::atmosphere::AtmosphericModel;
use federated_rocket_physics::gravity::GravityModel;
use federated_rocket_physics::wind::WindModel;
use federated_rocket_simulation::engine::SimulationEngine;
use federated_rocket_simulation::motor::MotorModel;
use federated_rocket_simulation::state::FlightState;

use crate::types::OptimizationGoal;

/// Wraps the flight simulation engine for use as an objective function in
/// optimisation algorithms.
///
/// The `evaluate` method runs a full 6-DOF simulation with the provided
/// parameter values and returns a scalar objective value that reflects how
/// well the flight performance matches the configured goal.
pub struct SimulatorObjective {
    /// The simulation engine (config, event config, aero calculator).
    pub engine: SimulationEngine,
    /// The rocket component tree describing the vehicle geometry.
    pub component_tree: ComponentTree,
    /// Optional motor model providing thrust.
    pub motor: Option<MotorModel>,
    /// The optimisation goal driving the objective calculation.
    pub goal: OptimizationGoal,
    /// Atmospheric model used during simulation.
    pub atmosphere: Box<dyn AtmosphericModel>,
    /// Gravity model used during simulation.
    pub gravity: Box<dyn GravityModel>,
    /// Wind model used during simulation.
    pub wind: Box<dyn WindModel>,
}

impl SimulatorObjective {
    /// Create a new `SimulatorObjective`.
    ///
    /// # Parameters
    ///
    /// * `engine` - The configured simulation engine.
    /// * `tree` - The rocket component tree.
    /// * `motor` - Optional motor.
    /// * `goal` - The desired optimisation goal.
    pub fn new(
        engine: SimulationEngine,
        tree: ComponentTree,
        motor: Option<MotorModel>,
        goal: OptimizationGoal,
    ) -> Self {
        // Use default models; users can replace them after construction if needed.
        let atmosphere: Box<dyn AtmosphericModel> =
            Box::new(federated_rocket_physics::atmosphere::StandardAtmosphere);
        let gravity: Box<dyn GravityModel> =
            Box::new(federated_rocket_physics::gravity::ConstantGravity);
        let wind: Box<dyn WindModel> = Box::new(federated_rocket_physics::wind::NoWind);

        Self {
            engine,
            component_tree: tree,
            motor,
            goal,
            atmosphere,
            gravity,
            wind,
        }
    }

    /// Evaluate the objective function for a given set of parameter values.
    ///
    /// This runs a full simulation and returns a scalar value.  The convention
    /// is **always minimise**: goals that represent maximisation are negated
    /// internally so that smaller return values correspond to better designs.
    ///
    /// # Parameters
    ///
    /// * `_params` - The design parameter values (currently unused; reserved
    ///   for future parameter-to-simulation mapping).
    pub fn evaluate(&self, _params: &[f64]) -> f64 {
        let initial_state = FlightState::new();
        let result = self.engine.simulate(
            initial_state,
            self.motor.clone(),
            &self.component_tree,
            self.atmosphere.as_ref(),
            self.gravity.as_ref(),
            self.wind.as_ref(),
        );

        match self.goal {
            OptimizationGoal::MaximizeAltitude => -result.max_altitude, // negate ⇒ minimise
            OptimizationGoal::MaximizeVelocity => -result.max_velocity,
            OptimizationGoal::MinimizeMass => result.final_state.mass,
            OptimizationGoal::MinimizeDrag => {
                // Use max dynamic pressure as a proxy for drag impact
                result
                    .trajectory
                    .iter()
                    .map(|s| s.dynamic_pressure)
                    .fold(f64::NEG_INFINITY, f64::max)
            }
            OptimizationGoal::MaximizeStability => {
                // Use angle of attack stability: minimise the integrated deviation
                let sum_aoa: f64 = result.trajectory.iter().map(|s| s.angle_of_attack).sum();
                -sum_aoa // negate ⇒ minimise ⇒ maximise stability
            }
            OptimizationGoal::MaximizeDelaySafety => {
                // Penalise late deployment (apogee_time close to ground_hit_time)
                if result.ground_hit_time > 0.0 {
                    -(result.apogee_time - result.ground_hit_time).abs()
                } else {
                    -1000.0 // no ground hit → very safe
                }
            }
            OptimizationGoal::TargetAltitude(target) => {
                (result.max_altitude - target).abs()
            }
            OptimizationGoal::TargetVelocity(target) => {
                (result.max_velocity - target).abs()
            }
            OptimizationGoal::Custom(_) => result.max_altitude,
        }
    }
}

#[cfg(test)]
mod tests {
    use federated_rocket_core::component::{
        BodyTubeData, NoseConeData, NoseConeShape, RocketComponent,
    };
    use federated_rocket_core::coordinate::Coordinate;
    use federated_rocket_core::material::{Material, MaterialType};
    use federated_rocket_core::units::{Quantity, Unit};
    use federated_rocket_simulation::engine::SimulationConfig;
    use federated_rocket_simulation::events::EventConfig;

    use super::*;

    fn make_test_material() -> Material {
        Material::new(
            "TestMaterial",
            MaterialType::Bulk,
            Quantity::new(800.0, Unit::Kilogram),
        )
    }

    fn make_test_rocket() -> ComponentTree {
        let mut tree = ComponentTree::new();
        let mat = make_test_material();

        let nose = RocketComponent::NoseCone(NoseConeData {
            name: "Nose".into(),
            position: Coordinate::new(0.0, 0.0, 0.0),
            length: Quantity::new(0.15, Unit::Meter),
            base_radius: Quantity::new(0.0254, Unit::Meter),
            shape: NoseConeShape::Conical,
            thickness: Quantity::new(0.002, Unit::Meter),
            material: mat.clone(),
            color: None,
            shoulder_length: Quantity::new(0.02, Unit::Meter),
            shoulder_radius: Quantity::new(0.024, Unit::Meter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Meter),
        });

        let body = RocketComponent::BodyTube(BodyTubeData {
            name: "Body".into(),
            position: Coordinate::new(0.0, 0.0, 0.15),
            length: Quantity::new(0.5, Unit::Meter),
            outer_radius: Quantity::new(0.0254, Unit::Meter),
            inner_radius: Quantity::new(0.024, Unit::Meter),
            material: mat,
            color: None,
            has_motor_mount: true,
        });

        let nose_key = tree.add_component(nose, None).unwrap();
        tree.add_component(body, Some(nose_key)).unwrap();
        tree
    }

    fn make_test_motor() -> MotorModel {
        let mut motor = MotorModel::new("TestCo".into(), "T100".into());
        motor.diameter = 29.0;
        motor.length = 150.0;
        motor.dry_mass = 0.15;
        motor.propellant_mass = 0.085;
        motor.burn_time = 2.0;
        motor.total_impulse = 200.0;

        motor.add_thrust_point(0.0, 0.0);
        motor.add_thrust_point(0.1, 100.0);
        motor.add_thrust_point(0.5, 120.0);
        motor.add_thrust_point(1.0, 110.0);
        motor.add_thrust_point(1.5, 80.0);
        motor.add_thrust_point(2.0, 0.0);

        motor
    }

    #[test]
    fn test_simulator_objective_creation() {
        let config = SimulationConfig::default();
        let event_config = EventConfig::default();
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();
        let motor = Some(make_test_motor());

        let objective = SimulatorObjective::new(
            engine,
            tree,
            motor,
            OptimizationGoal::MaximizeAltitude,
        );

        // Check that default models are set
        assert_eq!(
            objective.atmosphere.name(),
            "International Standard Atmosphere (ISA)"
        );
        assert_eq!(
            objective.gravity.name(),
            "Constant Gravity (9.80665 m/s²)"
        );
        assert_eq!(objective.wind.name(), "No Wind");
    }

    #[test]
    fn test_evaluate_maximize_altitude() {
        let config = SimulationConfig {
            time_step: 0.01,
            ..SimulationConfig::default()
        };
        let event_config = EventConfig {
            max_simulation_time: 3.0,
            output_interval: Some(0.1),
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();
        let motor = Some(make_test_motor());

        let objective = SimulatorObjective::new(
            engine,
            tree,
            motor,
            OptimizationGoal::MaximizeAltitude,
        );

        let value = objective.evaluate(&[]);

        // MaximiseAltitude negates the altitude, so value should be negative
        assert!(
            value <= 0.0,
            "MaximizeAltitude should return a negative value, got {}",
            value
        );
    }

    #[test]
    fn test_evaluate_maximize_velocity() {
        let config = SimulationConfig {
            time_step: 0.01,
            ..SimulationConfig::default()
        };
        let event_config = EventConfig {
            max_simulation_time: 3.0,
            output_interval: Some(0.1),
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();
        let motor = Some(make_test_motor());

        let objective = SimulatorObjective::new(
            engine,
            tree,
            motor,
            OptimizationGoal::MaximizeVelocity,
        );

        let value = objective.evaluate(&[]);

        // MaximizeVelocity negates velocity
        assert!(
            value <= 0.0,
            "MaximizeVelocity should return a negative value, got {}",
            value
        );
    }

    #[test]
    fn test_evaluate_minimize_mass() {
        let config = SimulationConfig::default();
        let event_config = EventConfig {
            max_simulation_time: 0.5,
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();

        let objective =
            SimulatorObjective::new(engine, tree, None, OptimizationGoal::MinimizeMass);

        let value = objective.evaluate(&[]);

        // Mass should be positive
        assert!(
            value > 0.0,
            "MinimizeMass should return a positive mass, got {}",
            value
        );
    }

    #[test]
    fn test_evaluate_target_altitude() {
        let config = SimulationConfig {
            time_step: 0.01,
            ..SimulationConfig::default()
        };
        let event_config = EventConfig {
            max_simulation_time: 3.0,
            output_interval: Some(0.1),
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();
        let motor = Some(make_test_motor());

        let objective = SimulatorObjective::new(
            engine,
            tree,
            motor,
            OptimizationGoal::TargetAltitude(100.0),
        );

        let value = objective.evaluate(&[]);

        // Should return a non-negative absolute difference
        assert!(
            value >= 0.0,
            "TargetAltitude should return a non-negative value, got {}",
            value
        );
    }

    #[test]
    fn test_evaluate_target_velocity() {
        let config = SimulationConfig {
            time_step: 0.01,
            ..SimulationConfig::default()
        };
        let event_config = EventConfig {
            max_simulation_time: 3.0,
            output_interval: Some(0.1),
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();
        let motor = Some(make_test_motor());

        let objective = SimulatorObjective::new(
            engine,
            tree,
            motor,
            OptimizationGoal::TargetVelocity(50.0),
        );

        let value = objective.evaluate(&[]);

        // Should return a non-negative absolute difference
        assert!(
            value >= 0.0,
            "TargetVelocity should return a non-negative value, got {}",
            value
        );
    }

    #[test]
    fn test_evaluate_custom_defaults_to_altitude() {
        let config = SimulationConfig {
            time_step: 0.01,
            ..SimulationConfig::default()
        };
        let event_config = EventConfig {
            max_simulation_time: 3.0,
            output_interval: Some(0.1),
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();
        let motor = Some(make_test_motor());

        let objective = SimulatorObjective::new(
            engine,
            tree,
            motor,
            OptimizationGoal::Custom("test_custom".to_string()),
        );

        let value = objective.evaluate(&[]);

        // Custom defaults to max_altitude (positive)
        assert!(
            value >= 0.0,
            "Custom goal should return a non-negative altitude, got {}",
            value
        );
    }

    #[test]
    fn test_set_atmosphere_model() {
        let config = SimulationConfig::default();
        let event_config = EventConfig::default();
        let engine = SimulationEngine::new(config, event_config);
        let tree = make_test_rocket();

        let mut objective = SimulatorObjective::new(
            engine,
            tree,
            None,
            OptimizationGoal::MinimizeMass,
        );

        // Replace with a custom model
        objective.atmosphere =
            Box::new(federated_rocket_physics::atmosphere::IsothermalAtmosphere::new(
                300.0,
                100000.0,
            ));

        assert_eq!(objective.atmosphere.name(), "Isothermal Atmosphere");
    }
}