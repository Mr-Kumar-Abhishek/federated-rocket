use std::fmt;
use std::sync::Mutex;

use federated_rocket_math::Vector3D;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// WindState
// ---------------------------------------------------------------------------

/// The state of the wind at a given point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindState {
    /// Wind velocity vector (m/s) in world coordinates (x=North, y=East, z=Up).
    pub velocity: Vector3D,
    /// Turbulence intensity (0–1).
    pub turbulence_intensity: f64,
    /// Wind speed (magnitude, m/s).
    pub speed: f64,
    /// Wind direction as azimuth (degrees, 0=North, 90=East).
    pub direction_azimuth: f64,
}

impl WindState {
    /// Create a new wind state from a speed and direction.
    ///
    /// The velocity vector is computed as:
    ///   x (North) = speed · cos(azimuth)
    ///   y (East)  = speed · sin(azimuth)
    ///   z (Up)    = 0
    fn from_speed_direction(speed: f64, direction_azimuth_deg: f64) -> Self {
        let az_rad = direction_azimuth_deg.to_radians();
        let x = speed * az_rad.cos();
        let y = speed * az_rad.sin();
        Self {
            velocity: Vector3D::new(x, y, 0.0),
            turbulence_intensity: 0.0,
            speed,
            direction_azimuth: direction_azimuth_deg,
        }
    }

    /// Create a zero-wind state.
    fn zero() -> Self {
        Self {
            velocity: Vector3D::zero(),
            turbulence_intensity: 0.0,
            speed: 0.0,
            direction_azimuth: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// WindModel trait
// ---------------------------------------------------------------------------

/// Trait for wind models (Strategy pattern).
pub trait WindModel: Send + Sync {
    /// Compute wind at a given position and altitude.
    ///
    /// * `position` - Position vector in world coordinates (x=North, y=East, z=Up).
    /// * `altitude` - Altitude above sea level (m).
    fn wind_at_position(&self, position: Vector3D, altitude: f64) -> WindState;

    /// Human-readable name of the model.
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// NoWind
// ---------------------------------------------------------------------------

/// Zero wind everywhere.
#[derive(Debug, Clone)]
pub struct NoWind;

impl NoWind {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoWind {
    fn default() -> Self {
        Self::new()
    }
}

impl WindModel for NoWind {
    fn wind_at_position(&self, _position: Vector3D, _altitude: f64) -> WindState {
        WindState::zero()
    }

    fn name(&self) -> &'static str {
        "No Wind"
    }
}

// ---------------------------------------------------------------------------
// ConstantWind
// ---------------------------------------------------------------------------

/// Constant wind with fixed speed and direction at all positions / altitudes.
#[derive(Debug, Clone)]
pub struct ConstantWind {
    speed: f64,
    direction_azimuth: f64,
}

impl ConstantWind {
    /// Create a new constant wind.
    ///
    /// * `speed` - Wind speed in m/s.
    /// * `direction_azimuth` - Direction as azimuth in degrees (0 = North, 90 = East).
    pub fn new(speed: f64, direction_azimuth: f64) -> Self {
        Self {
            speed,
            direction_azimuth,
        }
    }
}

impl WindModel for ConstantWind {
    fn wind_at_position(&self, _position: Vector3D, _altitude: f64) -> WindState {
        WindState::from_speed_direction(self.speed, self.direction_azimuth)
    }

    fn name(&self) -> &'static str {
        "Constant Wind"
    }
}

// ---------------------------------------------------------------------------
// PowerLawWind
// ---------------------------------------------------------------------------

/// Wind speed varying with altitude using the power law:
///
///   V(h) = V_ref · (h / h_ref)^α
///
/// Below a configurable minimum height, the speed is linearly interpolated to
/// zero at the ground.
#[derive(Debug, Clone)]
pub struct PowerLawWind {
    reference_speed: f64,
    reference_height: f64,
    direction_azimuth: f64,
    exponent: f64,
    /// Altitude below which linear interpolation to zero is applied.
    min_height: f64,
}

impl PowerLawWind {
    /// Create a new power-law wind model.
    ///
    /// * `reference_speed` - Wind speed (m/s) at the reference height.
    /// * `reference_height` - Reference height (m).
    /// * `direction_azimuth` - Wind direction as azimuth (degrees).
    /// * `exponent` - Power-law exponent α (typically 0.14–0.25).
    pub fn new(
        reference_speed: f64,
        reference_height: f64,
        direction_azimuth: f64,
        exponent: f64,
    ) -> Self {
        // Use 2 m as the minimum height for linear ramping by default
        let min_height = 2.0;
        Self {
            reference_speed,
            reference_height,
            direction_azimuth,
            exponent,
            min_height,
        }
    }

    /// Compute the wind speed at a given altitude using the power law.
    fn speed_at_altitude(&self, altitude: f64) -> f64 {
        let h = altitude.max(0.0);
        if h <= 0.0 {
            return 0.0;
        }

        let v_ref = self.reference_speed;
        let h_ref = self.reference_height;

        if h >= self.min_height {
            v_ref * (h / h_ref).powf(self.exponent)
        } else {
            // Linear interpolation from ground (zero) to min_height
            let v_at_min = v_ref * (self.min_height / h_ref).powf(self.exponent);
            v_at_min * (h / self.min_height)
        }
    }
}

impl WindModel for PowerLawWind {
    fn wind_at_position(&self, _position: Vector3D, altitude: f64) -> WindState {
        let speed = self.speed_at_altitude(altitude);
        WindState::from_speed_direction(speed, self.direction_azimuth)
    }

    fn name(&self) -> &'static str {
        "Power-Law Wind Profile"
    }
}

// ---------------------------------------------------------------------------
// LogarithmicWind
// ---------------------------------------------------------------------------

/// Wind speed varying with altitude using the logarithmic law:
///
///   V(h) = (u_* / k) · ln(h / z₀)
///
/// where:
/// * `u_*` = friction velocity (m/s)
/// * `k` = von Karman constant ≈ 0.41
/// * `z₀` = roughness length (m)
///
/// For h < z₀ the wind speed is forced to zero.
#[derive(Debug, Clone)]
pub struct LogarithmicWind {
    friction_velocity: f64,
    roughness_length: f64,
    direction_azimuth: f64,
    /// von Karman constant
    von_karman: f64,
}

impl LogarithmicWind {
    /// Create a new logarithmic wind model.
    ///
    /// * `friction_velocity` - Friction velocity u_* (m/s).
    /// * `roughness_length` - Surface roughness length z₀ (m).
    /// * `direction_azimuth` - Wind direction as azimuth (degrees).
    pub fn new(friction_velocity: f64, roughness_length: f64, direction_azimuth: f64) -> Self {
        Self {
            friction_velocity,
            roughness_length,
            direction_azimuth,
            von_karman: 0.41,
        }
    }

    /// Compute the wind speed at a given altitude.
    fn speed_at_altitude(&self, altitude: f64) -> f64 {
        let h = altitude.max(0.0);
        let z0 = self.roughness_length;

        if h <= z0 || z0 <= 0.0 {
            return 0.0;
        }

        (self.friction_velocity / self.von_karman) * (h / z0).ln()
    }
}

impl WindModel for LogarithmicWind {
    fn wind_at_position(&self, _position: Vector3D, altitude: f64) -> WindState {
        let speed = self.speed_at_altitude(altitude);
        WindState::from_speed_direction(speed, self.direction_azimuth)
    }

    fn name(&self) -> &'static str {
        "Logarithmic Wind Profile"
    }
}

// ---------------------------------------------------------------------------
// WindGust
// ---------------------------------------------------------------------------

/// Adds a sinusoidal gust component to any base wind model.
///
/// The instantaneous wind is:
///   v(t) = v_base + gust_speed · sin(2π · gust_frequency · (t - gust_start_time))
///
/// for `t > gust_start_time`.  Before `gust_start_time` no gust component is
/// added.
///
/// The gust is applied isotropically (in the direction of the base wind).
pub struct WindGust {
    base_model: Box<dyn WindModel>,
    gust_speed: f64,
    gust_frequency: f64,
    gust_start_time: f64,
    /// Elapsed simulation time (seconds). Uses interior mutability so
    /// [`WindModel::wind_at_position`] (which takes `&self`) can access it.
    current_time: Mutex<f64>,
}

impl fmt::Debug for WindGust {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindGust")
            .field("base_model", &self.base_model.name())
            .field("gust_speed", &self.gust_speed)
            .field("gust_frequency", &self.gust_frequency)
            .field("gust_start_time", &self.gust_start_time)
            .field("current_time", &self.current_time)
            .finish()
    }
}

impl WindGust {
    /// Create a new gust wind model.
    ///
    /// * `base_model` - The underlying wind model to augment.
    /// * `gust_speed` - Amplitude of the gust component (m/s).
    /// * `gust_frequency` - Frequency of the gust oscillation (Hz).
    /// * `gust_start_time` - Time at which the gust begins (s).
    pub fn new(
        base_model: Box<dyn WindModel>,
        gust_speed: f64,
        gust_frequency: f64,
        gust_start_time: f64,
    ) -> Self {
        Self {
            base_model,
            gust_speed,
            gust_frequency,
            gust_start_time,
            current_time: Mutex::new(0.0),
        }
    }

    /// Advance the simulation time by `dt` seconds.
    ///
    /// Call this each integration step to keep the gust in sync with the
    /// simulation clock.
    pub fn advance_time(&self, dt: f64) {
        let mut t = self.current_time.lock().unwrap();
        *t += dt;
    }

    /// Reset the simulation time to zero.
    pub fn reset_time(&self) {
        let mut t = self.current_time.lock().unwrap();
        *t = 0.0;
    }

    /// Set the current simulation time explicitly.
    pub fn set_time(&self, t: f64) {
        let mut current = self.current_time.lock().unwrap();
        *current = t;
    }

    /// Compute the gust magnitude at the current simulation time.
    fn gust_magnitude(&self) -> f64 {
        let t = *self.current_time.lock().unwrap();
        if t <= self.gust_start_time {
            return 0.0;
        }
        let phase = 2.0 * std::f64::consts::PI * self.gust_frequency * (t - self.gust_start_time);
        self.gust_speed * phase.sin()
    }
}

impl WindModel for WindGust {
    fn wind_at_position(&self, position: Vector3D, altitude: f64) -> WindState {
        let base = self.base_model.wind_at_position(position, altitude);
        let gust_mag = self.gust_magnitude();

        if gust_mag == 0.0 {
            return base;
        }

        // Add the gust component in the direction of the base wind.
        let new_speed = base.speed + gust_mag;
        WindState::from_speed_direction(new_speed.max(0.0), base.direction_azimuth)
    }

    fn name(&self) -> &'static str {
        "Wind Gust Model"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- NoWind ---

    #[test]
    fn test_no_wind() {
        let model = NoWind::new();
        let wind = model.wind_at_position(Vector3D::new(0.0, 0.0, 0.0), 1000.0);
        assert_eq!(wind.speed, 0.0);
        assert_eq!(wind.velocity, Vector3D::zero());
    }

    // --- ConstantWind ---

    #[test]
    fn test_constant_wind_north() {
        let model = ConstantWind::new(10.0, 0.0);
        let wind = model.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 10.0).abs() < 1e-10);
        assert!((wind.velocity.x - 10.0).abs() < 1e-10); // North
        assert!((wind.velocity.y - 0.0).abs() < 1e-10);  // East
        assert!((wind.direction_azimuth - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_constant_wind_east() {
        let model = ConstantWind::new(10.0, 90.0);
        let wind = model.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 10.0).abs() < 1e-10);
        assert!((wind.velocity.x - 0.0).abs() < 1e-10);  // North
        assert!((wind.velocity.y - 10.0).abs() < 1e-10); // East
    }

    #[test]
    fn test_constant_wind_northeast() {
        let model = ConstantWind::new(10.0, 45.0);
        let wind = model.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 10.0).abs() < 1e-10);
        let expected_comp = 10.0 * (45.0_f64).to_radians().cos();
        assert!((wind.velocity.x - expected_comp).abs() < 1e-10);
    }

    #[test]
    fn test_constant_wind_altitude_independent() {
        let model = ConstantWind::new(5.0, 180.0);
        let wind0 = model.wind_at_position(Vector3D::zero(), 0.0);
        let wind10k = model.wind_at_position(Vector3D::zero(), 10_000.0);
        assert!((wind0.speed - wind10k.speed).abs() < 1e-10);
    }

    // --- PowerLawWind ---

    #[test]
    fn test_power_law_at_reference_height() {
        let model = PowerLawWind::new(10.0, 10.0, 0.0, 0.143);
        let wind = model.wind_at_position(Vector3D::zero(), 10.0);
        assert!((wind.speed - 10.0).abs() < 1e-6, "V = {}", wind.speed);
    }

    #[test]
    fn test_power_law_increases_with_height() {
        let model = PowerLawWind::new(10.0, 10.0, 0.0, 0.2);
        let wind_lo = model.wind_at_position(Vector3D::zero(), 5.0);
        let wind_hi = model.wind_at_position(Vector3D::zero(), 50.0);
        assert!(wind_hi.speed > wind_lo.speed, "Wind should increase with height");
    }

    #[test]
    fn test_power_law_zero_at_ground() {
        let model = PowerLawWind::new(10.0, 10.0, 0.0, 0.2);
        let wind = model.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 0.0).abs() < 1e-10, "Wind at ground should be zero");
    }

    #[test]
    fn test_power_law_linear_interpolation_below_min_height() {
        let model = PowerLawWind::new(10.0, 10.0, 0.0, 0.2);
        let wind_1m = model.wind_at_position(Vector3D::zero(), 1.0);
        // At 1 m, should be roughly half of the value at 2 m
        assert!(wind_1m.speed > 0.0, "Wind at 1m should be positive");
        let wind_2m = model.wind_at_position(Vector3D::zero(), 2.0);
        assert!(wind_1m.speed < wind_2m.speed, "Wind at 1m should be less than at 2m");
    }

    // --- LogarithmicWind ---

    #[test]
    fn test_log_wind_positive_at_10m() {
        let model = LogarithmicWind::new(0.5, 0.05, 0.0);
        let wind = model.wind_at_position(Vector3D::zero(), 10.0);
        // V = (0.5/0.41) * ln(10/0.05) = 1.2195 * ln(200) = 1.2195 * 5.298 = 6.46 m/s
        assert!(wind.speed > 5.0, "V = {}", wind.speed);
        assert!(wind.speed < 8.0, "V = {}", wind.speed);
    }

    #[test]
    fn test_log_wind_zero_below_roughness() {
        let model = LogarithmicWind::new(0.5, 0.1, 0.0);
        let wind = model.wind_at_position(Vector3D::zero(), 0.05);
        assert!((wind.speed - 0.0).abs() < 1e-10, "Wind below z₀ should be zero");
    }

    #[test]
    fn test_log_wind_increases_with_height() {
        let model = LogarithmicWind::new(0.3, 0.03, 90.0);
        let wind_lo = model.wind_at_position(Vector3D::zero(), 5.0);
        let wind_hi = model.wind_at_position(Vector3D::zero(), 50.0);
        assert!(wind_hi.speed > wind_lo.speed, "Wind should increase with height");
    }

    #[test]
    fn test_log_wind_zero_roughness() {
        let model = LogarithmicWind::new(0.5, 0.0, 0.0);
        let wind = model.wind_at_position(Vector3D::zero(), 10.0);
        assert!((wind.speed - 0.0).abs() < 1e-10, "Zero roughness should give zero wind");
    }

    // --- WindGust ---

    #[test]
    fn test_gust_before_start_time() {
        let base = Box::new(ConstantWind::new(10.0, 0.0));
        let gust = WindGust::new(base, 5.0, 1.0, 10.0);
        gust.set_time(5.0); // Before gust_start_time
        let wind = gust.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 10.0).abs() < 1e-10, "No gust before start time");
    }

    #[test]
    fn test_gust_after_start_time() {
        let base = Box::new(ConstantWind::new(10.0, 0.0));
        let gust = WindGust::new(base, 5.0, 1.0, 10.0);
        gust.set_time(10.25); // At gust_start_time + 0.25 * period
        let wind = gust.wind_at_position(Vector3D::zero(), 0.0);
        // gust = 5.0 * sin(2π * 1.0 * 0.25) = 5.0 * sin(π/2) = 5.0
        assert!((wind.speed - 15.0).abs() < 1e-6, "V = {}", wind.speed);
    }

    #[test]
    fn test_gust_oscillation() {
        let base = Box::new(ConstantWind::new(10.0, 0.0));
        let gust = WindGust::new(base, 5.0, 0.5, 0.0);

        gust.set_time(0.0);
        let w0 = gust.wind_at_position(Vector3D::zero(), 0.0);

        gust.set_time(0.5);
        let w1 = gust.wind_at_position(Vector3D::zero(), 0.0);

        gust.set_time(1.0);
        let w2 = gust.wind_at_position(Vector3D::zero(), 0.0);

        // At t=0: sin(0) = 0 => speed = 10
        assert!((w0.speed - 10.0).abs() < 1e-6, "t=0: V = {}", w0.speed);
        // At t=0.5: sin(π/2) = 1 => speed = 15
        assert!((w1.speed - 15.0).abs() < 1e-6, "t=0.5: V = {}", w1.speed);
        // At t=1.0: sin(π) = 0 => speed = 10
        assert!((w2.speed - 10.0).abs() < 1e-6, "t=1.0: V = {}", w2.speed);
    }

    #[test]
    fn test_gust_advance_time() {
        let base = Box::new(ConstantWind::new(5.0, 90.0));
        let gust = WindGust::new(base, 3.0, 2.0, 0.0);

        gust.advance_time(0.125); // sin(2π * 2 * 0.125) = sin(π/2) = 1
        let wind = gust.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 8.0).abs() < 1e-6, "V = {}", wind.speed);
    }

    #[test]
    fn test_gust_reset_time() {
        let base = Box::new(ConstantWind::new(10.0, 0.0));
        let gust = WindGust::new(base, 5.0, 1.0, 0.0);
        gust.set_time(10.0);
        gust.reset_time();
        let wind = gust.wind_at_position(Vector3D::zero(), 0.0);
        assert!((wind.speed - 10.0).abs() < 1e-10, "After reset, no gust component");
    }

    // --- Trait dispatch ---

    #[test]
    fn test_wind_trait_dispatch() {
        let models: Vec<Box<dyn WindModel>> = vec![
            Box::new(NoWind::new()),
            Box::new(ConstantWind::new(5.0, 90.0)),
            Box::new(PowerLawWind::new(10.0, 10.0, 0.0, 0.2)),
            Box::new(LogarithmicWind::new(0.5, 0.05, 180.0)),
        ];
        for model in &models {
            let wind = model.wind_at_position(Vector3D::zero(), 10.0);
            assert!(wind.speed >= 0.0, "{} should have non-negative speed", model.name());
        }
    }
}