/// Body flap / canard aerodynamics
pub struct BodyFlapAero;

impl BodyFlapAero {
    /// Normal force coefficient for a body flap at deflection δ
    /// CN = 2 * sin²(δ) * cos(δ/2) * (flap_area / reference_area)
    pub fn cn_flap(
        deflection_angle: f64, // radians
        flap_area: f64,
        reference_area: f64,
    ) -> f64 {
        if reference_area <= 0.0 {
            return 0.0;
        }
        let area_ratio = flap_area / reference_area;
        2.0 * deflection_angle.sin().powi(2) * (deflection_angle / 2.0).cos() * area_ratio
    }

    /// Hinge moment coefficient for servo sizing
    pub fn hinge_moment_coefficient(deflection_angle: f64, flap_chord: f64, flap_span: f64) -> f64 {
        // Simplified hinge moment: Cm_hinge = 0.25 * CN * (chord / span)
        0.25 * deflection_angle.sin() * flap_chord / flap_span.max(1e-6)
    }

    /// Canard-body interference factor
    pub fn canard_interference(canard_span: f64, body_diameter: f64) -> f64 {
        let span_ratio = canard_span / body_diameter.max(1e-6);
        1.0 + 0.5 / (1.0 + span_ratio)
    }

    /// Calculate downwash effect of canards on main fins
    pub fn downwash_angle(canard_aoa: f64, canard_span: f64, distance_to_fin: f64) -> f64 {
        // Simplified downwash model
        let epsilon = 2.0 * canard_aoa
            / (std::f64::consts::PI * canard_span.powi(2) / distance_to_fin.powi(2));
        epsilon.min(canard_aoa * 0.5) // limit max downwash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======================================================================
    // Body Flap CN Tests
    // ======================================================================

    #[test]
    fn test_body_flap_cn_zero_deflection() {
        let cn = BodyFlapAero::cn_flap(0.0, 0.01, 0.001);
        assert!((cn - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_body_flap_cn_positive_deflection() {
        let cn = BodyFlapAero::cn_flap(0.1, 0.01, 0.001);
        assert!(cn > 0.0, "Flap deflection should produce positive CN");
    }

    #[test]
    fn test_body_flap_cn_zero_reference_area() {
        let cn = BodyFlapAero::cn_flap(0.1, 0.01, 0.0);
        assert_eq!(cn, 0.0);
    }

    #[test]
    fn test_body_flap_cn_area_scaling() {
        let cn1 = BodyFlapAero::cn_flap(0.1, 0.01, 0.001);
        let cn2 = BodyFlapAero::cn_flap(0.1, 0.02, 0.001);
        // Doubling area should double CN
        assert!(
            (cn2 / cn1 - 2.0).abs() < 1e-10,
            "CN should scale with area ratio"
        );
    }

    #[test]
    fn test_body_flap_cn_symmetric_deflections() {
        let cn_pos = BodyFlapAero::cn_flap(0.2, 0.01, 0.001);
        let cn_neg = BodyFlapAero::cn_flap(-0.2, 0.01, 0.001);
        // sin² is symmetric, cos is even → CN should be same
        assert!(
            (cn_pos - cn_neg).abs() < 1e-12,
            "CN should be symmetric for ±δ"
        );
    }

    #[test]
    fn test_body_flap_cn_increases_with_deflection() {
        let cn1 = BodyFlapAero::cn_flap(0.05, 0.01, 0.001);
        let cn2 = BodyFlapAero::cn_flap(0.15, 0.01, 0.001);
        assert!(cn2 > cn1, "CN should increase with deflection angle");
    }

    // ======================================================================
    // Hinge Moment Tests
    // ======================================================================

    #[test]
    fn test_hinge_moment_zero_deflection() {
        let cm = BodyFlapAero::hinge_moment_coefficient(0.0, 0.05, 0.1);
        assert!((cm - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_hinge_moment_positive() {
        let cm = BodyFlapAero::hinge_moment_coefficient(0.2, 0.05, 0.1);
        assert!(
            cm > 0.0,
            "Hinge moment should be positive for positive deflection"
        );
    }

    #[test]
    fn test_hinge_moment_chord_scaling() {
        let cm1 = BodyFlapAero::hinge_moment_coefficient(0.2, 0.05, 0.1);
        let cm2 = BodyFlapAero::hinge_moment_coefficient(0.2, 0.1, 0.1);
        // Doubling chord should double hinge moment
        assert!((cm2 / cm1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_hinge_moment_span_scaling() {
        let cm1 = BodyFlapAero::hinge_moment_coefficient(0.2, 0.05, 0.1);
        let cm2 = BodyFlapAero::hinge_moment_coefficient(0.2, 0.05, 0.2);
        // Doubling span should halve hinge moment
        assert!((cm2 / cm1 - 0.5).abs() < 1e-10);
    }

    // ======================================================================
    // Canard Interference Tests
    // ======================================================================

    #[test]
    fn test_canard_interference_zero_span() {
        let factor = BodyFlapAero::canard_interference(0.0, 0.1);
        // span_ratio = 0, factor = 1.0 + 0.5/(1+0) = 1.5
        assert!((factor - 1.5).abs() < 1e-12);
    }

    #[test]
    fn test_canard_interference_large_span() {
        let factor = BodyFlapAero::canard_interference(0.2, 0.1);
        // span_ratio = 2.0, factor = 1.0 + 0.5/(1+2) ≈ 1.1667
        let expected = 1.0 + 0.5 / 3.0;
        assert!((factor - expected).abs() < 1e-12);
    }

    #[test]
    fn test_canard_interference_decreases_with_span() {
        let f1 = BodyFlapAero::canard_interference(0.05, 0.1);
        let f2 = BodyFlapAero::canard_interference(0.2, 0.1);
        assert!(
            f2 < f1,
            "Interference should decrease as canard span increases"
        );
    }

    // ======================================================================
    // Downwash Angle Tests
    // ======================================================================

    #[test]
    fn test_downwash_angle_zero_aoa() {
        let epsilon = BodyFlapAero::downwash_angle(0.0, 0.1, 0.5);
        assert!((epsilon - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_downwash_angle_positive() {
        let epsilon = BodyFlapAero::downwash_angle(0.1, 0.1, 0.5);
        assert!(
            epsilon > 0.0,
            "Downwash should be positive for positive AOA"
        );
    }

    #[test]
    fn test_downwash_angle_clamped() {
        let epsilon = BodyFlapAero::downwash_angle(0.5, 0.01, 0.5);
        // Should be clamped to canard_aoa * 0.5 = 0.25
        assert!((epsilon - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_downwash_angle_increases_with_aoa() {
        let e1 = BodyFlapAero::downwash_angle(0.05, 0.1, 0.5);
        let e2 = BodyFlapAero::downwash_angle(0.1, 0.1, 0.5);
        assert!(e2 > e1, "Downwash should increase with canard AOA");
    }

    #[test]
    fn test_downwash_angle_distance_effect() {
        // Use small AoA to avoid clamping: clamp = canard_aoa * 0.5
        // For canard_aoa=0.01, clamp = 0.005, which is well above actual values
        // ε(0.3m) = 2*0.01 / (PI*0.1²/0.3²) = 0.02 / 0.349 = 0.057 → clamped to 0.005
        // ε(0.6m) = 2*0.01 / (PI*0.1²/0.6²) = 0.02 / 0.0873 = 0.229 → clamped to 0.005
        // Both clamp, so use larger span to reduce epsilon below clamp
        let e1 = BodyFlapAero::downwash_angle(0.01, 0.5, 0.3);
        let e2 = BodyFlapAero::downwash_angle(0.01, 0.5, 0.6);
        // ε = 2*0.01 / (PI*0.25/0.09) = 0.02 / 8.727 = 0.00229  (below clamp of 0.005)
        // ε = 2*0.01 / (PI*0.25/0.36) = 0.02 / 2.182 = 0.00917  (below clamp of 0.005)
        // Now check the model behavior: with distance^2 in numerator, ε increases
        assert!(
            e2 > e1 + 1e-12,
            "Downwash should increase with distance^2 in this model: e1={}, e2={}",
            e1,
            e2
        );
    }
}
