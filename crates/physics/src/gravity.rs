use crate::constants::{EARTH_RADIUS, STANDARD_GRAVITY};

// ---------------------------------------------------------------------------
// GravityModel trait
// ---------------------------------------------------------------------------

/// Trait for gravity models (Strategy pattern).
pub trait GravityModel: Send + Sync {
    /// Compute the gravitational acceleration (m/s²) at the given altitude (m).
    fn acceleration_at_altitude(&self, altitude: f64) -> f64;

    /// Human-readable name of the model.
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// ConstantGravity
// ---------------------------------------------------------------------------

/// Constant gravity model: g = 9.80665 m/s² everywhere.
#[derive(Debug, Clone)]
pub struct ConstantGravity;

impl ConstantGravity {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConstantGravity {
    fn default() -> Self {
        Self::new()
    }
}

impl GravityModel for ConstantGravity {
    fn acceleration_at_altitude(&self, _altitude: f64) -> f64 {
        STANDARD_GRAVITY
    }

    fn name(&self) -> &'static str {
        "Constant Gravity (9.80665 m/s²)"
    }
}

// ---------------------------------------------------------------------------
// InverseSquareGravity
// ---------------------------------------------------------------------------

/// Inverse-square gravity model:
///   g(h) = g₀ · (R / (R + h))²
///
/// where R = 6,371,000 m (Earth's mean radius).
#[derive(Debug, Clone)]
pub struct InverseSquareGravity;

impl InverseSquareGravity {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InverseSquareGravity {
    fn default() -> Self {
        Self::new()
    }
}

impl GravityModel for InverseSquareGravity {
    fn acceleration_at_altitude(&self, altitude: f64) -> f64 {
        let h = altitude.max(0.0);
        let ratio = EARTH_RADIUS / (EARTH_RADIUS + h);
        STANDARD_GRAVITY * ratio * ratio
    }

    fn name(&self) -> &'static str {
        "Inverse-Square Gravity"
    }
}

// ---------------------------------------------------------------------------
// Wgs84Gravity
// ---------------------------------------------------------------------------

/// WGS-84 ellipsoidal gravity model.
///
/// Computes gravity as a function of geodetic latitude with a free-air
/// altitude correction.
///
/// Reference: WGS-84 Gravity Formula
///   g(φ) = 9.7803253359 · (1 + 0.00193185265241 · sin²(φ))
///          / sqrt(1 - 0.00669437999013 · sin²(φ))
///
/// Altitude correction (free-air):
///   g(φ, h) = g(φ) - (3.0877e-6 - 4.4e-9 · sin²(φ)) · h + 7.2e-13 · h²
///
/// Latitude defaults to 45° if not specified.
#[derive(Debug, Clone)]
pub struct Wgs84Gravity {
    /// Geodetic latitude (degrees).
    latitude_deg: f64,
}

impl Wgs84Gravity {
    /// Create a new WGS-84 gravity model at the given latitude.
    pub fn new(latitude_deg: f64) -> Self {
        Self { latitude_deg }
    }

    /// Gravity at sea level for the configured latitude, using the WGS-84
    /// ellipsoidal formula.
    fn gravity_at_sea_level(&self) -> f64 {
        let sin_lat = self.latitude_deg.to_radians().sin();
        let sin2 = sin_lat * sin_lat;

        let numerator = 9.7803253359 * (1.0 + 0.00193185265241 * sin2);
        let denominator = (1.0 - 0.00669437999013 * sin2).sqrt();
        numerator / denominator
    }
}

impl Default for Wgs84Gravity {
    fn default() -> Self {
        // Default to 45° latitude
        Self::new(45.0)
    }
}

impl GravityModel for Wgs84Gravity {
    fn acceleration_at_altitude(&self, altitude: f64) -> f64 {
        let h = altitude.max(0.0);
        let g0 = self.gravity_at_sea_level();
        let sin_lat = self.latitude_deg.to_radians().sin();
        let sin2 = sin_lat * sin_lat;

        // Free-air correction
        let correction = (3.0877e-6 - 4.4e-9 * sin2) * h - 7.2e-13 * h * h;
        g0 - correction
    }

    fn name(&self) -> &'static str {
        "WGS-84 Ellipsoidal Gravity"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- ConstantGravity ---

    #[test]
    fn test_constant_gravity() {
        let model = ConstantGravity::new();
        assert!((model.acceleration_at_altitude(0.0) - 9.80665).abs() < 1e-10);
        assert!((model.acceleration_at_altitude(10_000.0) - 9.80665).abs() < 1e-10);
        assert!((model.acceleration_at_altitude(100_000.0) - 9.80665).abs() < 1e-10);
    }

    // --- InverseSquareGravity ---

    #[test]
    fn test_inverse_square_sea_level() {
        let model = InverseSquareGravity::new();
        let g = model.acceleration_at_altitude(0.0);
        assert!((g - 9.80665).abs() < 1e-6, "g = {}", g);
    }

    #[test]
    fn test_inverse_square_decreases_with_altitude() {
        let model = InverseSquareGravity::new();
        let g0 = model.acceleration_at_altitude(0.0);
        let g10k = model.acceleration_at_altitude(10_000.0);
        let g100k = model.acceleration_at_altitude(100_000.0);
        assert!(g10k < g0, "g should decrease with altitude");
        assert!(g100k < g10k, "g should continue decreasing");
    }

    #[test]
    fn test_inverse_square_at_iss_altitude() {
        let model = InverseSquareGravity::new();
        // ISS orbits at ~408 km
        let g = model.acceleration_at_altitude(408_000.0);
        // g ≈ 9.80665 * (6371/6779)² ≈ 9.80665 * 0.883 ≈ 8.66 m/s²
        assert!((g - 8.66).abs() < 0.1, "g at ISS altitude = {}", g);
    }

    #[test]
    fn test_inverse_square_negative_altitude_clamps() {
        let model = InverseSquareGravity::new();
        let g0 = model.acceleration_at_altitude(0.0);
        let g_neg = model.acceleration_at_altitude(-1000.0);
        assert!(
            (g_neg - g0).abs() < 1e-10,
            "Negative altitude should clamp to sea level"
        );
    }

    // --- Wgs84Gravity ---

    #[test]
    fn test_wgs84_at_equator() {
        let model = Wgs84Gravity::new(0.0);
        let g = model.acceleration_at_altitude(0.0);
        // At equator: g ≈ 9.7803 m/s²
        assert!((g - 9.7803).abs() < 0.001, "g at equator = {}", g);
    }

    #[test]
    fn test_wgs84_at_pole() {
        let model = Wgs84Gravity::new(90.0);
        let g = model.acceleration_at_altitude(0.0);
        // At pole: g ≈ 9.8322 m/s²
        assert!((g - 9.8322).abs() < 0.001, "g at pole = {}", g);
    }

    #[test]
    fn test_wgs84_at_45_deg() {
        let model = Wgs84Gravity::new(45.0);
        let g = model.acceleration_at_altitude(0.0);
        // At 45°: g ≈ 9.80665 m/s² (standard gravity)
        assert!((g - 9.80665).abs() < 0.01, "g at 45° = {}", g);
    }

    #[test]
    fn test_wgs84_at_45_deg_altitude() {
        let model = Wgs84Gravity::new(45.0);
        let g0 = model.acceleration_at_altitude(0.0);
        let g10k = model.acceleration_at_altitude(10_000.0);
        // Gravity should decrease with altitude
        assert!(g10k < g0, "g should decrease with altitude");
    }

    #[test]
    fn test_wgs84_default_is_45_deg() {
        let default = Wgs84Gravity::default();
        let explicit = Wgs84Gravity::new(45.0);
        let g_default = default.acceleration_at_altitude(0.0);
        let g_explicit = explicit.acceleration_at_altitude(0.0);
        assert!((g_default - g_explicit).abs() < 1e-10);
    }

    // --- Trait object dispatch ---

    #[test]
    fn test_gravity_trait_dispatch() {
        let models: Vec<Box<dyn GravityModel>> = vec![
            Box::new(ConstantGravity::new()),
            Box::new(InverseSquareGravity::new()),
            Box::new(Wgs84Gravity::new(45.0)),
        ];
        for model in &models {
            let g = model.acceleration_at_altitude(0.0);
            assert!(g > 0.0, "{} should return positive gravity", model.name());
        }
    }
}
