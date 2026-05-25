use serde::{Deserialize, Serialize};

/// Interference factors between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceFactors {
    /// Fin-body interference factor (typically 1.0-1.3)
    pub fin_body: f64,
    /// Nose-body interference factor
    pub nose_body: f64,
    /// Body-fin interference factor (affects fin effectiveness)
    pub body_fin: f64,
    /// Fin-fin interference (for closely spaced fins)
    pub fin_fin: f64,
    /// Transition interference factor
    pub transition: f64,
    /// Staging gap interference
    pub staging_gap: f64,
    /// Pod interference
    pub pod: f64,
}

impl Default for InterferenceFactors {
    fn default() -> Self {
        Self {
            fin_body: 1.15, // ~15% increase in fin CNα due to body presence
            nose_body: 1.0,
            body_fin: 1.25, // body affects fin lift by ~25%
            fin_fin: 1.0,
            transition: 1.0,
            staging_gap: 0.95, // staging gaps slightly reduce effectiveness
            pod: 1.05,
        }
    }
}

/// Calculate fin-body interference using the method from Barrowman
/// β = 1 + (r / (r + s)) where r = body radius, s = fin span
pub fn fin_body_interference_factor(body_radius: f64, fin_span: f64) -> f64 {
    if fin_span <= 0.0 {
        return 1.0;
    }
    1.0 + body_radius / (body_radius + fin_span)
}

/// Calculate body-fin interference factor (Barnwell-Sewell method)
/// This accounts for the body's influence on fin lift
pub fn body_fin_interference_factor(fin_count: u32) -> f64 {
    match fin_count {
        2 => 1.0,
        3 => 1.25,
        4 => 1.5,
        6 => 1.75,
        8 => 2.0,
        _ => 1.0 + 0.125 * (fin_count as f64),
    }
}

/// Calculate base drag correction for a boattail or nozzle
pub fn base_drag_stagger(base_area_ratio: f64, exit_area_ratio: f64) -> f64 {
    // Based on Hoerner's base drag theory
    let ratio = base_area_ratio.min(exit_area_ratio) / base_area_ratio.max(exit_area_ratio);
    1.0 - ratio * 0.5
}

/// Calculate the interference drag for a pod mounted on a body
pub fn pod_interference_drag(pod_diameter: f64, body_diameter: f64) -> f64 {
    let d_ratio = pod_diameter / body_diameter;
    0.5 * d_ratio * (1.0 - d_ratio) // peaks at d_ratio = 0.5
}

/// Calculate stagger correction for multi-stage rockets
pub fn staging_gap_correction(gap: f64, body_diameter: f64) -> f64 {
    let gap_ratio = (gap / body_diameter).clamp(0.1, 5.0);
    1.0 - 0.15 / gap_ratio // small gaps reduce effectiveness
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======================================================================
    // Fin-Body Interference Tests
    // ======================================================================

    #[test]
    fn test_fin_body_interference_zero_span() {
        let factor = fin_body_interference_factor(0.02, 0.0);
        assert!((factor - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_fin_body_interference_equal_radius_span() {
        // body_radius = fin_span = 0.02 → β = 1 + 0.02/(0.02+0.02) = 1.5
        let factor = fin_body_interference_factor(0.02, 0.02);
        assert!((factor - 1.5).abs() < 1e-12);
    }

    #[test]
    fn test_fin_body_interference_large_span() {
        // body_radius = 0.02, fin_span = 0.1 → β = 1 + 0.02/(0.02+0.1) = 1.1667
        let factor = fin_body_interference_factor(0.02, 0.1);
        let expected = 1.0 + 0.02 / 0.12;
        assert!((factor - expected).abs() < 1e-12);
    }

    #[test]
    fn test_fin_body_interference_large_body() {
        let factor = fin_body_interference_factor(0.1, 0.02);
        let expected = 1.0 + 0.1 / 0.12;
        assert!((factor - expected).abs() < 1e-12);
    }

    // ======================================================================
    // Body-Fin Interference Tests
    // ======================================================================

    #[test]
    fn test_body_fin_interference_2_fins() {
        let factor = body_fin_interference_factor(2);
        assert!((factor - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_body_fin_interference_4_fins() {
        let factor = body_fin_interference_factor(4);
        assert!((factor - 1.5).abs() < 1e-12);
    }

    #[test]
    fn test_body_fin_interference_6_fins() {
        let factor = body_fin_interference_factor(6);
        assert!((factor - 1.75).abs() < 1e-12);
    }

    #[test]
    fn test_body_fin_interference_3_fins() {
        let factor = body_fin_interference_factor(3);
        assert!((factor - 1.25).abs() < 1e-12);
    }

    #[test]
    fn test_body_fin_interference_unknown_count() {
        let factor_5 = body_fin_interference_factor(5);
        let expected_5 = 1.0 + 0.125 * 5.0;
        assert!((factor_5 - expected_5).abs() < 1e-12);

        let factor_7 = body_fin_interference_factor(7);
        let expected_7 = 1.0 + 0.125 * 7.0;
        assert!((factor_7 - expected_7).abs() < 1e-12);
    }

    // ======================================================================
    // Base Drag Stagger Tests
    // ======================================================================

    #[test]
    fn test_base_drag_stagger_equal_areas() {
        let factor = base_drag_stagger(1.0, 1.0);
        // ratio = 1.0, factor = 1.0 - 1.0 * 0.5 = 0.5
        assert!((factor - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_base_drag_stagger_different_areas() {
        let factor = base_drag_stagger(0.5, 0.8);
        let ratio = 0.5 / 0.8;
        let expected = 1.0 - ratio * 0.5;
        assert!((factor - expected).abs() < 1e-12);
    }

    // ======================================================================
    // Pod Interference Drag Tests
    // ======================================================================

    #[test]
    fn test_pod_interference_drag_zero_diameter() {
        let drag = pod_interference_drag(0.0, 0.1);
        assert!((drag - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_pod_interference_drag_equal_diameters() {
        let drag = pod_interference_drag(0.1, 0.1);
        // 0.5 * 1.0 * (1.0 - 1.0) = 0.0
        assert!((drag - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_pod_interference_drag_peak() {
        let drag = pod_interference_drag(0.05, 0.1);
        // 0.5 * 0.5 * (1.0 - 0.5) = 0.125
        assert!((drag - 0.125).abs() < 1e-12);
    }

    // ======================================================================
    // Staging Gap Correction Tests
    // ======================================================================

    #[test]
    fn test_staging_gap_correction_small_gap() {
        let correction = staging_gap_correction(0.01, 0.1);
        // gap_ratio clamped to 0.1, correction = 1.0 - 0.15/0.1 = -0.5
        assert!((correction - (-0.5)).abs() < 1e-12);
    }

    #[test]
    fn test_staging_gap_correction_large_gap() {
        let correction = staging_gap_correction(1.0, 0.1);
        // gap_ratio = 10.0, clamped to 5.0, correction = 1.0 - 0.15/5.0 = 0.97
        assert!((correction - 0.97).abs() < 1e-12);
    }

    #[test]
    fn test_staging_gap_correction_unit_gap() {
        let correction = staging_gap_correction(0.1, 0.1);
        // gap_ratio = 1.0, correction = 1.0 - 0.15/1.0 = 0.85
        assert!((correction - 0.85).abs() < 1e-12);
    }

    // ======================================================================
    // InterferenceFactors Default Tests
    // ======================================================================

    #[test]
    fn test_interference_factors_default() {
        let factors = InterferenceFactors::default();
        assert!((factors.fin_body - 1.15).abs() < 1e-12);
        assert!((factors.nose_body - 1.0).abs() < 1e-12);
        assert!((factors.body_fin - 1.25).abs() < 1e-12);
        assert!((factors.fin_fin - 1.0).abs() < 1e-12);
        assert!((factors.transition - 1.0).abs() < 1e-12);
        assert!((factors.staging_gap - 0.95).abs() < 1e-12);
        assert!((factors.pod - 1.05).abs() < 1e-12);
    }

    #[test]
    fn test_interference_factors_serde_roundtrip() {
        let factors = InterferenceFactors::default();
        let json = serde_json::to_string(&factors).unwrap();
        let deser: InterferenceFactors = serde_json::from_str(&json).unwrap();
        assert!((deser.fin_body - factors.fin_body).abs() < 1e-12);
        assert!((deser.staging_gap - factors.staging_gap).abs() < 1e-12);
    }
}
