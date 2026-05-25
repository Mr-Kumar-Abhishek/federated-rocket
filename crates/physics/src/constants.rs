/// Standard gravitational acceleration at sea level (m/s²)
pub const STANDARD_GRAVITY: f64 = 9.80665;

/// Universal gas constant (J/(mol·K))
pub const UNIVERSAL_GAS_CONSTANT: f64 = 8.314462618;

/// Molar mass of dry air (kg/mol)
pub const MOLAR_MASS_DRY_AIR: f64 = 0.0289647;

/// Specific gas constant for dry air (J/(kg·K))
pub const SPECIFIC_GAS_CONSTANT_DRY_AIR: f64 = 287.058;

/// Ratio of specific heats for dry air (gamma)
pub const GAMMA_DRY_AIR: f64 = 1.4;

/// Standard sea-level temperature (K)
pub const STANDARD_TEMPERATURE: f64 = 288.15;

/// Standard sea-level pressure (Pa)
pub const STANDARD_PRESSURE: f64 = 101325.0;

/// Standard sea-level density (kg/m³)
pub const STANDARD_DENSITY: f64 = 1.225;

/// Standard temperature lapse rate (K/m) up to 11 km
pub const STANDARD_LAPSE_RATE: f64 = -0.0065;

/// Speed of sound at sea level (m/s)
pub const SPEED_OF_SOUND_SEA_LEVEL: f64 = 340.294;

/// Dynamic viscosity of air at sea level (Pa·s)
pub const AIR_VISCOSITY_SEA_LEVEL: f64 = 1.7894e-5;

/// Earth's equatorial radius (m)
pub const EARTH_RADIUS: f64 = 6_371_000.0;

/// Earth's mass (kg)
pub const EARTH_MASS: f64 = 5.9722e24;

/// Gravitational constant (N·m²/kg²)
pub const GRAVITATIONAL_CONSTANT: f64 = 6.67430e-11;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_gravity_value() {
        assert!((STANDARD_GRAVITY - 9.80665).abs() < 1e-10);
    }

    #[test]
    fn test_universal_gas_constant_value() {
        assert!((UNIVERSAL_GAS_CONSTANT - 8.314462618).abs() < 1e-10);
    }

    #[test]
    fn test_molar_mass_dry_air_value() {
        assert!((MOLAR_MASS_DRY_AIR - 0.0289647).abs() < 1e-10);
    }

    #[test]
    fn test_specific_gas_constant_value() {
        assert!((SPECIFIC_GAS_CONSTANT_DRY_AIR - 287.058).abs() < 1e-10);
    }

    #[test]
    fn test_gamma_dry_air_value() {
        assert!((GAMMA_DRY_AIR - 1.4).abs() < 1e-10);
    }

    #[test]
    fn test_standard_temperature_value() {
        assert!((STANDARD_TEMPERATURE - 288.15).abs() < 1e-10);
    }

    #[test]
    fn test_standard_pressure_value() {
        assert!((STANDARD_PRESSURE - 101325.0).abs() < 1e-10);
    }

    #[test]
    fn test_standard_density_value() {
        assert!((STANDARD_DENSITY - 1.225).abs() < 1e-10);
    }

    #[test]
    fn test_lapse_rate_value() {
        assert!((STANDARD_LAPSE_RATE - (-0.0065)).abs() < 1e-10);
    }

    #[test]
    fn test_speed_of_sound_sea_level_value() {
        assert!((SPEED_OF_SOUND_SEA_LEVEL - 340.294).abs() < 1e-10);
    }

    #[test]
    fn test_air_viscosity_sea_level_value() {
        assert!((AIR_VISCOSITY_SEA_LEVEL - 1.7894e-5).abs() < 1e-10);
    }

    #[test]
    fn test_earth_radius_value() {
        assert!((EARTH_RADIUS - 6_371_000.0).abs() < 1e-10);
    }

    #[test]
    fn test_earth_mass_value() {
        assert!((EARTH_MASS - 5.9722e24).abs() < 1e-10);
    }

    #[test]
    fn test_gravitational_constant_value() {
        assert!((GRAVITATIONAL_CONSTANT - 6.67430e-11).abs() < 1e-10);
    }
}
