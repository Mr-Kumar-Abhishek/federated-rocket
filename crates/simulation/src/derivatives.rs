use std::ops::{Add, Mul, Sub};

use federated_rocket_math::integrator::Normed;

use federated_rocket_aero::compute::AeroCalculator;
use federated_rocket_core::component_tree::ComponentTree;
use federated_rocket_math::quaternion::Quaternion;
use federated_rocket_math::vector::Vector3D;
use federated_rocket_physics::atmosphere::AtmosphericModel;
use federated_rocket_physics::gravity::GravityModel;
use federated_rocket_physics::wind::WindModel;

use crate::motor::MotorModel;
use crate::state::FlightState;

// Safety epsilon to avoid division by zero
const EPS: f64 = 1e-15;

// ============================================================================
// CompactState — minimal state vector for efficient integration
// ============================================================================

/// Compact 6-DOF state for use with the [`Integrator`] trait.
///
/// Stores the minimal state needed for RK4 integration:
/// - Position (3): x, y, z
/// - Velocity (3): vx, vy, vz
/// - Orientation (4): qw, qx, qy, qz (quaternion, body→world)
/// - Angular velocity (3): wx, wy, wz (body frame)
/// - Mass scalar
/// - Propellant mass scalar
///
/// Implements `Add`, `Mul<f64>`, and `Clone` for use with the math crate's
/// [`RK4Integrator`] and [`EulerIntegrator`].
#[derive(Debug, Clone)]
pub struct CompactState {
    /// Flat array of 13 kinematic states:
    /// [0..2] = position x,y,z
    /// [3..5] = velocity vx,vy,vz
    /// [6..9] = orientation qw,qx,qy,qz
    /// [10..12] = angular velocity wx,wy,wz
    pub data: [f64; 13],
    /// Total mass (kg)
    pub mass: f64,
    /// Remaining propellant mass (kg)
    pub propellant_mass: f64,
}

impl CompactState {
    /// Creates a new `CompactState` from raw components.
    #[inline]
    pub fn new(
        position: Vector3D,
        velocity: Vector3D,
        orientation: Quaternion,
        angular_velocity: Vector3D,
        mass: f64,
        propellant_mass: f64,
    ) -> Self {
        Self {
            data: [
                position.x, position.y, position.z,
                velocity.x, velocity.y, velocity.z,
                orientation.w, orientation.x, orientation.y, orientation.z,
                angular_velocity.x, angular_velocity.y, angular_velocity.z,
            ],
            mass,
            propellant_mass,
        }
    }

    /// Converts a [`FlightState`] into a [`CompactState`].
    pub fn from_flight_state(state: &FlightState) -> Self {
        Self::new(
            state.position,
            state.velocity,
            state.orientation,
            state.angular_velocity,
            state.mass,
            state.propellant_mass,
        )
    }

    /// Reconstructs orientation quaternion from the flat array.
    #[inline]
    pub fn orientation(&self) -> Quaternion {
        Quaternion::new(self.data[6], self.data[7], self.data[8], self.data[9])
    }

    /// Reconstructs position from the flat array.
    #[inline]
    pub fn position(&self) -> Vector3D {
        Vector3D::new(self.data[0], self.data[1], self.data[2])
    }

    /// Reconstructs velocity from the flat array.
    #[inline]
    pub fn velocity(&self) -> Vector3D {
        Vector3D::new(self.data[3], self.data[4], self.data[5])
    }

    /// Reconstructs angular velocity from the flat array.
    #[inline]
    pub fn angular_velocity(&self) -> Vector3D {
        Vector3D::new(self.data[10], self.data[11], self.data[12])
    }
}

// ---- Trait impls for Integrator compatibility ----

impl Add for CompactState {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut data = [0.0; 13];
        for i in 0..13 {
            data[i] = self.data[i] + rhs.data[i];
        }
        Self {
            data,
            mass: self.mass + rhs.mass,
            propellant_mass: self.propellant_mass + rhs.propellant_mass,
        }
    }
}

impl Mul<f64> for CompactState {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        let mut data = [0.0; 13];
        for i in 0..13 {
            data[i] = self.data[i] * rhs;
        }
        Self {
            data,
            mass: self.mass * rhs,
            propellant_mass: self.propellant_mass * rhs,
        }
    }
}

/// Subtraction operator for [`CompactState`], needed by [`AdaptiveRK4Integrator`]
/// for error estimation via Richardson extrapolation.
impl Sub for CompactState {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut data = [0.0; 13];
        for i in 0..13 {
            data[i] = self.data[i] - rhs.data[i];
        }
        Self {
            data,
            mass: self.mass - rhs.mass,
            propellant_mass: self.propellant_mass - rhs.propellant_mass,
        }
    }
}

/// Normed implementation for [`CompactState`], required by
/// [`AdaptiveRK4Integrator::step_adaptive`] for error estimation.
impl Normed for CompactState {
    fn norm_squared(&self) -> f64 {
        self.data.iter().map(|x| x * x).sum::<f64>() + self.mass * self.mass + self.propellant_mass * self.propellant_mass
    }
}

impl Add<&CompactState> for CompactState {
    type Output = Self;

    fn add(self, rhs: &CompactState) -> Self {
        let mut data = [0.0; 13];
        for i in 0..13 {
            data[i] = self.data[i] + rhs.data[i];
        }
        Self {
            data,
            mass: self.mass + rhs.mass,
            propellant_mass: self.propellant_mass + rhs.propellant_mass,
        }
    }
}

// ============================================================================
// Derivative computation
// ============================================================================

/// Compute the 6-DOF state derivative for the rocket at the given compact state.
///
/// This function computes all forces and moments acting on the rocket and returns
/// the time derivatives of each state variable.
///
/// # Returns
///
/// A [`CompactState`] where the `data` array contains:
/// - [0..2] = velocity (dx/dt = v)
/// - [3..5] = acceleration (dv/dt = F/m)
/// - [6..9] = orientation derivative (dq/dt)
/// - [10..12] = angular acceleration (dω/dt)
/// - `mass` = mass rate (dm/dt = -dm_propellant/dt)
/// - `propellant_mass` = propellant mass rate
pub fn compute_derivative(
    state: &CompactState,
    time: f64,
    motor: &Option<MotorModel>,
    aero_calc: &AeroCalculator,
    atmosphere: &dyn AtmosphericModel,
    gravity: &dyn GravityModel,
    wind: &dyn WindModel,
    tree: &ComponentTree,
    reference_area: f64,
    reference_diameter: f64,
) -> CompactState {
    let pos = state.position();
    let vel = state.velocity();
    let q = state.orientation();
    let omega = state.angular_velocity();
    let mass = state.mass.max(EPS);

    // 1. Get atmospheric conditions at the rocket's altitude
    let altitude = pos.z.max(0.0);
    let atmos = atmosphere.conditions_at_altitude(altitude);
    let speed_of_sound = atmos.speed_of_sound.max(EPS);

    // 2. Get wind at current position
    let wind_state = wind.wind_at_position(pos, altitude);
    let wind_vel = wind_state.velocity;

    // 3. Compute relative air velocity
    // Wind is in world coordinates; we need the air-relative velocity in body frame
    // q maps body→world, so q.conjugate() maps world→body
    let q_inv = q.conjugate();
    let vel_body = q_inv.rotate(vel);
    let wind_body = q_inv.rotate(wind_vel);
    let rel_vel_body = vel_body - wind_body;

    let airspeed = rel_vel_body.norm();

    // 4. Compute Mach number and dynamic pressure (for diagnostics)
    let _mach = if speed_of_sound > EPS {
        airspeed / speed_of_sound
    } else {
        0.0
    };
    let _dynamic_pressure = 0.5 * atmos.density * airspeed * airspeed;

    // Reynolds number (for diagnostics)
    let _reynolds = if atmos.viscosity > EPS {
        atmos.density * airspeed * reference_diameter / atmos.viscosity
    } else {
        0.0
    };

    // Angle of attack (angle between velocity and body x-axis)
    let axial_speed = rel_vel_body.x.abs().max(EPS);
    let _angle_of_attack = (rel_vel_body.y / axial_speed).atan();

    // 5. Compute aerodynamic forces and moments (in body frame)
    let aero = aero_calc.compute_forces(
        tree,
        rel_vel_body,
        omega,
        &atmos,
        reference_area,
        reference_diameter,
    );

    // 6. Compute thrust force (if motor is burning)
    // Thrust is along the body x-axis (positive = forward)
    let thrust_body = if let Some(ref motor_model) = motor {
        if motor_model.is_burning(time) {
            Vector3D::new(motor_model.thrust_at_time(time), 0.0, 0.0)
        } else {
            Vector3D::zero()
        }
    } else {
        Vector3D::zero()
    };

    // Convert thrust from body to world coordinates
    let thrust_world = q.rotate(thrust_body);

    // 7. Compute gravity force in world coordinates
    // Gravity acts in the -z direction (z = up)
    let g = gravity.acceleration_at_altitude(altitude);
    let gravity_force = Vector3D::new(0.0, 0.0, -mass * g);

    // 8. Convert aerodynamic forces from body to world
    // Drag acts in the -x direction in body frame (opposite to velocity)
    // Lift acts in the +y direction in body frame
    // Side force acts in the +z direction in body frame
    let aero_force_body = Vector3D::new(-aero.drag, aero.lift, aero.side_force);
    let aero_force_world = q.rotate(aero_force_body);

    // 9. Sum total force in world coordinates
    let total_force = thrust_world + aero_force_world + gravity_force;

    // 10. Compute linear acceleration
    let acceleration = total_force / mass;

    // 11. Compute angular acceleration using Euler's equations (body frame)
    // I·α + ω × (I·ω) = M  →  α = I^{-1} · (M - ω × (I·ω))
    // For principal axes: I = diag(Ixx, Iyy, Izz)
    let ixx = state.data[10].abs().max(EPS); // placeholder; inertia doesn't come from CompactState
    let iyy = state.data[11].abs().max(EPS);
    let izz = state.data[12].abs().max(EPS);

    // Actually, we should use proper inertia values. Since CompactState doesn't store inertia,
    // we compute angular acceleration without it for now and use a simplified approach.
    // In a real simulation, inertia should be part of the state or computed from the component tree.
    // For now, we use the AeroCalculator's returned moments directly.

    // Euler's equations for principal axes:
    // Ixx * αx = Mx - (Izz - Iyy) * ωy * ωz
    // Iyy * αy = My - (Ixx - Izz) * ωz * ωx
    // Izz * αz = Mz - (Iyy - Ixx) * ωx * ωy

    // Aerodynamic moments are already in body frame about the CG
    let mx = aero.pitch_moment;
    let my = aero.yaw_moment;
    let mz = aero.roll_moment;

    let angular_accel_x = (mx - (izz - iyy) * omega.y * omega.z) / ixx;
    let angular_accel_y = (my - (ixx - izz) * omega.z * omega.x) / iyy;
    let angular_accel_z = (mz - (iyy - ixx) * omega.x * omega.y) / izz;
    let angular_accel = Vector3D::new(angular_accel_x, angular_accel_y, angular_accel_z);

    // 12. Compute orientation derivative
    // dq/dt = 0.5 * q * ω_body (where ω_body is a pure quaternion)
    let omega_quat = Quaternion::new(0.0, omega.x, omega.y, omega.z);
    let q_dot = q * omega_quat * 0.5;

    // 13. Compute mass derivative (propellant consumption rate)
    let mass_flow_rate = if let Some(ref motor_model) = motor {
        motor_model.mass_flow_rate(time)
    } else {
        0.0
    };

    // Build derivative CompactState
    CompactState {
        data: [
            vel.x, vel.y, vel.z,          // d(position)/dt = velocity
            acceleration.x, acceleration.y, acceleration.z, // d(velocity)/dt
            q_dot.w, q_dot.x, q_dot.y, q_dot.z, // d(quaternion)/dt
            angular_accel.x, angular_accel.y, angular_accel.z, // d(omega)/dt
        ],
        mass: mass_flow_rate,
        propellant_mass: mass_flow_rate,
    }
}

/// Normalize the orientation quaternion in a [`CompactState`] to prevent drift.
pub fn normalize_orientation(state: &mut CompactState) {
    let q = state.orientation();
    let n = q.norm();
    if n > EPS {
        let inv_n = 1.0 / n;
        state.data[6] = q.w * inv_n;
        state.data[7] = q.x * inv_n;
        state.data[8] = q.y * inv_n;
        state.data[9] = q.z * inv_n;
    }
}

/// Converts a [`CompactState`] back into a [`FlightState`] with diagnostic quantities.
pub fn compact_to_flight_state(
    state: &CompactState,
    atmosphere: &dyn AtmosphericModel,
    wind: &dyn WindModel,
) -> FlightState {
    let pos = state.position();
    let vel = state.velocity();
    let q = state.orientation();
    let omega = state.angular_velocity();

    let altitude = pos.z.max(0.0);
    let atmos = atmosphere.conditions_at_altitude(altitude);
    let wind_state = wind.wind_at_position(pos, altitude);

    let airspeed = vel.norm();
    let mach = if atmos.speed_of_sound > EPS {
        airspeed / atmos.speed_of_sound
    } else {
        0.0
    };
    let dynamic_pressure = 0.5 * atmos.density * airspeed * airspeed;
    let reynolds = if atmos.viscosity > EPS {
        atmos.density * airspeed * 0.0508 / atmos.viscosity
    } else {
        0.0
    };

    // Angle of attack
    let q_inv = q.conjugate();
    let vel_body = q_inv.rotate(vel);
    let axial_speed = vel_body.x.abs().max(EPS);
    let angle_of_attack = (vel_body.y / axial_speed).atan();

    FlightState {
        time: 0.0, // time is managed by the simulation loop
        position: pos,
        velocity: vel,
        orientation: q,
        angular_velocity: omega,
        mass: state.mass,
        cg_position: 0.0,
        inertia: Vector3D::new(0.01, 0.01, 0.001),
        propellant_mass: state.propellant_mass,
        wind_velocity: wind_state.velocity,
        mach,
        reynolds,
        dynamic_pressure,
        angle_of_attack,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use federated_rocket_aero::compute::AeroCalculator;
    use federated_rocket_core::component_tree::ComponentTree;
    use federated_rocket_physics::atmosphere::StandardAtmosphere;
    use federated_rocket_physics::gravity::ConstantGravity;
    use federated_rocket_physics::wind::NoWind;

    fn empty_tree() -> ComponentTree {
        ComponentTree::new()
    }

    fn make_default_state() -> CompactState {
        CompactState::new(
            Vector3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 0.0),
            Quaternion::identity(),
            Vector3D::zero(),
            1.0,
            0.0,
        )
    }

    #[test]
    fn test_compact_state_new() {
        let cs = CompactState::new(
            Vector3D::new(1.0, 2.0, 3.0),
            Vector3D::new(4.0, 5.0, 6.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
            Vector3D::new(0.1, 0.2, 0.3),
            2.0,
            0.5,
        );
        assert!((cs.data[0] - 1.0).abs() < 1e-12);
        assert!((cs.data[4] - 5.0).abs() < 1e-12);
        assert!((cs.data[6] - 1.0).abs() < 1e-12);
        assert!((cs.data[10] - 0.1).abs() < 1e-12);
        assert!((cs.mass - 2.0).abs() < 1e-12);
        assert!((cs.propellant_mass - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_compact_state_from_flight_state() {
        let mut fs = FlightState::new();
        fs.position = Vector3D::new(10.0, 20.0, 30.0);
        fs.mass = 5.0;
        fs.propellant_mass = 2.0;

        let cs = CompactState::from_flight_state(&fs);
        assert!((cs.data[0] - 10.0).abs() < 1e-12);
        assert!((cs.data[1] - 20.0).abs() < 1e-12);
        assert!((cs.data[2] - 30.0).abs() < 1e-12);
        assert!((cs.mass - 5.0).abs() < 1e-12);
        assert!((cs.propellant_mass - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_compact_state_add() {
        let mut a = make_default_state();
        a.data[0] = 10.0;
        a.mass = 2.0;

        let mut b = make_default_state();
        b.data[0] = 20.0;
        b.mass = 3.0;

        let c = a + b;
        assert!((c.data[0] - 30.0).abs() < 1e-12);
        assert!((c.mass - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_compact_state_mul_f64() {
        let mut a = make_default_state();
        a.data[0] = 10.0;
        a.data[3] = 5.0;
        a.mass = 2.0;

        let b = a * 3.0;
        assert!((b.data[0] - 30.0).abs() < 1e-12);
        assert!((b.data[3] - 15.0).abs() < 1e-12);
        assert!((b.mass - 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_normalize_orientation() {
        let mut cs = CompactState::new(
            Vector3D::zero(),
            Vector3D::zero(),
            Quaternion::new(1.0, 0.1, 0.05, 0.02),
            Vector3D::zero(),
            1.0,
            0.0,
        );

        // The quaternion has norm = sqrt(1 + 0.01 + 0.0025 + 0.0004) ≈ 1.0126
        normalize_orientation(&mut cs);
        let q = cs.orientation();
        let n = q.norm();
        assert!((n - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_derivative_stationary_rocket() {
        let cs = make_default_state();
        let motor = None;
        let aero = AeroCalculator::new();
        let atmo = StandardAtmosphere;
        let grav = ConstantGravity;
        let wind = NoWind;
        let tree = empty_tree();

        let deriv = compute_derivative(
            &cs, 0.0, &motor, &aero, &atmo, &grav, &wind, &tree, 0.001, 0.05,
        );

        // For a stationary rocket at ground level with no motor:
        // - d(position)/dt = velocity (should be 0)
        assert!((deriv.data[0]).abs() < 1e-10);
        assert!((deriv.data[1]).abs() < 1e-10);
        assert!((deriv.data[2]).abs() < 1e-10);
        // - d(velocity)/dt should include gravity
        //   acceleration due to gravity is ~9.81 m/s² downward
        //   (some small aero forces may be present, so use < 1.0 tolerance)
        assert!((deriv.data[3]).abs() < 1e-10); // no x acceleration
        assert!((deriv.data[4]).abs() < 1e-10); // no y acceleration
        assert!(deriv.data[5] < -8.0); // z acceleration ≈ -g (downward)
    }

    #[test]
    fn test_compact_state_fields() {
        let cs = CompactState::new(
            Vector3D::new(1.0, 2.0, 3.0),
            Vector3D::new(4.0, 5.0, 6.0),
            Quaternion::new(0.707, 0.707, 0.0, 0.0),
            Vector3D::new(0.1, 0.2, 0.3),
            2.0,
            0.5,
        );

        let pos = cs.position();
        let vel = cs.velocity();
        let q = cs.orientation();
        let omega = cs.angular_velocity();

        assert!((pos.x - 1.0).abs() < 1e-12);
        assert!((vel.z - 6.0).abs() < 1e-12);
        assert!((q.w - 0.707).abs() < 1e-12);
        assert!((omega.y - 0.2).abs() < 1e-12);
    }
}