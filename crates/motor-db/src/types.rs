use serde::{Deserialize, Serialize};

/// A model rocket motor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motor {
    pub id: Option<i64>,
    pub manufacturer: String,
    pub manufacturer_abbrev: String,
    pub designation: String, // e.g. "C6-5"
    pub motor_type: MotorType,
    pub diameter: f64,        // mm
    pub length: f64,          // mm
    pub total_impulse: f64,   // N·s
    pub burn_time: f64,       // s
    pub avg_thrust: f64,      // N
    pub max_thrust: f64,      // N
    pub propellant_mass: f64, // g
    pub dry_mass: f64,        // g
    pub delay_time: f64,      // s (0 = plugged)
    pub thrust_curve: Vec<ThrustPoint>,
    pub data_source: String,  // "embedded", "thrustcurve", "user"
}

impl Motor {
    /// Compute the NAR impulse class for this motor based on total impulse.
    pub fn impulse_class(&self) -> ImpulseClass {
        ImpulseClass::from_total_impulse(self.total_impulse)
    }

    /// Full designation including manufacturer, e.g. "Estes C6-5"
    pub fn full_designation(&self) -> String {
        format!("{} {}", self.manufacturer, self.designation)
    }

    /// Create a thrust curve interpolator model for continuous thrust lookup.
    pub fn thrust_curve_interpolator(&self) -> Result<MotorModel, String> {
        if self.thrust_curve.is_empty() {
            return Err("Thrust curve is empty".to_string());
        }
        // Validate that time points are strictly increasing
        for i in 1..self.thrust_curve.len() {
            if self.thrust_curve[i].time <= self.thrust_curve[i - 1].time {
                return Err(format!(
                    "Thrust curve time not strictly increasing at index {}: {} <= {}",
                    i,
                    self.thrust_curve[i].time,
                    self.thrust_curve[i - 1].time
                ));
            }
        }
        Ok(MotorModel {
            points: self.thrust_curve.clone(),
        })
    }
}

/// Simple linear interpolation model for motor thrust curves.
#[derive(Debug, Clone)]
pub struct MotorModel {
    points: Vec<ThrustPoint>,
}

impl MotorModel {
    /// Look up thrust at a given time using linear interpolation.
    /// Returns 0.0 for times before the first point or after the last point.
    pub fn thrust_at(&self, time: f64) -> f64 {
        let pts = &self.points;
        if pts.is_empty() {
            return 0.0;
        }
        if time <= pts[0].time {
            return pts[0].thrust;
        }
        if time >= pts[pts.len() - 1].time {
            return 0.0;
        }
        // Binary search for the interval
        let idx = match pts.binary_search_by(|p| p.time.partial_cmp(&time).unwrap()) {
            Ok(i) => return pts[i].thrust,
            Err(i) => i,
        };
        if idx == 0 {
            return pts[0].thrust;
        }
        if idx >= pts.len() {
            return 0.0;
        }
        let p0 = &pts[idx - 1];
        let p1 = &pts[idx];
        let t = (time - p0.time) / (p1.time - p0.time);
        p0.thrust + t * (p1.thrust - p0.thrust)
    }

    /// Total burn duration from the thrust curve.
    pub fn burn_duration(&self) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }
        self.points[self.points.len() - 1].time
    }

    /// Number of data points in the thrust curve.
    pub fn num_points(&self) -> usize {
        self.points.len()
    }
}

/// Type of rocket motor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorType {
    /// Standard solid propellant
    Solid,
    /// Hybrid motor
    Hybrid,
    /// Liquid motor
    Liquid,
    /// Electric motor for ducted fans
    Electric,
}

impl MotorType {
    /// Returns a string representation of the motor type.
    pub fn as_str(&self) -> &'static str {
        match self {
            MotorType::Solid => "Solid",
            MotorType::Hybrid => "Hybrid",
            MotorType::Liquid => "Liquid",
            MotorType::Electric => "Electric",
        }
    }

    /// Parse a motor type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "solid" => Some(MotorType::Solid),
            "hybrid" => Some(MotorType::Hybrid),
            "liquid" => Some(MotorType::Liquid),
            "electric" => Some(MotorType::Electric),
            _ => None,
        }
    }
}

/// NAR impulse class for model rocket motors.
///
/// Each class approximately doubles the total impulse of the previous class.
/// The standard classes range from 1/4A through Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImpulseClass {
    /// Motors smaller than A (1/4A, 1/2A)
    Micro,
    A, B, C, D, E, F, G,
    H, I, J, K, L, M,
    N, O, P, Q, R,
    S, T, U, V, W,
    X, Y, Z,
}

impl ImpulseClass {
    /// Determine the impulse class from total impulse in N·s.
    ///
    /// NAR standard impulse ranges (each class approximately doubles):
    /// - 1/4A: 0.3125 - 0.625 N·s
    /// - 1/2A: 0.625 - 1.25 N·s
    /// - A: 1.26 - 2.50 N·s
    /// - B: 2.51 - 5.00 N·s
    /// - C: 5.01 - 10.00 N·s
    /// - D: 10.01 - 20.00 N·s
    /// - etc.
    pub fn from_total_impulse(impulse: f64) -> Self {
        // A class starts at 1.26 N·s, each class doubles
        let classes = [
            (0.0, ImpulseClass::Micro),
            (1.26, ImpulseClass::A),
            (2.51, ImpulseClass::B),
            (5.01, ImpulseClass::C),
            (10.01, ImpulseClass::D),
            (20.01, ImpulseClass::E),
            (40.01, ImpulseClass::F),
            (80.01, ImpulseClass::G),
            (160.01, ImpulseClass::H),
            (320.01, ImpulseClass::I),
            (640.01, ImpulseClass::J),
            (1280.01, ImpulseClass::K),
            (2560.01, ImpulseClass::L),
            (5120.01, ImpulseClass::M),
            (10240.01, ImpulseClass::N),
            (20480.01, ImpulseClass::O),
            (40960.01, ImpulseClass::P),
            (81920.01, ImpulseClass::Q),
            (163840.01, ImpulseClass::R),
            (327680.01, ImpulseClass::S),
            (655360.01, ImpulseClass::T),
            (1310720.01, ImpulseClass::U),
            (2621440.01, ImpulseClass::V),
            (5242880.01, ImpulseClass::W),
            (10485760.01, ImpulseClass::X),
            (20971520.01, ImpulseClass::Y),
            (41943040.01, ImpulseClass::Z),
        ];

        for &(threshold, ref class) in classes.iter().rev() {
            if impulse >= threshold {
                return *class;
            }
        }
        ImpulseClass::Micro
    }

    /// Returns the minimum and maximum total impulse in N·s for this class.
    pub fn impulse_range(&self) -> (f64, f64) {
        match self {
            ImpulseClass::Micro => (0.0, 1.25),
            ImpulseClass::A => (1.26, 2.50),
            ImpulseClass::B => (2.51, 5.00),
            ImpulseClass::C => (5.01, 10.00),
            ImpulseClass::D => (10.01, 20.00),
            ImpulseClass::E => (20.01, 40.00),
            ImpulseClass::F => (40.01, 80.00),
            ImpulseClass::G => (80.01, 160.00),
            ImpulseClass::H => (160.01, 320.00),
            ImpulseClass::I => (320.01, 640.00),
            ImpulseClass::J => (640.01, 1280.00),
            ImpulseClass::K => (1280.01, 2560.00),
            ImpulseClass::L => (2560.01, 5120.00),
            ImpulseClass::M => (5120.01, 10240.00),
            ImpulseClass::N => (10240.01, 20480.00),
            ImpulseClass::O => (20480.01, 40960.00),
            ImpulseClass::P => (40960.01, 81920.00),
            ImpulseClass::Q => (81920.01, 163840.00),
            ImpulseClass::R => (163840.01, 327680.00),
            ImpulseClass::S => (327680.01, 655360.00),
            ImpulseClass::T => (655360.01, 1310720.00),
            ImpulseClass::U => (1310720.01, 2621440.00),
            ImpulseClass::V => (2621440.01, 5242880.00),
            ImpulseClass::W => (5242880.01, 10485760.00),
            ImpulseClass::X => (10485760.01, 20971520.00),
            ImpulseClass::Y => (20971520.01, 41943040.00),
            ImpulseClass::Z => (41943040.01, f64::INFINITY),
        }
    }

    /// Human-readable display name for the impulse class.
    pub fn display_name(&self) -> &'static str {
        match self {
            ImpulseClass::Micro => "Micro (1/4A, 1/2A)",
            ImpulseClass::A => "A",
            ImpulseClass::B => "B",
            ImpulseClass::C => "C",
            ImpulseClass::D => "D",
            ImpulseClass::E => "E",
            ImpulseClass::F => "F",
            ImpulseClass::G => "G",
            ImpulseClass::H => "H",
            ImpulseClass::I => "I",
            ImpulseClass::J => "J",
            ImpulseClass::K => "K",
            ImpulseClass::L => "L",
            ImpulseClass::M => "M",
            ImpulseClass::N => "N",
            ImpulseClass::O => "O",
            ImpulseClass::P => "P",
            ImpulseClass::Q => "Q",
            ImpulseClass::R => "R",
            ImpulseClass::S => "S",
            ImpulseClass::T => "T",
            ImpulseClass::U => "U",
            ImpulseClass::V => "V",
            ImpulseClass::W => "W",
            ImpulseClass::X => "X",
            ImpulseClass::Y => "Y",
            ImpulseClass::Z => "Z",
        }
    }

    /// Returns the short label for use in motor designations (e.g., "C" for C-class).
    pub fn label(&self) -> &'static str {
        match self {
            ImpulseClass::Micro => "Micro",
            ImpulseClass::A => "A",
            ImpulseClass::B => "B",
            ImpulseClass::C => "C",
            ImpulseClass::D => "D",
            ImpulseClass::E => "E",
            ImpulseClass::F => "F",
            ImpulseClass::G => "G",
            ImpulseClass::H => "H",
            ImpulseClass::I => "I",
            ImpulseClass::J => "J",
            ImpulseClass::K => "K",
            ImpulseClass::L => "L",
            ImpulseClass::M => "M",
            ImpulseClass::N => "N",
            ImpulseClass::O => "O",
            ImpulseClass::P => "P",
            ImpulseClass::Q => "Q",
            ImpulseClass::R => "R",
            ImpulseClass::S => "S",
            ImpulseClass::T => "T",
            ImpulseClass::U => "U",
            ImpulseClass::V => "V",
            ImpulseClass::W => "W",
            ImpulseClass::X => "X",
            ImpulseClass::Y => "Y",
            ImpulseClass::Z => "Z",
        }
    }
}

/// Single thrust curve data point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThrustPoint {
    /// Seconds from ignition
    pub time: f64,
    /// Thrust in Newtons
    pub thrust: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impulse_class_classification() {
        // Micro class (0 - 1.25 N·s)
        assert_eq!(ImpulseClass::from_total_impulse(0.0), ImpulseClass::Micro);
        assert_eq!(ImpulseClass::from_total_impulse(0.5), ImpulseClass::Micro);
        assert_eq!(ImpulseClass::from_total_impulse(1.0), ImpulseClass::Micro);
        assert_eq!(ImpulseClass::from_total_impulse(1.25), ImpulseClass::Micro);

        // A class (1.26 - 2.50 N·s)
        assert_eq!(ImpulseClass::from_total_impulse(1.26), ImpulseClass::A);
        assert_eq!(ImpulseClass::from_total_impulse(2.0), ImpulseClass::A);
        assert_eq!(ImpulseClass::from_total_impulse(2.50), ImpulseClass::A);

        // B class (2.51 - 5.00 N·s)
        assert_eq!(ImpulseClass::from_total_impulse(2.51), ImpulseClass::B);
        assert_eq!(ImpulseClass::from_total_impulse(4.0), ImpulseClass::B);
        assert_eq!(ImpulseClass::from_total_impulse(5.00), ImpulseClass::B);

        // C class (5.01 - 10.00 N·s)
        assert_eq!(ImpulseClass::from_total_impulse(5.01), ImpulseClass::C);
        assert_eq!(ImpulseClass::from_total_impulse(8.82), ImpulseClass::C);
        assert_eq!(ImpulseClass::from_total_impulse(10.00), ImpulseClass::C);

        // D class (10.01 - 20.00 N·s)
        assert_eq!(ImpulseClass::from_total_impulse(10.01), ImpulseClass::D);
        assert_eq!(ImpulseClass::from_total_impulse(15.0), ImpulseClass::D);

        // Higher classes
        assert_eq!(ImpulseClass::from_total_impulse(160.01), ImpulseClass::H);
        assert_eq!(ImpulseClass::from_total_impulse(500.0), ImpulseClass::I);
        assert_eq!(ImpulseClass::from_total_impulse(1000.0), ImpulseClass::J);
    }

    #[test]
    fn test_impulse_class_range() {
        assert_eq!(ImpulseClass::Micro.impulse_range(), (0.0, 1.25));
        assert_eq!(ImpulseClass::A.impulse_range(), (1.26, 2.50));
        assert_eq!(ImpulseClass::C.impulse_range(), (5.01, 10.00));
        assert_eq!(ImpulseClass::H.impulse_range(), (160.01, 320.00));
    }

    #[test]
    fn test_impulse_display_name() {
        assert_eq!(ImpulseClass::Micro.display_name(), "Micro (1/4A, 1/2A)");
        assert_eq!(ImpulseClass::C.display_name(), "C");
        assert_eq!(ImpulseClass::H.display_name(), "H");
    }

    #[test]
    fn test_motor_impulse_class() {
        let motor = Motor {
            id: None,
            manufacturer: "Estes".to_string(),
            manufacturer_abbrev: "EST".to_string(),
            designation: "C6-5".to_string(),
            motor_type: MotorType::Solid,
            diameter: 18.0,
            length: 70.0,
            total_impulse: 8.82,
            burn_time: 1.6,
            avg_thrust: 5.5,
            max_thrust: 12.0,
            propellant_mass: 10.5,
            dry_mass: 11.0,
            delay_time: 5.0,
            thrust_curve: vec![],
            data_source: "embedded".to_string(),
        };
        assert_eq!(motor.impulse_class(), ImpulseClass::C);
        assert_eq!(motor.full_designation(), "Estes C6-5");
    }

    #[test]
    fn test_motor_type_conversion() {
        assert_eq!(MotorType::from_str("Solid"), Some(MotorType::Solid));
        assert_eq!(MotorType::from_str("solid"), Some(MotorType::Solid));
        assert_eq!(MotorType::from_str("Hybrid"), Some(MotorType::Hybrid));
        assert_eq!(MotorType::from_str("Liquid"), Some(MotorType::Liquid));
        assert_eq!(MotorType::from_str("Electric"), Some(MotorType::Electric));
        assert_eq!(MotorType::from_str("unknown"), None);
        assert_eq!(MotorType::Solid.as_str(), "Solid");
        assert_eq!(MotorType::Hybrid.as_str(), "Hybrid");
    }

    #[test]
    fn test_motor_model_interpolation() {
        let motor = Motor {
            id: None,
            manufacturer: "Test".to_string(),
            manufacturer_abbrev: "TST".to_string(),
            designation: "T1-0".to_string(),
            motor_type: MotorType::Solid,
            diameter: 18.0,
            length: 70.0,
            total_impulse: 10.0,
            burn_time: 2.0,
            avg_thrust: 5.0,
            max_thrust: 10.0,
            propellant_mass: 10.0,
            dry_mass: 10.0,
            delay_time: 0.0,
            thrust_curve: vec![
                ThrustPoint { time: 0.0, thrust: 0.0 },
                ThrustPoint { time: 1.0, thrust: 10.0 },
                ThrustPoint { time: 2.0, thrust: 0.0 },
            ],
            data_source: "test".to_string(),
        };

        let model = motor.thrust_curve_interpolator().unwrap();
        assert_eq!(model.num_points(), 3);
        assert!((model.burn_duration() - 2.0).abs() < 1e-10);
        assert!((model.thrust_at(0.0) - 0.0).abs() < 1e-10);
        assert!((model.thrust_at(1.0) - 10.0).abs() < 1e-10);
        assert!((model.thrust_at(0.5) - 5.0).abs() < 1e-10); // linear interpolation
        assert!((model.thrust_at(1.5) - 5.0).abs() < 1e-10);
        assert!((model.thrust_at(3.0) - 0.0).abs() < 1e-10); // past end
        assert!((model.thrust_at(-1.0) - 0.0).abs() < 1e-10); // before start
    }

    #[test]
    fn test_empty_thrust_curve_error() {
        let motor = Motor {
            id: None,
            manufacturer: "Test".to_string(),
            manufacturer_abbrev: "TST".to_string(),
            designation: "T1-0".to_string(),
            motor_type: MotorType::Solid,
            diameter: 18.0,
            length: 70.0,
            total_impulse: 10.0,
            burn_time: 2.0,
            avg_thrust: 5.0,
            max_thrust: 10.0,
            propellant_mass: 10.0,
            dry_mass: 10.0,
            delay_time: 0.0,
            thrust_curve: vec![],
            data_source: "test".to_string(),
        };
        assert!(motor.thrust_curve_interpolator().is_err());
    }
}