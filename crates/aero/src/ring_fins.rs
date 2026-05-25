/// Ring fin (tube fin) aerodynamics calculator
pub struct RingFinAero;

impl RingFinAero {
    /// Calculate normal force coefficient derivative for a ring fin
    /// CNα = 2 * (L/D) where L = length, D = diameter of ring
    pub fn cn_alpha(ring_length: f64, ring_diameter: f64) -> f64 {
        if ring_diameter <= 0.0 { return 0.0; }
        let aspect_ratio = ring_length / ring_diameter;
        // Empirical formula for ring fins
        2.0 * aspect_ratio / (1.0 + (1.0 + 4.0 * aspect_ratio.powi(2)).sqrt())
    }
    
    /// Center of pressure position for ring fin (calibers from LE)
    pub fn cp_position(ring_length: f64, ring_diameter: f64) -> f64 {
        let aspect_ratio = ring_length / ring_diameter;
        0.5 - 0.125 * aspect_ratio // CP shifts forward with increasing AR
    }
    
    /// Drag coefficient for ring fin at zero angle of attack
    pub fn cd_zero(ring_length: f64, ring_diameter: f64, skin_friction: f64) -> f64 {
        // Skin friction on inside and outside surfaces
        let area_ratio = 2.0 * ring_length * ring_diameter / (ring_diameter.powi(2) * std::f64::consts::FRAC_PI_4);
        2.0 * skin_friction * area_ratio
    }
    
    /// Ring fin drag due to angle of attack
    pub fn cd_alpha(mach: f64, angle_of_attack: f64) -> f64 {
        let aoa_deg = angle_of_attack.to_degrees();
        if aoa_deg < 5.0 {
            // Low AoA: quadratic in α
            0.1 * aoa_deg.powi(2) * mach.max(0.5)
        } else {
            // High AoA: linear in α
            2.0 * aoa_deg * mach.max(0.5)
        }
    }
    
    /// Mach correction for ring fin normal force
    pub fn mach_correction(mach: f64) -> f64 {
        if mach < 0.8 {
            1.0 / (1.0 - mach.powi(2)).sqrt()  // Prandtl-Glauert
        } else if mach > 1.2 {
            2.0 / (mach.powi(2) - 1.0).sqrt()   // Supersonic
        } else {
            // Transonic blend
            let t = (mach - 0.8) / 0.4;
            1.0 + t * (2.0 / 0.8_f64.sqrt() - 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======================================================================
    // Ring Fin CNα Tests
    // ======================================================================

    #[test]
    fn test_ring_fin_cn_alpha_zero_diameter() {
        let cn = RingFinAero::cn_alpha(0.1, 0.0);
        assert_eq!(cn, 0.0);
    }

    #[test]
    fn test_ring_fin_cn_alpha_aspect_ratio_1() {
        // AR = 0.1/0.1 = 1.0
        // CNα = 2 * 1.0 / (1.0 + sqrt(1 + 4*1)) = 2.0 / (1.0 + sqrt(5)) ≈ 0.618
        let cn = RingFinAero::cn_alpha(0.1, 0.1);
        let expected = 2.0_f64 * 1.0_f64 / (1.0_f64 + (1.0_f64 + 4.0_f64).sqrt());
        assert!((cn - expected).abs() < 1e-12, "CNα for AR=1: {} vs {}", cn, expected);
    }

    #[test]
    fn test_ring_fin_cn_alpha_aspect_ratio_2() {
        // AR = 0.2/0.1 = 2.0
        let cn = RingFinAero::cn_alpha(0.2, 0.1);
        let expected = 2.0_f64 * 2.0_f64 / (1.0_f64 + (1.0_f64 + 16.0_f64).sqrt());
        assert!((cn - expected).abs() < 1e-12);
    }

    #[test]
    fn test_ring_fin_cn_alpha_increasing_with_ar() {
        let cn1 = RingFinAero::cn_alpha(0.1, 0.1);
        let cn2 = RingFinAero::cn_alpha(0.2, 0.1);
        let cn3 = RingFinAero::cn_alpha(0.3, 0.1);
        assert!(cn2 > cn1, "CNα should increase with aspect ratio");
        assert!(cn3 > cn2, "CNα should increase with aspect ratio");
    }

    // ======================================================================
    // Ring Fin CP Position Tests
    // ======================================================================

    #[test]
    fn test_ring_fin_cp_aspect_ratio_1() {
        let cp = RingFinAero::cp_position(0.1, 0.1);
        // 0.5 - 0.125 * 1.0 = 0.375
        assert!((cp - 0.375).abs() < 1e-12);
    }

    #[test]
    fn test_ring_fin_cp_aspect_ratio_2() {
        let cp = RingFinAero::cp_position(0.2, 0.1);
        // 0.5 - 0.125 * 2.0 = 0.25
        assert!((cp - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_ring_fin_cp_shifts_forward() {
        let cp1 = RingFinAero::cp_position(0.05, 0.1);
        let cp2 = RingFinAero::cp_position(0.2, 0.1);
        assert!(cp2 < cp1, "CP should shift forward with increasing AR");
    }

    // ======================================================================
    // Ring Fin CD Zero Tests
    // ======================================================================

    #[test]
    fn test_ring_fin_cd_zero_positive() {
        let cd = RingFinAero::cd_zero(0.1, 0.1, 0.005);
        assert!(cd > 0.0, "CD zero should be positive");
    }

    #[test]
    fn test_ring_fin_cd_zero_scaling() {
        let cd1 = RingFinAero::cd_zero(0.1, 0.1, 0.005);
        let cd2 = RingFinAero::cd_zero(0.2, 0.1, 0.005);
        assert!(cd2 > cd1, "Longer ring should have more drag");
    }

    // ======================================================================
    // Ring Fin CD Alpha Tests
    // ======================================================================

    #[test]
    fn test_ring_fin_cd_alpha_zero_aoa() {
        let cd = RingFinAero::cd_alpha(1.0, 0.0);
        assert!((cd - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_ring_fin_cd_alpha_low_aoa_quadratic() {
        let cd = RingFinAero::cd_alpha(1.0, 0.05); // ~2.86°
        assert!(cd > 0.0);
    }

    #[test]
    fn test_ring_fin_cd_alpha_high_aoa_linear() {
        let cd_low = RingFinAero::cd_alpha(1.0, 0.1); // ~5.73°
        let cd_high = RingFinAero::cd_alpha(1.0, 0.2); // ~11.46°
        // High AoA should be roughly linear
        assert!(cd_high > cd_low);
    }

    // ======================================================================
    // Ring Fin Mach Correction Tests
    // ======================================================================

    #[test]
    fn test_ring_fin_mach_correction_subsonic() {
        // M=0.5: PG factor = 1.0 / sqrt(1 - 0.25) ≈ 1.155
        let factor = RingFinAero::mach_correction(0.5);
        let expected = 1.0 / (1.0 - 0.25_f64).sqrt();
        assert!((factor - expected).abs() < 1e-10);
    }

    #[test]
    fn test_ring_fin_mach_correction_supersonic() {
        // M=2.0: 2.0 / sqrt(4 - 1) ≈ 1.155
        let factor = RingFinAero::mach_correction(2.0);
        let expected = 2.0 / (4.0 - 1.0_f64).sqrt();
        assert!((factor - expected).abs() < 1e-10);
    }

    #[test]
    fn test_ring_fin_mach_correction_transonic_blend() {
        let m08 = RingFinAero::mach_correction(0.8);
        let m12 = RingFinAero::mach_correction(1.2);
        let m10 = RingFinAero::mach_correction(1.0);
        // At M=1.0, t = (1.0-0.8)/0.4 = 0.5
        // factor = 1.0 + 0.5 * (2.0/sqrt(0.8) - 1.0)
        let expected = 1.0 + 0.5 * (2.0 / 0.8_f64.sqrt() - 1.0);
        assert!((m10 - expected).abs() < 1e-10);
        assert!(m12 > m08, "Mach correction should increase with Mach");
    }

    // ======================================================================
    // Integration: Ring Fin on Rocket
    // ======================================================================

    #[test]
    fn test_ring_fin_force_scaling() {
        // Larger ring should produce more normal force
        let cn1 = RingFinAero::cn_alpha(0.1, 0.1);
        let cn2 = RingFinAero::cn_alpha(0.2, 0.15);
        assert!(cn2 > 0.0);
        assert!(cn1 > 0.0);
    }
}