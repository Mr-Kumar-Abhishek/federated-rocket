use crate::types::FlowRegime;

/// Supersonic and compressibility correction factors for aerodynamic
/// coefficients using analytic methods.
pub struct SupersonicCorrections;

impl SupersonicCorrections {
    // ======================================================================
    // Compressibility Corrections (Subsonic)
    // ======================================================================

    /// Prandtl-Glauert correction for subsonic compressibility.
    ///
    /// CNα(M) = CNα(0) / sqrt(1 - M²)  for M < 0.8
    ///
    /// Returns 1.0 / sqrt(1 - M²) as the correction factor to multiply
    /// the incompressible CNα by. For M >= 0.8, returns the value at M=0.8
    /// to avoid the singularity.
    pub fn prandtl_glauert_factor(mach: f64) -> f64 {
        let clamped_mach = mach.clamp(0.0, 0.799);
        1.0 / (1.0 - clamped_mach * clamped_mach).sqrt()
    }

    /// Karman-Tsien correction for subsonic compressibility.
    ///
    /// A more accurate correction than Prandtl-Glauert that accounts for
    /// non-linear effects. Valid for M < 0.8.
    ///
    /// Correction factor = 1 / (sqrt(1 - M²) + M² / (1 + sqrt(1 - M²)))
    pub fn karman_tsien_factor(mach: f64) -> f64 {
        let clamped_mach = mach.clamp(0.0, 0.799);
        let beta = (1.0 - clamped_mach * clamped_mach).sqrt();
        1.0 / (beta + clamped_mach * clamped_mach / (1.0 + beta))
    }

    // ======================================================================
    // Supersonic Normal Force
    // ======================================================================

    /// Supersonic normal force coefficient derivative.
    ///
    /// CNα(M) = 4 / sqrt(M² - 1)  for M > 1.2 (cone-cylinder approximation)
    ///
    /// For M <= 1.2, returns the value at M=1.2 to avoid division by zero.
    pub fn supersonic_normal_force(mach: f64) -> f64 {
        let clamped_mach = mach.max(1.201);
        4.0 / (clamped_mach * clamped_mach - 1.0).sqrt()
    }

    // ======================================================================
    // Transonic Blending
    // ======================================================================

    /// Smoothly blend between subsonic and supersonic values in the
    /// transonic region (0.8 <= M <= 1.2) using a sinusoidal weighting
    /// function.
    ///
    /// At M=0.8, result ≈ subsonic_val; at M=1.2, result ≈ supersonic_val.
    pub fn transonic_blend(mach: f64, subsonic_val: f64, supersonic_val: f64) -> f64 {
        let t = ((mach - 0.8) / 0.4).clamp(0.0, 1.0);
        // Smoothstep: 3t² - 2t³
        let weight = t * t * (3.0 - 2.0 * t);
        subsonic_val * (1.0 - weight) + supersonic_val * weight
    }

    // ======================================================================
    // Wave Drag
    // ======================================================================

    /// Mach-dependent wave drag coefficient for supersonic flow.
    ///
    /// Uses a simplified wave-drag model based on nose fineness ratio:
    ///
    /// CD_wave = 1.5 * (D/Ln)² * (M² - 1)^(-0.5)  for conical nose
    ///
    /// where D is base diameter, Ln is nose length, and fineness = Ln/D.
    /// The drag decreases with increasing fineness ratio (more pointed noses).
    pub fn wave_drag(mach: f64, nose_fineness: f64) -> f64 {
        let regime = FlowRegime::from_mach(mach);
        match regime {
            FlowRegime::Subsonic => 0.0,
            FlowRegime::Transonic => {
                // Blend from 0 at M=0.8 to wave drag at M=1.2
                let t = ((mach - 0.8) / 0.4).clamp(0.0, 1.0);
                let cd_supersonic = Self::wave_drag_supersonic(mach.max(1.201), nose_fineness);
                t * cd_supersonic
            }
            FlowRegime::Supersonic | FlowRegime::Hypersonic => {
                Self::wave_drag_supersonic(mach, nose_fineness)
            }
        }
    }

    /// Wave drag for M > 1.2.
    fn wave_drag_supersonic(mach: f64, nose_fineness: f64) -> f64 {
        if nose_fineness <= 0.0 {
            return 0.0;
        }
        let beta = (mach * mach - 1.0).sqrt();
        if beta <= 0.0 {
            return 0.0;
        }
        1.5 / (nose_fineness * nose_fineness * beta)
    }

    // ======================================================================
    // Base Drag Correction
    // ======================================================================

    /// Base drag correction factor as a function of Mach number.
    ///
    /// Base drag decreases with increasing Mach in the supersonic regime
    /// due to the base pressure recovery. Returns a factor to multiply
    /// the subsonic base drag by.
    ///
    /// Subsonic (M < 0.8):   factor = 1.0
    /// Transonic (0.8-1.2):  smooth transition
    /// Supersonic (M > 1.2): factor = 0.5 + 0.5 * exp(-0.5 * (M - 1.2))
    pub fn base_drag_correction(mach: f64) -> f64 {
        let regime = FlowRegime::from_mach(mach);
        match regime {
            FlowRegime::Subsonic => 1.0,
            FlowRegime::Transonic => {
                let t = ((mach - 0.8) / 0.4).clamp(0.0, 1.0);
                let supersonic_factor = 0.5 + 0.5 * (-0.5 * (mach.max(1.2) - 1.2)).exp();
                1.0 * (1.0 - t) + supersonic_factor * t
            }
            FlowRegime::Supersonic | FlowRegime::Hypersonic => {
                0.5 + 0.5 * (-0.5 * (mach - 1.2)).exp()
            }
        }
    }

    // ======================================================================
    // Skin Friction Correction
    // ======================================================================

    /// Skin friction correction factor with Mach and temperature effects.
    ///
    /// Accounts for compressibility effects on skin friction drag.
    /// Based on the reference temperature method.
    ///
    /// For subsonic: factor ≈ 1.0 (negligible effect)
    /// For supersonic: factor decreases slightly due to boundary layer heating
    pub fn skin_friction_correction(mach: f64, temperature: f64) -> f64 {
        let regime = FlowRegime::from_mach(mach);
        let factor = match regime {
            FlowRegime::Subsonic => 1.0,
            FlowRegime::Transonic => {
                let t = ((mach - 0.8) / 0.4).clamp(0.0, 1.0);
                let supersonic_corr =
                    Self::supersonic_skin_friction_factor(mach.max(1.2), temperature);
                1.0 * (1.0 - t) + supersonic_corr * t
            }
            FlowRegime::Supersonic | FlowRegime::Hypersonic => {
                Self::supersonic_skin_friction_factor(mach, temperature)
            }
        };
        factor.max(0.5).min(1.5)
    }

    /// Reference temperature method skin friction factor for supersonic.
    fn supersonic_skin_friction_factor(mach: f64, temperature: f64) -> f64 {
        if temperature <= 0.0 {
            return 1.0;
        }
        // Recovery temperature ratio (turbulent)
        let prandtl_number: f64 = 0.71; // for air
        let recovery_factor = prandtl_number.powf(1.0 / 3.0);
        let t_ratio = 1.0 + 0.5 * (mach * mach) * (temperature - 1.0) * recovery_factor;
        if t_ratio <= 0.0 {
            return 1.0;
        }
        // Reference temperature ratio (approximate)
        let t_star_ratio = 0.5 + 0.5 * t_ratio + 0.04 * mach * mach;
        if t_star_ratio <= 0.0 {
            return 1.0;
        }
        // Correction factor = (T*/T)^(-0.2) roughly
        t_star_ratio.powf(-0.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======================================================================
    // Prandtl-Glauert
    // ======================================================================

    #[test]
    fn test_prandtl_glauert_at_mach_0() {
        let pg = SupersonicCorrections::prandtl_glauert_factor(0.0);
        assert!((pg - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_prandtl_glauert_at_mach_0_5() {
        let pg = SupersonicCorrections::prandtl_glauert_factor(0.5);
        let expected = 1.0 / (1.0 - 0.25_f64).sqrt();
        assert!((pg - expected).abs() < 1e-10, "PG at M=0.5: {} vs {}", pg, expected);
    }

    #[test]
    fn test_prandtl_glauert_clamps_high_mach() {
        let pg = SupersonicCorrections::prandtl_glauert_factor(0.9);
        let expected = 1.0 / (1.0 - 0.799_f64 * 0.799_f64).sqrt();
        assert!((pg - expected).abs() < 1e-10);
    }

    // ======================================================================
    // Karman-Tsien
    // ======================================================================

    #[test]
    fn test_karman_tsien_at_mach_0() {
        let kt = SupersonicCorrections::karman_tsien_factor(0.0);
        assert!((kt - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_karman_tsien_at_mach_0_5() {
        let kt = SupersonicCorrections::karman_tsien_factor(0.5);
        let beta = (1.0 - 0.25_f64).sqrt();
        let expected = 1.0 / (beta + 0.25 / (1.0 + beta));
        assert!((kt - expected).abs() < 1e-10, "KT at M=0.5: {} vs {}", kt, expected);
    }

    #[test]
    fn test_karman_tsien_differs_from_pg() {
        let pg = SupersonicCorrections::prandtl_glauert_factor(0.6);
        let kt = SupersonicCorrections::karman_tsien_factor(0.6);
        // KT should be slightly different from PG
        assert!((pg - kt).abs() > 1e-6, "KT should differ from PG at M=0.6");
    }

    // ======================================================================
    // Supersonic Normal Force
    // ======================================================================

    #[test]
    fn test_supersonic_normal_force_at_mach_2() {
        let cn = SupersonicCorrections::supersonic_normal_force(2.0);
        let expected = 4.0 / (4.0 - 1.0_f64).sqrt();
        assert!((cn - expected).abs() < 1e-10, "CNα at M=2: {} vs {}", cn, expected);
    }

    #[test]
    fn test_supersonic_normal_force_at_mach_1_5() {
        let cn = SupersonicCorrections::supersonic_normal_force(1.5);
        let expected = 4.0 / (2.25 - 1.0_f64).sqrt();
        assert!((cn - expected).abs() < 1e-10);
    }

    #[test]
    fn test_supersonic_normal_force_clamps_low_mach() {
        let cn = SupersonicCorrections::supersonic_normal_force(0.5);
        // Should return value at M≈1.201
        let expected = 4.0 / (1.201 * 1.201 - 1.0_f64).sqrt();
        assert!((cn - expected).abs() < 1e-10);
    }

    // ======================================================================
    // Transonic Blend
    // ======================================================================

    #[test]
    fn test_transonic_blend_at_mach_0_8() {
        let result = SupersonicCorrections::transonic_blend(0.8, 2.0, 4.0);
        assert!((result - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_transonic_blend_at_mach_1_2() {
        let result = SupersonicCorrections::transonic_blend(1.2, 2.0, 4.0);
        assert!((result - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_transonic_blend_at_mach_1_0() {
        let result = SupersonicCorrections::transonic_blend(1.0, 2.0, 4.0);
        // At M=1.0, t = (1.0-0.8)/0.4 = 0.5, weight = 0.5²*(3-2*0.5) = 0.25*2 = 0.5
        // result = 2*0.5 + 4*0.5 = 3.0
        assert!((result - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_transonic_blend_smoothness() {
        let v1 = SupersonicCorrections::transonic_blend(0.81, 2.0, 4.0);
        let v2 = SupersonicCorrections::transonic_blend(0.82, 2.0, 4.0);
        // Should be smooth (no jumps)
        assert!(v2 > v1);
    }

    // ======================================================================
    // Wave Drag
    // ======================================================================

    #[test]
    fn test_wave_drag_subsonic() {
        let cd = SupersonicCorrections::wave_drag(0.5, 5.0);
        assert_eq!(cd, 0.0);
    }

    #[test]
    fn test_wave_drag_supersonic() {
        let cd = SupersonicCorrections::wave_drag(2.0, 5.0);
        // Should be positive
        assert!(cd > 0.0);
        // Finer nose = less wave drag
        let cd_fine = SupersonicCorrections::wave_drag(2.0, 10.0);
        assert!(cd_fine < cd);
    }

    #[test]
    fn test_wave_drag_zero_fineness() {
        let cd = SupersonicCorrections::wave_drag(2.0, 0.0);
        assert_eq!(cd, 0.0);
    }

    // ======================================================================
    // Base Drag Correction
    // ======================================================================

    #[test]
    fn test_base_drag_correction_subsonic() {
        let f = SupersonicCorrections::base_drag_correction(0.5);
        assert!((f - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_base_drag_correction_supersonic() {
        let f = SupersonicCorrections::base_drag_correction(2.0);
        assert!(f > 0.5);
        assert!(f < 1.0);
    }

    #[test]
    fn test_base_drag_correction_monotonic() {
        let f0 = SupersonicCorrections::base_drag_correction(0.8);
        let f2 = SupersonicCorrections::base_drag_correction(1.2);
        let f3 = SupersonicCorrections::base_drag_correction(2.0);
        let f4 = SupersonicCorrections::base_drag_correction(5.0);
        // Should be non-increasing overall (may have flat regions at 1.0)
        assert!(f0 >= f2 - 1e-12, "f0={} should be >= f2={}", f0, f2);
        assert!(f2 >= f3 - 1e-12, "f2={} should be >= f3={}", f2, f3);
        assert!(f3 >= f4 - 1e-12, "f3={} should be >= f4={}", f3, f4);
    }

    // ======================================================================
    // Skin Friction Correction
    // ======================================================================

    #[test]
    fn test_skin_friction_correction_subsonic() {
        let f = SupersonicCorrections::skin_friction_correction(0.5, 288.15);
        assert!((f - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_skin_friction_correction_supersonic() {
        let f = SupersonicCorrections::skin_friction_correction(2.0, 288.15);
        // Should be between 0.5 and 1.5
        assert!(f >= 0.5);
        assert!(f <= 1.5);
    }

    #[test]
    fn test_skin_friction_correction_clamped() {
        let f = SupersonicCorrections::skin_friction_correction(10.0, 1.0);
        assert!(f >= 0.5);
        assert!(f <= 1.5);
    }
}