use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// Enum representing all common units used in model rocketry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    // Length
    Millimeter,
    Centimeter,
    Meter,
    Kilometer,
    Inch,
    Foot,
    Yard,
    // Mass
    Gram,
    Kilogram,
    Ounce,
    Pound,
    // Time
    Second,
    Millisecond,
    Minute,
    // Velocity
    MetersPerSecond,
    FeetPerSecond,
    KilometersPerHour,
    MilesPerHour,
    Knots,
    // Acceleration
    MetersPerSecondSquared,
    GForce,
    FeetPerSecondSquared,
    // Angle
    Degree,
    Radian,
    // Force
    Newton,
    PoundForce,
    Kilonewton,
    // Pressure
    Pascal,
    Kilopascal,
    Atmosphere,
    Bar,
    PSI,
    // Temperature
    Celsius,
    Fahrenheit,
    Kelvin,
    // Area
    SquareMeter,
    SquareMillimeter,
    SquareInch,
    SquareFoot,
    // Volume
    CubicMeter,
    CubicMillimeter,
    Liter,
    GallonUS,
    GallonImperial,
    // Impulse
    NewtonSecond,
    PoundSecond,
}

impl Unit {
    /// Returns the conversion factor from this unit to the SI base unit.
    ///
    /// For temperature units (Celsius, Fahrenheit), this returns a multiplicative
    /// factor only; the offset is handled separately in `Quantity`.
    pub fn to_si(&self) -> f64 {
        match self {
            // Length → meter
            Unit::Millimeter => 0.001,
            Unit::Centimeter => 0.01,
            Unit::Meter => 1.0,
            Unit::Kilometer => 1000.0,
            Unit::Inch => 0.0254,
            Unit::Foot => 0.3048,
            Unit::Yard => 0.9144,
            // Mass → kilogram
            Unit::Gram => 0.001,
            Unit::Kilogram => 1.0,
            Unit::Ounce => 0.028349523125,
            Unit::Pound => 0.45359237,
            // Time → second
            Unit::Second => 1.0,
            Unit::Millisecond => 0.001,
            Unit::Minute => 60.0,
            // Velocity → meter/second
            Unit::MetersPerSecond => 1.0,
            Unit::FeetPerSecond => 0.3048,
            Unit::KilometersPerHour => 1000.0 / 3600.0,
            Unit::MilesPerHour => 1609.344 / 3600.0,
            Unit::Knots => 1852.0 / 3600.0,
            // Acceleration → meter/second²
            Unit::MetersPerSecondSquared => 1.0,
            Unit::GForce => 9.80665,
            Unit::FeetPerSecondSquared => 0.3048,
            // Angle → radian
            Unit::Degree => std::f64::consts::PI / 180.0,
            Unit::Radian => 1.0,
            // Force → newton
            Unit::Newton => 1.0,
            Unit::PoundForce => 4.4482216152605,
            Unit::Kilonewton => 1000.0,
            // Pressure → pascal
            Unit::Pascal => 1.0,
            Unit::Kilopascal => 1000.0,
            Unit::Atmosphere => 101_325.0,
            Unit::Bar => 100_000.0,
            Unit::PSI => 6894.757293168,
            // Temperature → kelvin (multiplicative factor only; offset in Quantity)
            Unit::Celsius => 1.0,
            Unit::Fahrenheit => 1.0 / 1.8,
            Unit::Kelvin => 1.0,
            // Area → square meter
            Unit::SquareMeter => 1.0,
            Unit::SquareMillimeter => 1e-6,
            Unit::SquareInch => 0.00064516,
            Unit::SquareFoot => 0.09290304,
            // Volume → cubic meter
            Unit::CubicMeter => 1.0,
            Unit::CubicMillimeter => 1e-9,
            Unit::Liter => 0.001,
            Unit::GallonUS => 0.003785411784,
            Unit::GallonImperial => 0.00454609,
            // Impulse → newton-second
            Unit::NewtonSecond => 1.0,
            Unit::PoundSecond => 4.4482216152605,
        }
    }

    /// Returns `true` if this unit is a temperature unit requiring offset conversion.
    pub fn is_temperature(&self) -> bool {
        matches!(self, Unit::Celsius | Unit::Fahrenheit | Unit::Kelvin)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Millimeter => write!(f, "mm"),
            Unit::Centimeter => write!(f, "cm"),
            Unit::Meter => write!(f, "m"),
            Unit::Kilometer => write!(f, "km"),
            Unit::Inch => write!(f, "in"),
            Unit::Foot => write!(f, "ft"),
            Unit::Yard => write!(f, "yd"),
            Unit::Gram => write!(f, "g"),
            Unit::Kilogram => write!(f, "kg"),
            Unit::Ounce => write!(f, "oz"),
            Unit::Pound => write!(f, "lb"),
            Unit::Second => write!(f, "s"),
            Unit::Millisecond => write!(f, "ms"),
            Unit::Minute => write!(f, "min"),
            Unit::MetersPerSecond => write!(f, "m/s"),
            Unit::FeetPerSecond => write!(f, "ft/s"),
            Unit::KilometersPerHour => write!(f, "km/h"),
            Unit::MilesPerHour => write!(f, "mph"),
            Unit::Knots => write!(f, "kn"),
            Unit::MetersPerSecondSquared => write!(f, "m/s²"),
            Unit::GForce => write!(f, "G"),
            Unit::FeetPerSecondSquared => write!(f, "ft/s²"),
            Unit::Degree => write!(f, "°"),
            Unit::Radian => write!(f, "rad"),
            Unit::Newton => write!(f, "N"),
            Unit::PoundForce => write!(f, "lbf"),
            Unit::Kilonewton => write!(f, "kN"),
            Unit::Pascal => write!(f, "Pa"),
            Unit::Kilopascal => write!(f, "kPa"),
            Unit::Atmosphere => write!(f, "atm"),
            Unit::Bar => write!(f, "bar"),
            Unit::PSI => write!(f, "psi"),
            Unit::Celsius => write!(f, "°C"),
            Unit::Fahrenheit => write!(f, "°F"),
            Unit::Kelvin => write!(f, "K"),
            Unit::SquareMeter => write!(f, "m²"),
            Unit::SquareMillimeter => write!(f, "mm²"),
            Unit::SquareInch => write!(f, "in²"),
            Unit::SquareFoot => write!(f, "ft²"),
            Unit::CubicMeter => write!(f, "m³"),
            Unit::CubicMillimeter => write!(f, "mm³"),
            Unit::Liter => write!(f, "L"),
            Unit::GallonUS => write!(f, "gal (US)"),
            Unit::GallonImperial => write!(f, "gal (imp)"),
            Unit::NewtonSecond => write!(f, "N·s"),
            Unit::PoundSecond => write!(f, "lbf·s"),
        }
    }
}

/// A quantity holding a numeric value in SI units internally.
///
/// `T` is typically `f64`, but can be any numeric type.
///
/// # Examples
///
/// ```
/// use federated_rocket_core::units::{Quantity, Unit};
///
/// let length = Quantity::new(100.0, Unit::Centimeter);
/// assert!((length.as_unit(Unit::Meter) - 1.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quantity<T> {
    /// Internal value stored in SI base units.
    si_value: T,
}

impl<T> Quantity<T> {
    /// Creates a new `Quantity` from a value in the given unit.
    ///
    /// The value is converted to SI internally.
    pub fn new(value: T, unit: Unit) -> Self
    where
        T: std::ops::Mul<f64, Output = T>,
    {
        let si_value = value * unit.to_si();
        Quantity { si_value }
    }

    /// Returns the value in the given unit.
    pub fn as_unit(&self, unit: Unit) -> T
    where
        T: std::ops::Div<f64, Output = T> + Copy,
    {
        self.si_value / unit.to_si()
    }

    /// Returns a reference to the internal SI value.
    pub fn value(&self) -> &T {
        &self.si_value
    }

    /// Returns a mutable reference to the internal SI value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.si_value
    }

    /// Consumes the quantity and returns the internal SI value.
    pub fn into_value(self) -> T {
        self.si_value
    }

    /// Creates a `Quantity` from a value already in SI units.
    pub fn from_si(si_value: T) -> Self {
        Quantity { si_value }
    }
}

// --- Arithmetic operations for Quantity<f64> ---

impl Add for Quantity<f64> {
    type Output = Quantity<f64>;

    fn add(self, other: Quantity<f64>) -> Quantity<f64> {
        Quantity {
            si_value: self.si_value + other.si_value,
        }
    }
}

impl Sub for Quantity<f64> {
    type Output = Quantity<f64>;

    fn sub(self, other: Quantity<f64>) -> Quantity<f64> {
        Quantity {
            si_value: self.si_value - other.si_value,
        }
    }
}

impl Mul<f64> for Quantity<f64> {
    type Output = Quantity<f64>;

    fn mul(self, scalar: f64) -> Quantity<f64> {
        Quantity {
            si_value: self.si_value * scalar,
        }
    }
}

impl Div<f64> for Quantity<f64> {
    type Output = Quantity<f64>;

    fn div(self, scalar: f64) -> Quantity<f64> {
        Quantity {
            si_value: self.si_value / scalar,
        }
    }
}

impl AddAssign for Quantity<f64> {
    fn add_assign(&mut self, other: Quantity<f64>) {
        self.si_value += other.si_value;
    }
}

impl SubAssign for Quantity<f64> {
    fn sub_assign(&mut self, other: Quantity<f64>) {
        self.si_value -= other.si_value;
    }
}

impl fmt::Display for Quantity<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} SI", self.si_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    #[test]
    fn test_length_conversions() {
        let m = Quantity::new(1.0, Unit::Meter);
        assert!((m.as_unit(Unit::Millimeter) - 1000.0).abs() < EPS);
        assert!((m.as_unit(Unit::Centimeter) - 100.0).abs() < EPS);
        assert!((m.as_unit(Unit::Kilometer) - 0.001).abs() < EPS);
        assert!((m.as_unit(Unit::Inch) - 39.37007874015748).abs() < 1e-10);
        assert!((m.as_unit(Unit::Foot) - 3.280839895013123).abs() < 1e-10);
        assert!((m.as_unit(Unit::Yard) - 1.0936132983377078).abs() < 1e-10);
    }

    #[test]
    fn test_mass_conversions() {
        let kg = Quantity::new(1.0, Unit::Kilogram);
        assert!((kg.as_unit(Unit::Gram) - 1000.0).abs() < EPS);
        assert!((kg.as_unit(Unit::Pound) - 2.2046226218487757).abs() < 1e-10);
        assert!((kg.as_unit(Unit::Ounce) - 35.27396194958041).abs() < 1e-10);
    }

    #[test]
    fn test_time_conversions() {
        let s = Quantity::new(60.0, Unit::Second);
        assert!((s.as_unit(Unit::Minute) - 1.0).abs() < EPS);
        assert!((s.as_unit(Unit::Millisecond) - 60_000.0).abs() < EPS);
    }

    #[test]
    fn test_velocity_conversions() {
        let ms = Quantity::new(1.0, Unit::MetersPerSecond);
        assert!((ms.as_unit(Unit::KilometersPerHour) - 3.6).abs() < EPS);
        assert!((ms.as_unit(Unit::FeetPerSecond) - 3.280839895013123).abs() < 1e-10);
        assert!((ms.as_unit(Unit::MilesPerHour) - 2.2369362920544025).abs() < 1e-10);
        assert!((ms.as_unit(Unit::Knots) - 1.9438444924406045).abs() < 1e-10);
    }

    #[test]
    fn test_acceleration_conversions() {
        let ms2 = Quantity::new(1.0, Unit::MetersPerSecondSquared);
        assert!((ms2.as_unit(Unit::GForce) - 0.10197162129779283).abs() < 1e-10);
        assert!((ms2.as_unit(Unit::FeetPerSecondSquared) - 3.280839895013123).abs() < 1e-10);
    }

    #[test]
    fn test_angle_conversions() {
        let rad = Quantity::new(std::f64::consts::PI, Unit::Radian);
        assert!((rad.as_unit(Unit::Degree) - 180.0).abs() < EPS);
    }

    #[test]
    fn test_force_conversions() {
        let n = Quantity::new(1.0, Unit::Newton);
        assert!((n.as_unit(Unit::PoundForce) - 0.2248089430997105).abs() < 1e-10);
        assert!((n.as_unit(Unit::Kilonewton) - 0.001).abs() < EPS);
    }

    #[test]
    fn test_pressure_conversions() {
        let pa = Quantity::new(101_325.0, Unit::Pascal);
        assert!((pa.as_unit(Unit::Atmosphere) - 1.0).abs() < EPS);
        assert!((pa.as_unit(Unit::Bar) - 1.01325).abs() < EPS);
        assert!((pa.as_unit(Unit::PSI) - 14.69594877551345).abs() < 1e-10);
        assert!((pa.as_unit(Unit::Kilopascal) - 101.325).abs() < EPS);
    }

    #[test]
    fn test_area_conversions() {
        let sm = Quantity::new(1.0, Unit::SquareMeter);
        assert!((sm.as_unit(Unit::SquareMillimeter) - 1_000_000.0).abs() < EPS);
        assert!((sm.as_unit(Unit::SquareInch) - 1550.0031000062).abs() < 1e-8);
        assert!((sm.as_unit(Unit::SquareFoot) - 10.763910416709722).abs() < 1e-10);
    }

    #[test]
    fn test_volume_conversions() {
        let cm = Quantity::new(1.0, Unit::CubicMeter);
        assert!((cm.as_unit(Unit::Liter) - 1000.0).abs() < EPS);
        // 1 m³ = 1,000,000,000 mm³; use explicit literal for large number
        assert!((cm.as_unit(Unit::CubicMillimeter) - 1_000_000_000.0).abs() < 1e-6);
        assert!((cm.as_unit(Unit::GallonUS) - 264.17205235814845).abs() < 1e-10);
        assert!((cm.as_unit(Unit::GallonImperial) - 219.96924829908778).abs() < 1e-10);
    }

    #[test]
    fn test_impulse_conversions() {
        let ns = Quantity::new(1.0, Unit::NewtonSecond);
        assert!((ns.as_unit(Unit::PoundSecond) - 0.2248089430997105).abs() < 1e-10);
    }

    #[test]
    fn test_arithmetic_add() {
        let a = Quantity::new(1.0, Unit::Meter);
        let b = Quantity::new(50.0, Unit::Centimeter);
        let sum = a + b;
        assert!((sum.as_unit(Unit::Meter) - 1.5).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_sub() {
        let a = Quantity::new(1.0, Unit::Kilogram);
        let b = Quantity::new(200.0, Unit::Gram);
        let diff = a - b;
        assert!((diff.as_unit(Unit::Kilogram) - 0.8).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_mul() {
        let a = Quantity::new(10.0, Unit::Meter);
        let prod = a * 2.0;
        assert!((prod.as_unit(Unit::Meter) - 20.0).abs() < EPS);
    }

    #[test]
    fn test_arithmetic_div() {
        let a = Quantity::new(10.0, Unit::Second);
        let quot = a / 2.0;
        assert!((quot.as_unit(Unit::Second) - 5.0).abs() < EPS);
    }

    #[test]
    fn test_add_assign() {
        let mut a = Quantity::new(1.0, Unit::Meter);
        a += Quantity::new(50.0, Unit::Centimeter);
        assert!((a.as_unit(Unit::Meter) - 1.5).abs() < EPS);
    }

    #[test]
    fn test_sub_assign() {
        let mut a = Quantity::new(1.0, Unit::Kilogram);
        a -= Quantity::new(200.0, Unit::Gram);
        assert!((a.as_unit(Unit::Kilogram) - 0.8).abs() < EPS);
    }

    #[test]
    fn test_roundtrip() {
        let q = Quantity::new(3.14159, Unit::Meter);
        let si = q.into_value();
        let q2 = Quantity::from_si(si);
        assert!((q2.as_unit(Unit::Meter) - 3.14159).abs() < EPS);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Unit::Meter), "m");
        assert_eq!(format!("{}", Unit::Kilogram), "kg");
        assert_eq!(format!("{}", Unit::NewtonSecond), "N·s");
        assert_eq!(format!("{}", Unit::PoundForce), "lbf");
        assert_eq!(format!("{}", Unit::GForce), "G");
    }
}