use serde::{Deserialize, Serialize};

/// A single data point from the reference trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencePoint {
    pub time: f64,
    pub altitude: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub mach: f64,
    pub drag_force: f64,
    pub lift_force: f64,
    pub pitch_moment: f64,
    pub cp_position: f64,
    pub cg_position: f64,
    pub mass: f64,
    pub dynamic_pressure: f64,
    pub angle_of_attack: f64,
    pub stability_margin: f64,
}

/// Complete reference simulation result from OpenRocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSimulation {
    pub name: String,
    pub description: String,
    pub motor_designation: String,
    pub max_altitude: f64,
    pub max_velocity: f64,
    pub max_acceleration: f64,
    pub flight_time: f64,
    pub apogee_time: f64,
    pub burnout_time: f64,
    pub launch_rod_velocity: f64,
    pub stability_margin: f64,
    pub trajectory: Vec<ReferencePoint>,
    pub events: Vec<ReferenceEvent>,
}

/// Reference flight event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceEvent {
    pub time: f64,
    pub event_type: String,
    pub altitude: f64,
    pub velocity: f64,
}

/// Comparison tolerance configuration
#[derive(Debug, Clone)]
pub struct ValidationTolerances {
    /// Maximum relative error for altitude (%)
    pub altitude_tolerance: f64,
    /// Maximum relative error for velocity (%)
    pub velocity_tolerance: f64,
    /// Maximum relative error for acceleration (%)
    pub acceleration_tolerance: f64,
    /// Maximum absolute error for Mach number
    pub mach_tolerance: f64,
    /// Maximum absolute error for CP position (m)
    pub cp_tolerance: f64,
    /// Maximum absolute error for event times (s)
    pub event_time_tolerance: f64,
    /// Maximum absolute error for stability margin (calibers)
    pub stability_tolerance: f64,
}

impl Default for ValidationTolerances {
    fn default() -> Self {
        Self {
            altitude_tolerance: 0.1,      // 0.1%
            velocity_tolerance: 0.1,      // 0.1%
            acceleration_tolerance: 0.5,  // 0.5%
            mach_tolerance: 0.01,
            cp_tolerance: 0.005,          // 5mm
            event_time_tolerance: 0.01,   // 10ms
            stability_tolerance: 0.1,     // 0.1 calibers
        }
    }
}
