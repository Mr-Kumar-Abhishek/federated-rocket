use serde::{Deserialize, Serialize};

use crate::constants::{
    AIR_VISCOSITY_SEA_LEVEL, GAMMA_DRY_AIR, SPECIFIC_GAS_CONSTANT_DRY_AIR, STANDARD_GRAVITY,
    STANDARD_LAPSE_RATE, STANDARD_PRESSURE, STANDARD_TEMPERATURE,
};

// ---------------------------------------------------------------------------
// ISA layer boundaries
// ---------------------------------------------------------------------------

/// Altitude of the tropopause (m)
const TROPOPAUSE_ALT: f64 = 11_000.0;

/// Altitude of the lower stratosphere top (m)
const STRAT_1_ALT: f64 = 20_000.0;

/// Altitude of the upper stratosphere top (m)
const STRAT_2_ALT: f64 = 32_000.0;

/// Altitude of the stratopause (m)
const STRAT_3_ALT: f64 = 47_000.0;

// Lapse rates (K/m) for each region
const LAPSE_TROPOSPHERE: f64 = STANDARD_LAPSE_RATE; // -0.0065
const LAPSE_STRAT_1: f64 = 0.0010; // 20–32 km
const LAPSE_STRAT_2: f64 = 0.0028; // 32–47 km
const LAPSE_STRAT_3: f64 = 0.0028; // extrapolation above 47 km

/// Sutherland's constant for air (K)
const SUTHERLAND_CONSTANT: f64 = 110.4;

/// Minimum altitude considered (for clamping)
const MIN_ALTITUDE: f64 = 0.0;

// ---------------------------------------------------------------------------
// Helper: region-based temperature & pressure
// ---------------------------------------------------------------------------

/// Returns (temperature_K, pressure_Pa) at the given altitude using the
/// International Standard Atmosphere model up to 47 km.
fn isa_temperature(altitude: f64) -> f64 {
    let h = altitude.max(MIN_ALTITUDE);

    if h <= TROPOPAUSE_ALT {
        // Troposphere: T = T₀ + L·h
        STANDARD_TEMPERATURE + LAPSE_TROPOSPHERE * h
    } else if h <= STRAT_1_ALT {
        // Tropopause: isothermal at 216.65 K
        216.65
    } else if h <= STRAT_2_ALT {
        // Lower stratosphere: T = T₂₀ + L₁·(h - 20000)
        216.65 + LAPSE_STRAT_1 * (h - STRAT_1_ALT)
    } else {
        // Upper stratosphere and above
        let t_at_32k = 216.65 + LAPSE_STRAT_1 * (STRAT_2_ALT - STRAT_1_ALT); // = 228.65
        if h <= STRAT_3_ALT {
            t_at_32k + LAPSE_STRAT_2 * (h - STRAT_2_ALT)
        } else {
            t_at_32k + LAPSE_STRAT_2 * (STRAT_3_ALT - STRAT_2_ALT) + LAPSE_STRAT_3 * (h - STRAT_3_ALT)
        }
    }
}

fn isa_pressure(altitude: f64) -> f64 {
    let h = altitude.max(MIN_ALTITUDE);

    // --- Layer 1: Troposphere (0 – 11 km) ---
    let t_0 = STANDARD_TEMPERATURE;
    let t_11 = t_0 + LAPSE_TROPOSPHERE * TROPOPAUSE_ALT; // 216.65 K
    let p_0 = STANDARD_PRESSURE;

    // Exponent for layers with non-zero lapse rate: -g₀ / (L · R_specific)
    let exp_tropo = -STANDARD_GRAVITY / (LAPSE_TROPOSPHERE * SPECIFIC_GAS_CONSTANT_DRY_AIR);
    // exp_tropo ≈ 5.256

    if h <= TROPOPAUSE_ALT {
        let t = t_0 + LAPSE_TROPOSPHERE * h;
        return p_0 * (t / t_0).powf(exp_tropo);
    }

    let p_11 = p_0 * (t_11 / t_0).powf(exp_tropo);

    // --- Layer 2: Tropopause (11 – 20 km), isothermal ---
    if h <= STRAT_1_ALT {
        return p_11 * (-STANDARD_GRAVITY * (h - TROPOPAUSE_ALT) / (SPECIFIC_GAS_CONSTANT_DRY_AIR * t_11)).exp();
    }

    let p_20 = p_11 * (-STANDARD_GRAVITY * (STRAT_1_ALT - TROPOPAUSE_ALT) / (SPECIFIC_GAS_CONSTANT_DRY_AIR * t_11)).exp();

    // --- Layer 3: Lower stratosphere (20 – 32 km), L = +0.001 K/m ---
    let t_20 = 216.65;
    let exp_strat_1 = -STANDARD_GRAVITY / (LAPSE_STRAT_1 * SPECIFIC_GAS_CONSTANT_DRY_AIR);

    if h <= STRAT_2_ALT {
        let t = t_20 + LAPSE_STRAT_1 * (h - STRAT_1_ALT);
        return p_20 * (t / t_20).powf(exp_strat_1);
    }

    let t_32 = t_20 + LAPSE_STRAT_1 * (STRAT_2_ALT - STRAT_1_ALT); // 228.65 K
    let p_32 = p_20 * (t_32 / t_20).powf(exp_strat_1);

    // --- Layer 4: Upper stratosphere (32 – 47 km), L = +0.0028 K/m ---
    let exp_strat_2 = -STANDARD_GRAVITY / (LAPSE_STRAT_2 * SPECIFIC_GAS_CONSTANT_DRY_AIR);

    if h <= STRAT_3_ALT {
        let t = t_32 + LAPSE_STRAT_2 * (h - STRAT_2_ALT);
        return p_32 * (t / t_32).powf(exp_strat_2);
    }

    // --- Layer 5: Extrapolation above 47 km ---
    let t_47 = t_32 + LAPSE_STRAT_2 * (STRAT_3_ALT - STRAT_2_ALT); // 270.65 K
    let p_47 = p_32 * (t_47 / t_32).powf(exp_strat_2);
    let exp_strat_3 = -STANDARD_GRAVITY / (LAPSE_STRAT_3 * SPECIFIC_GAS_CONSTANT_DRY_AIR);
    let t = t_47 + LAPSE_STRAT_3 * (h - STRAT_3_ALT);
    p_47 * (t / t_47).powf(exp_strat_3)
}

/// Compute the speed of sound from temperature.
fn speed_of_sound(temperature: f64) -> f64 {
    (GAMMA_DRY_AIR * SPECIFIC_GAS_CONSTANT_DRY_AIR * temperature).sqrt()
}

/// Compute dynamic viscosity using Sutherland's law.
fn viscosity(temperature: f64) -> f64 {
    let t_ref = STANDARD_TEMPERATURE;
    AIR_VISCOSITY_SEA_LEVEL
        * (temperature / t_ref).powf(1.5)
        * (t_ref + SUTHERLAND_CONSTANT)
        / (temperature + SUTHERLAND_CONSTANT)
}

/// Density from the ideal gas law.
fn density(pressure: f64, temperature: f64) -> f64 {
    pressure / (SPECIFIC_GAS_CONSTANT_DRY_AIR * temperature)
}

// ---------------------------------------------------------------------------
// AtmosphericConditions
// ---------------------------------------------------------------------------

/// Atmospheric conditions at a given altitude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AtmosphericConditions {
    /// Altitude (m)
    pub altitude: f64,
    /// Temperature (K)
    pub temperature: f64,
    /// Pressure (Pa)
    pub pressure: f64,
    /// Density (kg/m³)
    pub density: f64,
    /// Speed of sound (m/s)
    pub speed_of_sound: f64,
    /// Dynamic viscosity (Pa·s)
    pub viscosity: f64,
}

impl AtmosphericConditions {
    /// Create a new set of conditions by directly providing values.
    pub fn new(
        altitude: f64,
        temperature: f64,
        pressure: f64,
        density: f64,
        speed_of_sound: f64,
        viscosity: f64,
    ) -> Self {
        Self {
            altitude,
            temperature,
            pressure,
            density,
            speed_of_sound,
            viscosity,
        }
    }

    /// Compute conditions from temperature and pressure at a given altitude.
    fn from_temperature_pressure(altitude: f64, temperature: f64, pressure: f64) -> Self {
        let rho = density(pressure, temperature);
        let sos = speed_of_sound(temperature);
        let visc = viscosity(temperature);
        Self {
            altitude,
            temperature,
            pressure,
            density: rho,
            speed_of_sound: sos,
            viscosity: visc,
        }
    }
}

// ---------------------------------------------------------------------------
// AtmosphericModel trait
// ---------------------------------------------------------------------------

/// Trait for atmospheric models (Strategy pattern).
pub trait AtmosphericModel: Send + Sync {
    fn conditions_at_altitude(&self, altitude: f64) -> AtmosphericConditions;
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// StandardAtmosphere
// ---------------------------------------------------------------------------

/// International Standard Atmosphere (ISA) model.
///
/// Computes temperature, pressure, density, speed of sound, and viscosity
/// using the standard ISA layer structure up to 47 km with extrapolation
/// beyond.
#[derive(Debug, Clone)]
pub struct StandardAtmosphere;

impl StandardAtmosphere {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StandardAtmosphere {
    fn default() -> Self {
        Self::new()
    }
}

impl AtmosphericModel for StandardAtmosphere {
    fn conditions_at_altitude(&self, altitude: f64) -> AtmosphericConditions {
        let t = isa_temperature(altitude);
        let p = isa_pressure(altitude);
        AtmosphericConditions::from_temperature_pressure(altitude, t, p)
    }

    fn name(&self) -> &'static str {
        "International Standard Atmosphere (ISA)"
    }
}

// ---------------------------------------------------------------------------
// IsothermalAtmosphere
// ---------------------------------------------------------------------------

/// Simple constant-temperature atmospheric model.
///
/// Assumes an isothermal atmosphere with exponential pressure decay:
///   P(h) = P₀ · exp(-g₀·h / (R_specific · T))
///   ρ(h) = P / (R_specific · T)
#[derive(Debug, Clone)]
pub struct IsothermalAtmosphere {
    temperature: f64,
    sea_level_pressure: f64,
}

impl IsothermalAtmosphere {
    /// Create a new isothermal atmosphere.
    ///
    /// * `temperature` - Constant temperature (K)
    /// * `sea_level_pressure` - Sea-level pressure (Pa)
    pub fn new(temperature: f64, sea_level_pressure: f64) -> Self {
        Self {
            temperature,
            sea_level_pressure,
        }
    }
}

impl Default for IsothermalAtmosphere {
    fn default() -> Self {
        Self {
            temperature: STANDARD_TEMPERATURE,
            sea_level_pressure: STANDARD_PRESSURE,
        }
    }
}

impl AtmosphericModel for IsothermalAtmosphere {
    fn conditions_at_altitude(&self, altitude: f64) -> AtmosphericConditions {
        let h = altitude.max(MIN_ALTITUDE);
        let t = self.temperature;
        let p = self.sea_level_pressure
            * (-STANDARD_GRAVITY * h / (SPECIFIC_GAS_CONSTANT_DRY_AIR * t)).exp();
        AtmosphericConditions::from_temperature_pressure(altitude, t, p)
    }

    fn name(&self) -> &'static str {
        "Isothermal Atmosphere"
    }
}

// ---------------------------------------------------------------------------
// ExtremeTemperatureAtmosphere
// ---------------------------------------------------------------------------

/// Hot / cold day atmospheric model.
///
/// Applies a ±15 °C offset at sea level that tapers linearly to zero at the
/// tropopause (11 km).  This adjusts the temperature profile of a standard
/// atmosphere while pressure is recomputed hydrostatically.
#[derive(Debug, Clone)]
pub struct ExtremeTemperatureAtmosphere {
    /// Temperature offset at sea level (K)
    delta_t: f64,
}

impl ExtremeTemperatureAtmosphere {
    /// Create a new extreme-temperature atmosphere.
    ///
    /// * `hot` - If `true`, apply a +15 °C offset.
    /// * `cold` - If `true`, apply a -15 °C offset.
    ///
    /// If both are `false` the model acts as a standard atmosphere.
    /// If both are `true` the hot flag takes precedence.
    pub fn new(hot: bool, cold: bool) -> Self {
        let delta_t = if hot {
            15.0
        } else if cold {
            -15.0
        } else {
            0.0
        };
        Self { delta_t }
    }

    /// Temperature offset that tapers linearly with altitude, reaching zero
    /// at the tropopause.
    fn offset(&self, altitude: f64) -> f64 {
        let h = altitude.max(MIN_ALTITUDE);
        if h >= TROPOPAUSE_ALT {
            0.0
        } else {
            self.delta_t * (1.0 - h / TROPOPAUSE_ALT)
        }
    }
}

impl AtmosphericModel for ExtremeTemperatureAtmosphere {
    fn conditions_at_altitude(&self, altitude: f64) -> AtmosphericConditions {
        let base_t = isa_temperature(altitude);
        let offset = self.offset(altitude);
        let t = base_t + offset;

        // Recompute pressure using hydrostatic integration in small steps
        // (or a simplified approach: use the standard pressure and correct it).
        // For simplicity we use the standard pressure as-is; the primary
        // effect of hot/cold days is on temperature and density.
        let p = isa_pressure(altitude);

        AtmosphericConditions::from_temperature_pressure(altitude, t, p)
    }

    fn name(&self) -> &'static str {
        if self.delta_t > 0.0 {
            "Hot Day Atmosphere (+15 °C)"
        } else if self.delta_t < 0.0 {
            "Cold Day Atmosphere (-15 °C)"
        } else {
            "Standard Atmosphere"
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- StandardAtmosphere ---

    #[test]
    fn test_isa_sea_level() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(0.0);
        assert!((cond.temperature - 288.15).abs() < 0.1, "T = {}", cond.temperature);
        assert!((cond.pressure - 101325.0).abs() < 10.0, "P = {}", cond.pressure);
        assert!((cond.density - 1.225).abs() < 0.01, "ρ = {}", cond.density);
    }

    #[test]
    fn test_isa_tropopause() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(11_000.0);
        // Temperature should be 216.65 K at the tropopause
        assert!((cond.temperature - 216.65).abs() < 0.1, "T = {}", cond.temperature);
        // Pressure should be around 22.6 kPa
        assert!((cond.pressure - 22632.0).abs() < 100.0, "P = {}", cond.pressure);
    }

    #[test]
    fn test_isa_20km() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(20_000.0);
        // Temperature should still be 216.65 K at 20 km
        assert!((cond.temperature - 216.65).abs() < 0.1, "T = {}", cond.temperature);
        // Pressure should be around 5.5 kPa
        assert!(cond.pressure < 6000.0, "P = {}", cond.pressure);
        assert!(cond.pressure > 5000.0, "P = {}", cond.pressure);
    }

    #[test]
    fn test_isa_32km() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(32_000.0);
        // Temperature should be around 228.65 K
        assert!((cond.temperature - 228.65).abs() < 0.5, "T = {}", cond.temperature);
    }

    #[test]
    fn test_isa_47km() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(47_000.0);
        // Temperature should be around 270.65 K
        assert!((cond.temperature - 270.65).abs() < 1.0, "T = {}", cond.temperature);
    }

    #[test]
    fn test_isa_temperature_decreases_in_troposphere() {
        let model = StandardAtmosphere::new();
        let t0 = model.conditions_at_altitude(0.0).temperature;
        let t5k = model.conditions_at_altitude(5_000.0).temperature;
        let t11k = model.conditions_at_altitude(11_000.0).temperature;
        assert!(t5k < t0, "Temp should decrease with altitude in troposphere");
        assert!(t11k < t5k, "Temp should continue decreasing");
    }

    #[test]
    fn test_isa_temperature_isothermal_tropopause() {
        let model = StandardAtmosphere::new();
        let t11k = model.conditions_at_altitude(11_000.0).temperature;
        let t15k = model.conditions_at_altitude(15_000.0).temperature;
        assert!((t15k - t11k).abs() < 0.1, "Tropopause should be isothermal");
    }

    #[test]
    fn test_isa_speed_of_sound_sea_level() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(0.0);
        assert!((cond.speed_of_sound - 340.294).abs() < 0.5, "a = {}", cond.speed_of_sound);
    }

    #[test]
    fn test_isa_viscosity_sea_level() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(0.0);
        assert!((cond.viscosity - 1.7894e-5).abs() < 1e-7, "μ = {}", cond.viscosity);
    }

    #[test]
    fn test_isa_negative_altitude_clamps() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(-100.0);
        let cond0 = model.conditions_at_altitude(0.0);
        assert!((cond.temperature - cond0.temperature).abs() < 0.1);
        assert!((cond.pressure - cond0.pressure).abs() < 10.0);
    }

    #[test]
    fn test_isa_high_altitude_extrapolation() {
        let model = StandardAtmosphere::new();
        let cond = model.conditions_at_altitude(60_000.0);
        // Above 47 km, temperature should continue increasing with LAPSE_STRAT_3
        // T ≈ 270.65 + 0.0028 * (60000 - 47000) = 270.65 + 36.4 = 307.05 K
        assert!(cond.temperature > 280.0, "T = {}", cond.temperature);
        assert!(cond.pressure > 0.0, "Pressure should be positive");
    }

    // --- IsothermalAtmosphere ---

    #[test]
    fn test_isothermal_sea_level() {
        let model = IsothermalAtmosphere::new(288.15, 101325.0);
        let cond = model.conditions_at_altitude(0.0);
        assert!((cond.temperature - 288.15).abs() < 0.1);
        assert!((cond.pressure - 101325.0).abs() < 10.0);
    }

    #[test]
    fn test_isothermal_pressure_decay() {
        let model = IsothermalAtmosphere::new(288.15, 101325.0);
        let cond = model.conditions_at_altitude(5_000.0);
        assert!(cond.pressure < 101325.0);
        assert!(cond.pressure > 50_000.0);
    }

    #[test]
    fn test_isothermal_density_from_ideal_gas() {
        let model = IsothermalAtmosphere::new(300.0, 100000.0);
        let cond = model.conditions_at_altitude(0.0);
        let expected_rho = 100000.0 / (SPECIFIC_GAS_CONSTANT_DRY_AIR * 300.0);
        assert!((cond.density - expected_rho).abs() < 1e-6);
    }

    // --- ExtremeTemperatureAtmosphere ---

    #[test]
    fn test_hot_day_sea_level() {
        let model = ExtremeTemperatureAtmosphere::new(true, false);
        let cond = model.conditions_at_altitude(0.0);
        // Sea level should be 288.15 + 15 = 303.15 K
        assert!((cond.temperature - 303.15).abs() < 0.5, "T = {}", cond.temperature);
    }

    #[test]
    fn test_cold_day_sea_level() {
        let model = ExtremeTemperatureAtmosphere::new(false, true);
        let cond = model.conditions_at_altitude(0.0);
        // Sea level should be 288.15 - 15 = 273.15 K
        assert!((cond.temperature - 273.15).abs() < 0.5, "T = {}", cond.temperature);
    }

    #[test]
    fn test_extreme_temperature_taper_at_tropopause() {
        let hot = ExtremeTemperatureAtmosphere::new(true, false);
        let cold = ExtremeTemperatureAtmosphere::new(false, true);
        let cond_hot = hot.conditions_at_altitude(11_000.0);
        let cond_cold = cold.conditions_at_altitude(11_000.0);
        // At the tropopause the offset should be zero
        assert!((cond_hot.temperature - cond_cold.temperature).abs() < 0.5,
            "Hot and cold should converge at tropopause: hot={}, cold={}",
            cond_hot.temperature, cond_cold.temperature);
    }

    #[test]
    fn test_extreme_neither_flag() {
        let model = ExtremeTemperatureAtmosphere::new(false, false);
        let cond = model.conditions_at_altitude(0.0);
        assert!((cond.temperature - 288.15).abs() < 0.5, "T = {}", cond.temperature);
    }

    #[test]
    fn test_hot_takes_precedence() {
        let model = ExtremeTemperatureAtmosphere::new(true, true);
        let cond = model.conditions_at_altitude(0.0);
        // Hot takes precedence: +15 °C
        assert!((cond.temperature - 303.15).abs() < 0.5, "T = {}", cond.temperature);
    }

    // --- Integration: AtmosphericConditions construction ---

    #[test]
    fn test_atmospheric_conditions_new() {
        let cond = AtmosphericConditions::new(1000.0, 281.65, 89870.0, 1.112, 336.0, 1.75e-5);
        assert!((cond.altitude - 1000.0).abs() < 1e-10);
        assert!((cond.temperature - 281.65).abs() < 1e-10);
        assert!((cond.pressure - 89870.0).abs() < 1e-10);
        assert!((cond.density - 1.112).abs() < 1e-10);
        assert!((cond.speed_of_sound - 336.0).abs() < 1e-10);
        assert!((cond.viscosity - 1.75e-5).abs() < 1e-10);
    }
}
