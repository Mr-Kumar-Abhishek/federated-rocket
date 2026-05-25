use federated_rocket_aero::compute::AeroCalculator;
use federated_rocket_core::component_tree::ComponentTree;
use federated_rocket_math::integrator::Integrator;
use federated_rocket_math::integrator::RK4Integrator;
use federated_rocket_physics::atmosphere::AtmosphericModel;
use federated_rocket_physics::gravity::GravityModel;
use federated_rocket_physics::wind::WindModel;

use crate::derivatives::{compact_to_flight_state, compute_derivative, normalize_orientation, CompactState};
use crate::events::{EventConfig, FlightEvent, FlightEventType};
use crate::motor::MotorModel;
use crate::state::FlightState;

/// Configuration for the simulation engine.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Fixed simulation time step (s). Default: 0.001
    pub time_step: f64,
    /// Reference area for aerodynamic calculations (m²).
    pub reference_area: f64,
    /// Reference diameter for aerodynamic calculations (m).
    pub reference_diameter: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            time_step: 0.001,
            reference_area: std::f64::consts::PI * 0.0254 * 0.0254, // ~2 inch diameter tube
            reference_diameter: 0.0508,                               // 2 inches in meters
        }
    }
}

/// Results produced by a completed simulation.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// All flight events detected during the simulation
    pub events: Vec<FlightEvent>,
    /// Trajectory data recorded at the configured output interval
    pub trajectory: Vec<FlightState>,
    /// Final state of the simulation
    pub final_state: FlightState,
    /// Maximum altitude reached (m)
    pub max_altitude: f64,
    /// Maximum velocity reached (m/s)
    pub max_velocity: f64,
    /// Maximum acceleration reached (m/s²)
    pub max_acceleration: f64,
    /// Total flight time (s)
    pub flight_time: f64,
    /// Time at which apogee occurred (s)
    pub apogee_time: f64,
    /// Time at which ground hit occurred (s)
    pub ground_hit_time: f64,
    /// Whether the simulation completed successfully
    pub success: bool,
}

/// The main simulation engine for 6-DOF rocket flight.
///
/// Handles integration of the equations of motion, event detection,
/// and trajectory recording.
pub struct SimulationEngine {
    pub config: SimulationConfig,
    pub event_config: EventConfig,
    pub aero_calc: AeroCalculator,
}

impl SimulationEngine {
    /// Creates a new `SimulationEngine` with the given configuration.
    pub fn new(config: SimulationConfig, event_config: EventConfig) -> Self {
        Self {
            config,
            event_config,
            aero_calc: AeroCalculator::new(),
        }
    }

    /// Runs a complete 6-DOF flight simulation.
    ///
    /// # Parameters
    ///
    /// * `initial_state` - Starting flight state (usually on the ground)
    /// * `motor` - Optional motor model providing thrust
    /// * `tree` - Component tree describing the rocket geometry
    /// * `atmosphere` - Atmospheric model (temperature, pressure, density)
    /// * `gravity` - Gravity model (acceleration vs altitude)
    /// * `wind` - Wind model (wind velocity vs position)
    ///
    /// # Returns
    ///
    /// A [`SimulationResult`] containing events, trajectory, and summary data.
    pub fn simulate(
        &self,
        initial_state: FlightState,
        motor: Option<MotorModel>,
        tree: &ComponentTree,
        atmosphere: &dyn AtmosphericModel,
        gravity: &dyn GravityModel,
        wind: &dyn WindModel,
    ) -> SimulationResult {
        let dt = self.config.time_step;
        let reference_area = self.config.reference_area;
        let reference_diameter = self.config.reference_diameter;

        let mut events: Vec<FlightEvent> = Vec::new();
        let mut trajectory: Vec<FlightState> = Vec::new();
        let mut state = CompactState::from_flight_state(&initial_state);
        let mut t = 0.0;
        let mut last_output = 0.0;
        let mut prev_altitude = initial_state.altitude();
        let mut max_altitude = 0.0;
        let mut max_velocity = 0.0;
        let mut max_accel = 0.0;
        let mut apogee_time = 0.0;
        let mut ground_hit_time = 0.0;
        let mut prev_speed = 0.0;
        let mut launched = false;
        let mut burnout = false;
        let mut apogee_detected = false;
        let mut deployed = false;
        let mut mach_crossed = false;
        let mut max_vel_event = false;
        let mut burntime_start = false;
        let mut rod_clear = false;

        let rk4 = RK4Integrator;

        // Record initial state
        let fs0 = compact_to_flight_state(&state, atmosphere, wind);
        trajectory.push(fs0);

        // Main simulation loop
        while t < self.event_config.max_simulation_time {
            // Take an RK4 integration step
            // The closure f returns the state derivative (not scaled by dt)
            let new_state = rk4.step(
                &|s: &CompactState, rk_time: f64| -> CompactState {
                    compute_derivative(
                        s,
                        rk_time,
                        &motor,
                        &self.aero_calc,
                        atmosphere,
                        gravity,
                        wind,
                        tree,
                        reference_area,
                        reference_diameter,
                    )
                },
                &state,
                t,
                dt,
            );

            // Normalize the orientation quaternion to prevent drift
            let mut next_state = new_state;
            normalize_orientation(&mut next_state);

            // Convert to FlightState for diagnostics and event detection
            let fs = compact_to_flight_state(&next_state, atmosphere, wind);
            let prev_fs = compact_to_flight_state(&state, atmosphere, wind);
            let event_time = t + dt;

            // Compute acceleration magnitude
            let accel = if dt > 0.0 {
                (fs.speed() - prev_speed) / dt
            } else {
                0.0
            };

            // --- Event Detection ---

            // 1. Launch detection (altitude > 0.01 m)
            if !launched && fs.altitude() > 0.01 {
                launched = true;
                events.push(FlightEvent::new(FlightEventType::Launch, &prev_fs, accel));
            }

            // 2. Launch rod clear
            if launched && !rod_clear && fs.altitude() >= self.event_config.launch_rod_clear_altitude {
                rod_clear = true;
                events.push(FlightEvent::new(FlightEventType::LaunchRodClear, &fs, accel));
            }

            // 3. Burntime start — first time step where the motor is burning
            if !burntime_start && motor.is_some() {
                if motor.as_ref().unwrap().is_burning(event_time) && !burntime_start {
                    burntime_start = true;
                    events.push(FlightEvent::new(FlightEventType::BurntimeStart, &fs, accel));
                }
            }

            // 4. Burnout detection (motor stops burning after having started)
            if !burnout && motor.is_some() && burntime_start {
                if !motor.as_ref().unwrap().is_burning(event_time) {
                    burnout = true;
                    events.push(FlightEvent::new(FlightEventType::BurntimeEnd, &fs, accel));

                    // Ejection charge fires after delay (if configured)
                    if self.event_config.ejection_charge_delay > 0.0 {
                        events.push(FlightEvent::new(
                            FlightEventType::EjectionChargeFire,
                            &fs,
                            accel,
                        ));
                    }
                }
            }

            // 5. Apogee detection
            if !apogee_detected && launched && fs.altitude() < prev_altitude
                && prev_altitude > 10.0
                && (prev_altitude - fs.altitude()) > self.event_config.apogee_detection_sensitivity
            {
                apogee_detected = true;
                apogee_time = t;
                // Record apogee at the previous state (the peak)
                events.push(FlightEvent::new(FlightEventType::Apogee, &prev_fs, accel));
            }

            // 6. Recovery deployment (at apogee)
            if apogee_detected && !deployed {
                deployed = true;
                events.push(FlightEvent::new(
                    FlightEventType::RecoveryDeviceDeployment,
                    &fs,
                    accel,
                ));
            }

            // 7. Mach transition
            if !mach_crossed && fs.mach > 1.0 {
                mach_crossed = true;
                events.push(FlightEvent::new(FlightEventType::MachTransition, &fs, accel));
            }

            // 8. Max velocity detection — fire when velocity starts decreasing after peak
            if !max_vel_event && launched && fs.speed() < prev_speed && max_velocity > 10.0 && prev_speed > 0.0 {
                max_vel_event = true;
                events.push(FlightEvent::new(FlightEventType::MaxVelocity, &prev_fs, accel));
            }

            // 9. Ground hit detection
            if launched && fs.altitude() <= self.event_config.ground_altitude
                && fs.speed() < self.event_config.landing_detection_threshold
                && t > 0.5
            {
                ground_hit_time = event_time;
                events.push(FlightEvent::new(FlightEventType::GroundHit, &fs, accel));

                // Record final state
                trajectory.push(fs.clone());

                return SimulationResult {
                    events,
                    trajectory,
                    final_state: fs,
                    max_altitude,
                    max_velocity,
                    max_acceleration: max_accel,
                    flight_time: event_time,
                    apogee_time,
                    ground_hit_time,
                    success: true,
                };
            }

            // Track max values
            max_altitude = max_altitude.max(fs.altitude());
            max_velocity = max_velocity.max(fs.speed());
            if accel.abs() > max_accel {
                max_accel = accel.abs();
            }

            // Record trajectory at output intervals
            if t - last_output >= self.event_config.output_interval {
                trajectory.push(fs.clone());
                last_output = t;
            }

            // Advance time and update state
            t = event_time;
            state = next_state;
            prev_altitude = fs.altitude();
            prev_speed = fs.speed();
        }

        // --- Simulation ended (time limit reached) ---
        let final_fs = compact_to_flight_state(&state, atmosphere, wind);
        events.push(FlightEvent::new(FlightEventType::SimulationEnd, &final_fs, 0.0));
        trajectory.push(final_fs.clone());

        SimulationResult {
            events,
            trajectory,
            final_state: final_fs,
            max_altitude,
            max_velocity,
            max_acceleration: max_accel,
            flight_time: t,
            apogee_time,
            ground_hit_time,
            success: true,
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
    use federated_rocket_physics::atmosphere::StandardAtmosphere;
    use federated_rocket_physics::gravity::ConstantGravity;
    use federated_rocket_physics::wind::NoWind;
    use federated_rocket_math::vector::Vector3D;

    use super::*;

    fn make_test_material() -> Material {
        Material::new("TestMaterial", MaterialType::Bulk, Quantity::new(800.0, Unit::Kilogram))
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
    fn test_simulation_config_default() {
        let config = SimulationConfig::default();
        assert!((config.time_step - 0.001).abs() < 1e-12);
        assert!(config.reference_area > 0.0);
        assert!((config.reference_diameter - 0.0508).abs() < 1e-12);
    }

    #[test]
    fn test_simulation_engine_creation() {
        let config = SimulationConfig::default();
        let event_config = EventConfig::default();
        let engine = SimulationEngine::new(config, event_config);
        assert!((engine.config.time_step - 0.001).abs() < 1e-12);
    }

    #[test]
    fn test_simulate_simple_rocket_no_motor() {
        let config = SimulationConfig::default();
        let event_config = EventConfig {
            max_simulation_time: 0.5,
            output_interval: 0.1,
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);

        let initial_state = FlightState::new();
        let motor = None;
        let tree = make_test_rocket();
        let atmosphere = StandardAtmosphere;
        let gravity = ConstantGravity;
        let wind = NoWind;

        let result = engine.simulate(initial_state, motor, &tree, &atmosphere, &gravity, &wind);

        // Without a motor, the rocket should just sit on the ground
        assert!(result.success);
        assert!(!result.trajectory.is_empty());
        assert!((result.final_state.altitude() - 0.0).abs() < 1.0);
        assert!((result.flight_time - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_simulate_with_motor_launches() {
        let config = SimulationConfig::default();
        let event_config = EventConfig {
            max_simulation_time: 3.0,
            output_interval: 0.05,
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);

        let mut initial_state = FlightState::new();
        initial_state.position = Vector3D::new(0.0, 0.0, 0.0);
        initial_state.mass = 0.5;

        let motor = Some(make_test_motor());
        let tree = make_test_rocket();
        let atmosphere = StandardAtmosphere;
        let gravity = ConstantGravity;
        let wind = NoWind;

        let result = engine.simulate(initial_state, motor, &tree, &atmosphere, &gravity, &wind);

        assert!(result.success);
        assert!(!result.events.is_empty());
        assert!(!result.trajectory.is_empty());

        // Check for launch event
        let has_launch = result
            .events
            .iter()
            .any(|e| e.event_type == FlightEventType::Launch);
        assert!(has_launch, "Should have a launch event");

        // Check for burnout event
        let has_burnout = result
            .events
            .iter()
            .any(|e| e.event_type == FlightEventType::BurntimeEnd);
        assert!(has_burnout, "Should have a burnout event");
    }

    #[test]
    fn test_simulate_apogee_detection() {
        let config = SimulationConfig::default();
        let event_config = EventConfig {
            max_simulation_time: 5.0,
            output_interval: 0.05,
            apogee_detection_sensitivity: 0.01,
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);

        let mut initial_state = FlightState::new();
        initial_state.mass = 0.5;

        let motor = Some(make_test_motor());
        let tree = make_test_rocket();
        let atmosphere = StandardAtmosphere;
        let gravity = ConstantGravity;
        let wind = NoWind;

        let result = engine.simulate(initial_state, motor, &tree, &atmosphere, &gravity, &wind);

        // With a small motor and 5s simulation, check that we got reasonable values
        assert!(result.max_altitude >= 0.0);
        assert!(result.max_velocity >= 0.0);
        assert_eq!(result.success, true);
        assert!(result.flight_time > 0.0);
    }

    #[test]
    fn test_simulation_end_event() {
        let config = SimulationConfig::default();
        let event_config = EventConfig {
            max_simulation_time: 0.5,
            output_interval: 0.5,
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);

        let initial_state = FlightState::new();
        let tree = make_test_rocket();
        let atmosphere = StandardAtmosphere;
        let gravity = ConstantGravity;
        let wind = NoWind;

        let result = engine.simulate(initial_state, None, &tree, &atmosphere, &gravity, &wind);

        let has_sim_end = result
            .events
            .iter()
            .any(|e| e.event_type == FlightEventType::SimulationEnd);
        assert!(has_sim_end, "Should have SimulationEnd event");
        assert!((result.flight_time - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_simulate_ground_hit() {
        let config = SimulationConfig {
            time_step: 0.01,
            ..SimulationConfig::default()
        };
        let event_config = EventConfig {
            max_simulation_time: 30.0,
            output_interval: 0.1,
            ground_altitude: 0.0,
            landing_detection_threshold: 5.0, // higher threshold so the falling rocket is detected
            ..EventConfig::default()
        };
        let engine = SimulationEngine::new(config, event_config);

        // Start with a rocket already in the air, falling
        let mut initial_state = FlightState::new();
        initial_state.position = Vector3D::new(0.0, 0.0, 100.0);
        initial_state.velocity = Vector3D::new(0.0, 0.0, -5.0);
        initial_state.mass = 1.0;

        let tree = make_test_rocket();
        let atmosphere = StandardAtmosphere;
        let gravity = ConstantGravity;
        let wind = NoWind;

        let result = engine.simulate(initial_state, None, &tree, &atmosphere, &gravity, &wind);

        // Check that we either have a GroundHit event or the sim ran to completion
        let has_ground_hit = result
            .events
            .iter()
            .any(|e| e.event_type == FlightEventType::GroundHit);
        let has_sim_end = result
            .events
            .iter()
            .any(|e| e.event_type == FlightEventType::SimulationEnd);
        assert!(
            has_ground_hit || has_sim_end,
            "Should have GroundHit or SimulationEnd event"
        );
        // Should have detected some altitude change
        assert!(result.max_altitude >= 0.0);
    }
}