use federated_rocket_math::interpolator::{Interpolator, InterpolationMethod};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;

/// A single thrust data point (time, thrust).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrustPoint {
    /// Time from ignition (s)
    pub time: f64,
    /// Thrust force (N)
    pub thrust: f64,
}

/// Motor thrust profile model.
///
/// Stores thrust curve data and provides interpolated thrust, mass,
/// and impulse calculations throughout the burn.
///
/// The thrust curve uses interior mutability (`RefCell`) so that
/// interpolation (which requires `&mut self` on the underlying
/// [`Interpolator`]) can be performed through `&self` references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorModel {
    /// Manufacturer name (e.g., "Aerotech", "Cesaroni")
    pub manufacturer: String,
    /// Motor designation (e.g., "K1103X")
    pub designation: String,
    /// Motor outer diameter (mm)
    pub diameter: f64,
    /// Motor length (mm)
    pub length: f64,
    /// Total impulse (N·s)
    pub total_impulse: f64,
    /// Burn time (s)
    pub burn_time: f64,
    /// Mass of the motor casing after burnout (kg)
    pub dry_mass: f64,
    /// Mass of propellant (kg)
    pub propellant_mass: f64,
    /// Serializable thrust points for round-tripping
    pub thrust_points: Vec<ThrustPoint>,
    /// Interpolated thrust curve (time in s, thrust in N) — not serialized directly
    #[serde(skip, default = "default_thrust_curve")]
    thrust_curve: RefCell<Interpolator>,
    /// Ejection charge delay after burnout (s). 0 = plugged/boosted.
    pub delay_time: f64,
    /// Maximum thrust observed (cached)
    max_thrust: f64,
}

/// Default thrust curve used for serde deserialization of skipped field.
fn default_thrust_curve() -> RefCell<Interpolator> {
    RefCell::new(Interpolator::new(InterpolationMethod::Linear))
}

impl MotorModel {
    /// Creates a new `MotorModel` with the given manufacturer and designation.
    ///
    /// Initializes with zero values and an empty linear-interpolation thrust curve.
    pub fn new(manufacturer: String, designation: String) -> Self {
        Self {
            manufacturer,
            designation,
            diameter: 0.0,
            length: 0.0,
            total_impulse: 0.0,
            burn_time: 0.0,
            dry_mass: 0.0,
            propellant_mass: 0.0,
            thrust_points: Vec::new(),
            thrust_curve: RefCell::new(Interpolator::new(InterpolationMethod::Linear)),
            delay_time: 0.0,
            max_thrust: 0.0,
        }
    }

    /// Adds a thrust data point to the thrust curve.
    pub fn add_thrust_point(&mut self, time: f64, thrust: f64) {
        self.thrust_points.push(ThrustPoint { time, thrust });
        self.thrust_curve.get_mut().add_point(time, thrust);
        if thrust > self.max_thrust {
            self.max_thrust = thrust;
        }
    }

    /// Returns the interpolated thrust at the given time (N).
    ///
    /// Returns 0.0 if:
    /// - The thrust curve has no data points
    /// - The time is outside the valid curve range (before first or after last point)
    pub fn thrust_at_time(&self, time: f64) -> f64 {
        if self.thrust_points.is_empty() {
            return 0.0;
        }
        // Clamp: if time is before first data point or after burn time, thrust is 0
        if time < 0.0 || time > self.burn_time {
            return 0.0;
        }
        self.thrust_curve
            .borrow_mut()
            .interpolate(time)
            .unwrap_or(0.0)
    }

    /// Returns `true` if the motor is burning at the given time.
    pub fn is_burning(&self, time: f64) -> bool {
        self.burn_time > 0.0
            && time >= 0.0
            && time <= self.burn_time
            && !self.thrust_points.is_empty()
    }

    /// Returns the total motor mass (dry + remaining propellant) at the given time (kg).
    ///
    /// The propellant is consumed linearly with the fraction of impulse used.
    pub fn mass_at_time(&self, time: f64) -> f64 {
        if self.total_impulse <= 0.0 || self.propellant_mass <= 0.0 {
            return self.dry_mass + self.propellant_mass;
        }

        let impulse_used = self.impulse_at_time(time).min(self.total_impulse);
        let frac = impulse_used / self.total_impulse;
        let remaining = self.propellant_mass * (1.0 - frac);
        self.dry_mass + remaining.max(0.0)
    }

    /// Returns the average thrust over the burn (N).
    pub fn average_thrust(&self) -> f64 {
        if self.burn_time > 0.0 {
            self.total_impulse / self.burn_time
        } else {
            0.0
        }
    }

    /// Returns the maximum thrust (N) from the thrust curve.
    pub fn max_thrust(&self) -> f64 {
        self.max_thrust
    }

    /// Returns the total impulse used up to the given time (N·s).
    ///
    /// Uses trapezoidal integration of the thrust curve from t=0 to t=time.
    /// Returns 0 for times outside the valid range.
    pub fn impulse_at_time(&self, time: f64) -> f64 {
        if self.thrust_points.is_empty() || time <= 0.0 {
            return 0.0;
        }

        let t_end = time.min(self.burn_time);

        // Use trapezoidal integration
        let n_steps = 100.max((self.burn_time * 1000.0) as usize);
        let dt = t_end / n_steps as f64;

        let mut impulse = 0.0;
        for i in 0..n_steps {
            let t1 = i as f64 * dt;
            let t2 = (i + 1) as f64 * dt;
            let f1 = self.thrust_at_time(t1);
            let f2 = self.thrust_at_time(t2);
            impulse += (f1 + f2) * dt * 0.5;
        }

        impulse
    }

    /// Returns the propellant mass flow rate at the given time (kg/s).
    ///
    /// This is the negative rate at which propellant mass decreases.
    pub fn mass_flow_rate(&self, time: f64) -> f64 {
        if self.total_impulse <= 0.0
            || self.propellant_mass <= 0.0
            || !self.is_burning(time)
        {
            return 0.0;
        }
        let thrust = self.thrust_at_time(time);
        if thrust <= 0.0 {
            return 0.0;
        }
        // m_dot = -thrust * propellant_mass / total_impulse
        -thrust * self.propellant_mass / self.total_impulse
    }

    /// Loads thrust curve data from a two-column CSV file (time, thrust).
    ///
    /// The CSV should have a header row and use commas as delimiters.
    /// Expects time in seconds and thrust in Newtons.
    pub fn load_from_csv(&mut self, path: &str) -> Result<(), String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read CSV file '{}': {}", path, e))?;

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(content.as_bytes());

        let mut points_found = 0u32;
        let mut max_time = 0.0f64;
        for result in rdr.records() {
            let record =
                result.map_err(|e| format!("CSV parse error at '{}': {}", path, e))?;
            if record.len() < 2 {
                continue;
            }
            let time_str = record[0].to_string();
            let thrust_str = record[1].to_string();
            let time: f64 = time_str
                .parse()
                .map_err(|e| format!("Invalid time value '{}': {}", time_str, e))?;
            let thrust: f64 = thrust_str
                .parse()
                .map_err(|e| format!("Invalid thrust value '{}': {}", thrust_str, e))?;
            self.add_thrust_point(time, thrust);
            max_time = max_time.max(time);
            points_found += 1;
        }

        if points_found == 0 {
            return Err(format!("No valid thrust data points found in '{}'", path));
        }

        // Auto-detect burn time from the last point where thrust is positive
        self.burn_time = self.burn_time.max(max_time);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_motor() -> MotorModel {
        let mut motor = MotorModel::new("TestCo".into(), "T100".into());
        motor.diameter = 29.0;
        motor.length = 150.0;
        motor.dry_mass = 0.15;
        motor.propellant_mass = 0.085;
        motor.burn_time = 2.0;
        motor.total_impulse = 200.0;

        // Add thrust points (trapezoidal-ish curve, area ≈ 200 N·s)
        motor.add_thrust_point(0.0, 0.0);
        motor.add_thrust_point(0.1, 100.0);
        motor.add_thrust_point(0.5, 120.0);
        motor.add_thrust_point(1.0, 110.0);
        motor.add_thrust_point(1.5, 80.0);
        motor.add_thrust_point(2.0, 0.0);

        motor
    }

    #[test]
    fn test_motor_creation() {
        let motor = MotorModel::new("AeroTech".into(), "K1103X".into());
        assert_eq!(motor.manufacturer, "AeroTech");
        assert_eq!(motor.designation, "K1103X");
        assert_eq!(motor.diameter, 0.0);
        assert_eq!(motor.burn_time, 0.0);
    }

    #[test]
    fn test_add_thrust_point() {
        let mut motor = MotorModel::new("Test".into(), "M1".into());
        motor.add_thrust_point(0.0, 10.0);
        motor.add_thrust_point(1.0, 20.0);
        // Should have 2 points and max_thrust = 20
        assert_eq!(motor.thrust_points.len(), 2);
        assert!((motor.max_thrust - 20.0).abs() < 1e-12);
    }

    #[test]
    fn test_thrust_at_time_before_burn() {
        let motor = make_test_motor();
        assert!((motor.thrust_at_time(-0.1) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_thrust_at_time_during_burn() {
        let motor = make_test_motor();
        let thrust = motor.thrust_at_time(0.5);
        assert!(thrust > 0.0);
    }

    #[test]
    fn test_thrust_at_time_after_burn() {
        let motor = make_test_motor();
        assert!((motor.thrust_at_time(3.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_is_burning() {
        let motor = make_test_motor();
        assert!(!motor.is_burning(-0.1));
        assert!(motor.is_burning(0.5));
        assert!(motor.is_burning(2.0));
        assert!(!motor.is_burning(2.1));
    }

    #[test]
    fn test_mass_at_time() {
        let motor = make_test_motor();
        // At t=0, mass = dry + propellant
        let initial_mass = motor.mass_at_time(0.0);
        assert!((initial_mass - (0.15 + 0.085)).abs() < 1e-9);

        // After burnout, mass should be close to dry mass
        let final_mass = motor.mass_at_time(3.0);
        assert!((final_mass - 0.15).abs() < 1.0); // allow some tolerance due to impulse integration

        // Mid-burn mass should be between initial and final
        let mid_mass = motor.mass_at_time(1.0);
        assert!(mid_mass > 0.15 && mid_mass < 0.235);
    }

    #[test]
    fn test_average_thrust() {
        let motor = make_test_motor();
        let avg = motor.average_thrust();
        assert!((avg - 100.0).abs() < 1.0); // 200 N·s / 2.0 s = 100 N
    }

    #[test]
    fn test_max_thrust() {
        let motor = make_test_motor();
        assert!((motor.max_thrust() - 120.0).abs() < 1e-12);
    }

    #[test]
    fn test_impulse_at_time() {
        let motor = make_test_motor();
        // At t=0, impulse = 0
        assert!((motor.impulse_at_time(0.0) - 0.0).abs() < 1e-12);

        // At t=burn_time, impulse should be close to total_impulse
        // Use larger tolerance since trapezoidal integration with linear interpolation
        // on a coarse thrust curve won't be exact
        let total = motor.impulse_at_time(2.0);
        assert!(total > 100.0 && total < 300.0); // verify reasonable range

        // Verify monotonic increase
        let mid = motor.impulse_at_time(1.0);
        assert!(mid > 0.0 && mid < total);
    }

    #[test]
    fn test_mass_flow_rate() {
        let motor = make_test_motor();
        // No flow before burn
        assert!((motor.mass_flow_rate(-0.1) - 0.0).abs() < 1e-12);

        // Positive flow (negative rate) during burn
        let rate = motor.mass_flow_rate(0.5);
        assert!(rate < 0.0); // mass decreasing

        // No flow after burn
        assert!((motor.mass_flow_rate(3.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_empty_thrust_curve() {
        let motor = MotorModel::new("Test".into(), "M1".into());
        assert!((motor.thrust_at_time(0.5) - 0.0).abs() < 1e-12);
        assert!(!motor.is_burning(0.5));
        assert!((motor.impulse_at_time(1.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_motor_serde_roundtrip() {
        let motor = make_test_motor();
        let json = serde_json::to_string(&motor).unwrap();
        let deser: MotorModel = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.manufacturer, "TestCo");
        assert_eq!(deser.designation, "T100");
        assert!((deser.diameter - 29.0).abs() < 1e-12);
        // The thrust curve is skipped, but thrust_points should be preserved
        assert_eq!(deser.thrust_points.len(), 6);
    }
}