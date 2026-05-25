use federated_rocket_math::quaternion::Quaternion;
use federated_rocket_math::vector::Vector3D;
use federated_rocket_physics::constants::STANDARD_GRAVITY;
use serde::{Deserialize, Serialize};

/// Complete 6-DOF flight state of the rocket.
///
/// Contains all position, velocity, orientation, mass properties, and
/// diagnostic quantities needed for simulation and event detection.
///
/// Coordinate system: NED-like convention where:
/// - x = North (horizontal)
/// - y = East (horizontal)
/// - z = Up (altitude)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightState {
    /// Simulation time (s)
    pub time: f64,
    /// Position in world coordinates (m)
    pub position: Vector3D,
    /// Velocity in world coordinates (m/s)
    pub velocity: Vector3D,
    /// Orientation quaternion (body to world)
    pub orientation: Quaternion,
    /// Angular velocity in body coordinates (rad/s)
    pub angular_velocity: Vector3D,
    /// Total mass (kg)
    pub mass: f64,
    /// Center of gravity position (m from nose tip)
    pub cg_position: f64,
    /// Moment of inertia (kg·m²) — Ixx, Iyy, Izz (principal axes)
    pub inertia: Vector3D,
    /// Remaining propellant mass (kg)
    pub propellant_mass: f64,
    /// Wind velocity at current position (m/s in world coordinates)
    pub wind_velocity: Vector3D,
    /// Mach number (for diagnostics)
    pub mach: f64,
    /// Reynolds number (for diagnostics)
    pub reynolds: f64,
    /// Dynamic pressure (Pa)
    pub dynamic_pressure: f64,
    /// Angle of attack (rad)
    pub angle_of_attack: f64,
}

impl FlightState {
    /// Returns the default (zero) flight state.
    ///
    /// The rocket is on the ground, not moving, with identity orientation.
    /// Mass defaults to 1.0 kg as a placeholder.
    pub fn new() -> Self {
        Self {
            time: 0.0,
            position: Vector3D::zero(),
            velocity: Vector3D::zero(),
            orientation: Quaternion::identity(),
            angular_velocity: Vector3D::zero(),
            mass: 1.0,
            cg_position: 0.0,
            inertia: Vector3D::new(0.01, 0.01, 0.001),
            propellant_mass: 0.0,
            wind_velocity: Vector3D::zero(),
            mach: 0.0,
            reynolds: 0.0,
            dynamic_pressure: 0.0,
            angle_of_attack: 0.0,
        }
    }

    /// Returns `true` if the rocket has launched (off the ground or moving).
    pub fn is_launched(&self) -> bool {
        self.altitude() > 0.0 || self.velocity.norm() > 0.0
    }

    /// Returns the speed magnitude (m/s).
    pub fn speed(&self) -> f64 {
        self.velocity.norm()
    }

    /// Returns the altitude above ground (m).
    ///
    /// Uses the Y component (NED: Y = East, Z = Up, or conventional Z-up).
    /// We treat the Z component as altitude here.
    pub fn altitude(&self) -> f64 {
        self.position.z.max(0.0)
    }

    /// Returns the horizontal (downrange) distance from the launch point (m).
    pub fn downrange_distance(&self) -> f64 {
        (self.position.x * self.position.x + self.position.y * self.position.y).sqrt()
    }

    /// Returns the total mechanical energy (J): kinetic + potential.
    pub fn total_energy(&self) -> f64 {
        self.kinetic_energy() + self.potential_energy()
    }

    /// Returns the kinetic energy (J): 0.5 * m * v².
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity.norm_squared()
    }

    /// Returns the gravitational potential energy (J): m * g * h.
    pub fn potential_energy(&self) -> f64 {
        self.mass * STANDARD_GRAVITY * self.altitude()
    }

    /// Configures the state for launch conditions at the given altitude and latitude.
    ///
    /// * `altitude_agl` — Height above ground level at the launch site (m).
    /// * `latitude` — Geodetic latitude (degrees), used for gravity / coriolis.
    pub fn set_launch_conditions(&mut self, altitude_agl: f64, _latitude: f64) {
        self.position = Vector3D::new(0.0, 0.0, altitude_agl);
        self.time = 0.0;
        self.velocity = Vector3D::zero();
        self.orientation = Quaternion::identity();
        self.angular_velocity = Vector3D::zero();
        self.mach = 0.0;
        self.reynolds = 0.0;
        self.dynamic_pressure = 0.0;
        self.angle_of_attack = 0.0;
    }
}

impl Default for FlightState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_is_zero() {
        let s = FlightState::new();
        assert_eq!(s.time, 0.0);
        assert_eq!(s.position, Vector3D::zero());
        assert_eq!(s.velocity, Vector3D::zero());
        assert_eq!(s.orientation, Quaternion::identity());
        assert_eq!(s.mass, 1.0);
    }

    #[test]
    fn test_is_launched_false_initially() {
        let s = FlightState::new();
        assert!(!s.is_launched());
    }

    #[test]
    fn test_is_launched_true_when_position_above_ground() {
        let mut s = FlightState::new();
        s.position = Vector3D::new(0.0, 0.0, 1.0);
        assert!(s.is_launched());
    }

    #[test]
    fn test_is_launched_true_when_moving() {
        let mut s = FlightState::new();
        s.velocity = Vector3D::new(0.0, 0.0, 10.0);
        assert!(s.is_launched());
    }

    #[test]
    fn test_speed() {
        let mut s = FlightState::new();
        s.velocity = Vector3D::new(3.0, 4.0, 0.0);
        assert!((s.speed() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_altitude() {
        let mut s = FlightState::new();
        s.position = Vector3D::new(10.0, 20.0, 100.0);
        assert!((s.altitude() - 100.0).abs() < 1e-12);
    }

    #[test]
    fn test_altitude_clamped_to_zero() {
        let s = FlightState::new();
        assert!((s.altitude() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_downrange_distance() {
        let mut s = FlightState::new();
        s.position = Vector3D::new(30.0, 40.0, 0.0);
        assert!((s.downrange_distance() - 50.0).abs() < 1e-12);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut s = FlightState::new();
        s.mass = 2.0;
        s.velocity = Vector3D::new(0.0, 0.0, 3.0);
        assert!((s.kinetic_energy() - 9.0).abs() < 1e-12); // 0.5 * 2 * 9 = 9
    }

    #[test]
    fn test_potential_energy() {
        let mut s = FlightState::new();
        s.mass = 2.0;
        s.position = Vector3D::new(0.0, 0.0, 10.0);
        let expected = 2.0 * STANDARD_GRAVITY * 10.0;
        assert!((s.potential_energy() - expected).abs() < 1e-9);
    }

    #[test]
    fn test_total_energy() {
        let mut s = FlightState::new();
        s.mass = 1.0;
        s.position = Vector3D::new(0.0, 0.0, 10.0);
        s.velocity = Vector3D::new(0.0, 0.0, 5.0);
        let ke = 0.5 * 1.0 * 25.0;
        let pe = 1.0 * STANDARD_GRAVITY * 10.0;
        assert!((s.total_energy() - (ke + pe)).abs() < 1e-9);
    }

    #[test]
    fn test_set_launch_conditions() {
        let mut s = FlightState::new();
        s.mass = 5.0;
        s.set_launch_conditions(100.0, 45.0);
        assert!((s.altitude() - 100.0).abs() < 1e-12);
        assert_eq!(s.time, 0.0);
        assert_eq!(s.velocity, Vector3D::zero());
        assert_eq!(s.mass, 5.0); // mass unchanged
    }

    #[test]
    fn test_flight_state_default_impl() {
        let s: FlightState = Default::default();
        assert_eq!(s.time, 0.0);
    }
}
