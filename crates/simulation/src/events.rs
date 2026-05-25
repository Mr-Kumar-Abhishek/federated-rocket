use crate::state::FlightState;
use serde::{Deserialize, Serialize};

/// Types of flight events that can occur during simulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlightEventType {
    /// Rocket has left the launch pad
    Launch,
    /// Rocket has cleared the launch rod / rail
    LaunchRodClear,
    /// Motor ignition / start of burn
    BurntimeStart,
    /// Motor burnout (end of thrust)
    BurntimeEnd,
    /// Ejection charge fires
    EjectionChargeFire,
    /// Apogee (maximum altitude) reached
    Apogee,
    /// Drogue / recovery device deploys
    RecoveryDeviceDeployment,
    /// Main parachute deploys
    MainDeployment,
    /// Rocket has hit the ground
    GroundHit,
    /// Maximum velocity reached
    MaxVelocity,
    /// Maximum acceleration reached
    MaxAcceleration,
    /// Simulation has ended
    SimulationEnd,
    /// Significant wind change detected
    WindChange,
    /// Stage separation event
    StageSeparation,
    /// Motor ignition (for multi-stage)
    Ignition,
    /// Transonic / supersonic transition (Mach > 1)
    MachTransition,
    /// Rocket has entered a tumble / ballistic fall
    Tumble,
    /// An error condition occurred
    Error,
}

/// A flight event with time and state information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightEvent {
    /// Type of event
    pub event_type: FlightEventType,
    /// Simulation time of the event (s)
    pub time: f64,
    /// Altitude at event (m)
    pub altitude: f64,
    /// Velocity magnitude at event (m/s)
    pub velocity: f64,
    /// Mach number at event
    pub mach: f64,
    /// Acceleration magnitude at event (m/s²)
    pub acceleration: f64,
    /// Human-readable description of the event
    pub description: String,
}

impl FlightEvent {
    /// Creates a new `FlightEvent` from an event type and the current flight state.
    ///
    /// The acceleration parameter is passed separately because it requires
    /// a time derivative that the state alone does not carry.
    pub fn new(
        event_type: FlightEventType,
        state: &FlightState,
        acceleration: f64,
    ) -> Self {
        // Build a default description based on event type
        let description = match event_type {
            FlightEventType::Launch => "Rocket launched".to_string(),
            FlightEventType::LaunchRodClear => "Launch rod cleared".to_string(),
            FlightEventType::BurntimeStart => "Motor ignition / burn started".to_string(),
            FlightEventType::BurntimeEnd => "Motor burnout".to_string(),
            FlightEventType::EjectionChargeFire => "Ejection charge fired".to_string(),
            FlightEventType::Apogee => "Apogee reached".to_string(),
            FlightEventType::RecoveryDeviceDeployment => "Recovery device deployed".to_string(),
            FlightEventType::MainDeployment => "Main parachute deployed".to_string(),
            FlightEventType::GroundHit => "Rocket landed".to_string(),
            FlightEventType::MaxVelocity => "Maximum velocity reached".to_string(),
            FlightEventType::MaxAcceleration => "Maximum acceleration reached".to_string(),
            FlightEventType::SimulationEnd => "Simulation ended".to_string(),
            FlightEventType::WindChange => "Wind condition change detected".to_string(),
            FlightEventType::StageSeparation => "Stage separation".to_string(),
            FlightEventType::Ignition => "Motor ignition".to_string(),
            FlightEventType::MachTransition => "Mach transition (supersonic)".to_string(),
            FlightEventType::Tumble => "Rocket tumbling".to_string(),
            FlightEventType::Error => "Simulation error encountered".to_string(),
        };

        Self {
            event_type,
            time: state.time,
            altitude: state.altitude(),
            velocity: state.speed(),
            mach: state.mach,
            acceleration,
            description,
        }
    }

    /// Creates a new `FlightEvent` with a custom time (for interpolated events).
    ///
    /// This is used when event detection occurs at a time between simulation steps,
    /// such as when using bisection or interpolation for more accurate event timing.
    pub fn new_with_time(
        event_type: FlightEventType,
        time: f64,
        state: &FlightState,
    ) -> Self {
        let description = match event_type {
            FlightEventType::Launch => "Rocket launched".to_string(),
            FlightEventType::LaunchRodClear => "Launch rod cleared".to_string(),
            FlightEventType::BurntimeStart => "Motor ignition / burn started".to_string(),
            FlightEventType::BurntimeEnd => "Motor burnout".to_string(),
            FlightEventType::EjectionChargeFire => "Ejection charge fired".to_string(),
            FlightEventType::Apogee => "Apogee reached".to_string(),
            FlightEventType::RecoveryDeviceDeployment => "Recovery device deployed".to_string(),
            FlightEventType::MainDeployment => "Main parachute deployed".to_string(),
            FlightEventType::GroundHit => "Rocket landed".to_string(),
            FlightEventType::MaxVelocity => "Maximum velocity reached".to_string(),
            FlightEventType::MaxAcceleration => "Maximum acceleration reached".to_string(),
            FlightEventType::SimulationEnd => "Simulation ended".to_string(),
            FlightEventType::WindChange => "Wind condition change detected".to_string(),
            FlightEventType::StageSeparation => "Stage separation".to_string(),
            FlightEventType::Ignition => "Motor ignition".to_string(),
            FlightEventType::MachTransition => "Mach transition (supersonic)".to_string(),
            FlightEventType::Tumble => "Rocket tumbling".to_string(),
            FlightEventType::Error => "Simulation error encountered".to_string(),
        };

        Self {
            event_type,
            time,
            altitude: state.altitude(),
            velocity: state.speed(),
            mach: state.mach,
            acceleration: 0.0,
            description,
        }
    }

    /// Creates a new `FlightEvent` with a custom description.
    pub fn with_description(
        event_type: FlightEventType,
        state: &FlightState,
        acceleration: f64,
        description: String,
    ) -> Self {
        Self {
            event_type,
            time: state.time,
            altitude: state.altitude(),
            velocity: state.speed(),
            mach: state.mach,
            acceleration,
            description,
        }
    }
}

/// Configuration for when flight events should trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// Altitude at which the rocket is considered to have cleared the launch rod (m).
    /// Default: 2.0
    pub launch_rod_clear_altitude: f64,
    /// Delay after burnout before ejection charge fires (s).
    /// Default: 0.0
    pub ejection_charge_delay: f64,
    /// Altitude for main parachute deployment. `None` means deploy at apogee.
    /// Default: None
    pub main_deployment_altitude: Option<f64>,
    /// Sensitivity for apogee detection (minimum altitude change to confirm apogee).
    /// Default: 0.1
    pub apogee_detection_sensitivity: f64,
    /// Maximum simulation time (s).
    /// Default: 120.0
    pub max_simulation_time: f64,
    /// Ground altitude (m) at which the rocket is considered landed.
    /// Default: 0.0
    pub ground_altitude: f64,
    /// Speed threshold (m/s) below which the rocket is considered landed.
    /// Default: 0.5
    pub landing_detection_threshold: f64,
    /// Interval at which trajectory data points are recorded (s).
    /// `None` uses the default of 0.1s.
    /// Default: None (0.1)
    pub output_interval: Option<f64>,
    /// Acceleration threshold (m/s²) for MaxAcceleration event detection.
    /// Default: 1.0
    pub acceleration_threshold: f64,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            launch_rod_clear_altitude: 2.0,
            ejection_charge_delay: 0.0,
            main_deployment_altitude: None,
            apogee_detection_sensitivity: 0.1,
            max_simulation_time: 120.0,
            ground_altitude: 0.0,
            landing_detection_threshold: 0.5,
            output_interval: Some(0.1),
            acceleration_threshold: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> FlightState {
        let mut state = FlightState::new();
        state.time = 5.0;
        state.position = federated_rocket_math::vector::Vector3D::new(100.0, 50.0, 500.0);
        state.velocity = federated_rocket_math::vector::Vector3D::new(20.0, 0.0, 100.0);
        state.mach = 0.3;
        state
    }

    #[test]
    fn test_flight_event_creation() {
        let state = make_test_state();
        let event = FlightEvent::new(FlightEventType::Apogee, &state, 9.81);
        let expected_speed = (20.0_f64 * 20.0 + 100.0 * 100.0).sqrt(); // vx=20, vy=0, vz=100
        assert_eq!(event.event_type, FlightEventType::Apogee);
        assert!((event.time - 5.0).abs() < 1e-12);
        assert!((event.altitude - 500.0).abs() < 1e-12);
        assert!((event.velocity - expected_speed).abs() < 1e-9);
        assert!((event.acceleration - 9.81).abs() < 1e-12);
        assert_eq!(event.mach, 0.3);
    }

    #[test]
    fn test_flight_event_launch_description() {
        let state = FlightState::new();
        let event = FlightEvent::new(FlightEventType::Launch, &state, 0.0);
        assert_eq!(event.description, "Rocket launched");
    }

    #[test]
    fn test_flight_event_with_description() {
        let state = make_test_state();
        let event = FlightEvent::with_description(
            FlightEventType::Error,
            &state,
            0.0,
            "Simulation divergence detected".to_string(),
        );
        assert_eq!(event.event_type, FlightEventType::Error);
        assert_eq!(event.description, "Simulation divergence detected");
    }

    #[test]
    fn test_event_config_defaults() {
        let config = EventConfig::default();
        assert!((config.launch_rod_clear_altitude - 2.0).abs() < 1e-12);
        assert!((config.max_simulation_time - 120.0).abs() < 1e-12);
        assert!(config.main_deployment_altitude.is_none());
        assert!((config.ground_altitude - 0.0).abs() < 1e-12);
        assert!((config.output_interval.unwrap_or(0.1) - 0.1).abs() < 1e-12);
    }

    #[test]
    fn test_flight_event_serde_roundtrip() {
        let state = make_test_state();
        let event = FlightEvent::new(FlightEventType::MachTransition, &state, 50.0);
        let json = serde_json::to_string(&event).unwrap();
        let deser: FlightEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.event_type, FlightEventType::MachTransition);
        assert!((deser.time - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_event_config_serde_roundtrip() {
        let config = EventConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deser: EventConfig = serde_json::from_str(&json).unwrap();
        assert!((deser.launch_rod_clear_altitude - 2.0).abs() < 1e-12);
    }
}