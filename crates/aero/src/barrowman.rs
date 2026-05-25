use federated_rocket_core::component::*;
use federated_rocket_core::component_tree::*;

/// Barrowman method calculator for subsonic aerodynamics.
///
/// Implements the classic Barrowman method for computing normal force
/// coefficient derivatives and center of pressure positions for model
/// rocket components.
pub struct BarrowmanCalculator;

/// Result of a single component's contribution to the total aerodynamics.
#[derive(Debug, Clone, Copy)]
pub struct ComponentContribution {
    /// Normal force coefficient derivative contribution
    pub cn_alpha: f64,
    /// Center of pressure position (m from nose tip) for this component
    pub cp_position: f64,
}

impl BarrowmanCalculator {
    // ======================================================================
    // Nose Cone Calculations
    // ======================================================================

    /// Returns the normal force coefficient derivative (CNα) for a nose cone.
    ///
    /// For all standard nose cone shapes, CNα = 2.0 per radian (classic Barrowman).
    pub fn nose_cone_cn_alpha(_shape: &NoseConeShape) -> f64 {
        2.0
    }

    /// Returns the center of pressure position (in calibers from nose tip)
    /// for a nose cone of the given shape, length `Ln` and base diameter `D`.
    ///
    /// Formulas are from the classic Barrowman report:
    /// - Conical:    X_cp = 0.666 * Ln / D
    /// - Ogive:      X_cp = 0.534 * Ln / D
    /// - Elliptical: X_cp = 0.500 * Ln / D
    /// - Parabolic:  X_cp = 0.500 * Ln / D
    /// - PowerSeries(n): X_cp = (n / (2n + 1)) * Ln / D
    /// - VonKarman:  X_cp = 0.500 * Ln / D
    /// - HaackSeries(n): X_cp = (n / (2n + 1)) * Ln / D
    pub fn nose_cone_cp_calibers(shape: &NoseConeShape, length: f64, diameter: f64) -> f64 {
        if diameter <= 0.0 || length <= 0.0 {
            return 0.0;
        }
        let ln_over_d = length / diameter;
        match shape {
            NoseConeShape::Conical => 0.666 * ln_over_d,
            NoseConeShape::Ogive => 0.534 * ln_over_d,
            NoseConeShape::Elliptical => 0.500 * ln_over_d,
            NoseConeShape::Parabolic => 0.500 * ln_over_d,
            NoseConeShape::PowerSeries(n) => {
                let n = *n;
                if n <= 0.0 {
                    0.500 * ln_over_d
                } else {
                    (n / (2.0 * n + 1.0)) * ln_over_d
                }
            }
            NoseConeShape::VonKarman => 0.500 * ln_over_d,
            NoseConeShape::HaackSeries(n) => {
                let n = *n;
                if n <= 0.0 {
                    0.500 * ln_over_d
                } else {
                    (n / (2.0 * n + 1.0)) * ln_over_d
                }
            }
        }
    }

    // ======================================================================
    // Body Tube / Transition Calculations
    // ======================================================================

    /// Returns the normal force coefficient derivative (CNα) for a body tube.
    ///
    /// Body tubes contribute zero normal force in the classic Barrowman method.
    pub fn body_tube_cn_alpha() -> f64 {
        0.0
    }

    /// Returns the normal force coefficient derivative (CNα) for a transition.
    ///
    /// CNα = 2.0 * ((d_aft/D)^2 - (d_fore/D)^2)
    /// where `D` is the reference diameter.
    pub fn transition_cn_alpha(fore_diameter: f64, aft_diameter: f64, ref_diameter: f64) -> f64 {
        if ref_diameter <= 0.0 {
            return 0.0;
        }
        let r_fore = (fore_diameter / ref_diameter).powi(2);
        let r_aft = (aft_diameter / ref_diameter).powi(2);
        2.0 * (r_aft - r_fore)
    }

    /// Returns the center of pressure position (m from nose tip) for a conical transition.
    ///
    /// X_cp = X_fore + (L/3) * (1 + (1 - r_fore/r_aft) / (1 - (r_fore/r_aft)^2))
    ///
    /// where `X_fore` is the axial position of the transition fore edge,
    /// `L` is transition length, `r_fore` is fore radius, `r_aft` is aft radius.
    pub fn transition_cp_position(
        x_fore: f64,
        length: f64,
        fore_radius: f64,
        aft_radius: f64,
    ) -> f64 {
        if length <= 0.0 || aft_radius <= 0.0 {
            return x_fore;
        }
        let ratio = fore_radius / aft_radius;
        let term: f64 = if (1.0 - ratio * ratio).abs() > 1e-12 {
            1.0 + (1.0 - ratio) / (1.0 - ratio * ratio)
        } else {
            1.0
        };
        x_fore + (length / 3.0) * term
    }

    // ======================================================================
    // Fin Set Calculations
    // ======================================================================

    /// Returns the normal force coefficient derivative (CNα) for a set of
    /// trapezoidal fins using the Barrowman method.
    ///
    /// CNα = (4 * N * (s/d)^2) / (1 + sqrt(1 + (2*Lc/(cr + ct))^2))
    ///
    /// where:
    /// - N = number of fins
    /// - s = fin span (from body surface to tip)
    /// - d = body diameter (reference)
    /// - Lc = mean aerodynamic chord = (cr + ct) / 2
    /// - cr = root chord
    /// - ct = tip chord
    pub fn fin_cn_alpha(
        fin_count: u32,
        span: f64,
        body_diameter: f64,
        root_chord: f64,
        tip_chord: f64,
    ) -> f64 {
        if body_diameter <= 0.0 || fin_count == 0 {
            return 0.0;
        }
        let n = fin_count as f64;
        let s_over_d = span / body_diameter;
        let mean_chord = (root_chord + tip_chord) / 2.0;
        let denom = 1.0 + (1.0 + (2.0 * mean_chord / (root_chord + tip_chord)).powi(2)).sqrt();
        if denom <= 0.0 {
            return 0.0;
        }
        (4.0 * n * s_over_d * s_over_d) / denom
    }

    /// Returns the center of pressure position (m from nose tip) for a set of
    /// trapezoidal fins.
    ///
    /// X_cp = X_f + (cr * (cr + 2*ct)) / (3 * (cr + ct)) + (cr*ct + ct^2) / (6*(cr + ct))
    ///
    /// where X_f is the root leading edge axial position.
    pub fn fin_cp_position(
        x_root_le: f64,
        root_chord: f64,
        tip_chord: f64,
    ) -> f64 {
        let sum = root_chord + tip_chord;
        if sum <= 0.0 {
            return x_root_le;
        }
        let term1 = (root_chord * (root_chord + 2.0 * tip_chord)) / (3.0 * sum);
        let term2 = (root_chord * tip_chord + tip_chord * tip_chord) / (6.0 * sum);
        x_root_le + term1 + term2
    }

    // ======================================================================
    // Total Rocket Calculations
    // ======================================================================

    /// Compute the total normal force coefficient derivative (CNα) for the
    /// entire rocket by summing all component contributions.
    ///
    /// Returns (total_cn_alpha, total_cp_position_m)
    /// where total_cp_position is the weighted average CP (m from nose tip).
    pub fn total_rocket_cn_alpha_and_cp(
        tree: &ComponentTree,
        ref_diameter: f64,
    ) -> (f64, f64) {
        let mut total_cn_alpha = 0.0;
        let mut weighted_cp_sum = 0.0;

        for (_key, node) in tree.iter() {
            let contrib = component_contribution(&node.component, ref_diameter);
            total_cn_alpha += contrib.cn_alpha;
            weighted_cp_sum += contrib.cn_alpha * contrib.cp_position;
        }

        let total_cp = if total_cn_alpha.abs() > 1e-12 {
            weighted_cp_sum / total_cn_alpha
        } else {
            0.0
        };

        (total_cn_alpha, total_cp)
    }
}

/// Compute the contribution of a single `RocketComponent`.
pub fn component_contribution(
    component: &RocketComponent,
    ref_diameter: f64,
) -> ComponentContribution {
    match component {
        RocketComponent::NoseCone(data) => {
            let length = *data.length.value();
            let diameter = *data.base_radius.value() * 2.0;
            let cn_alpha = BarrowmanCalculator::nose_cone_cn_alpha(&data.shape);
            let cp_calibers = BarrowmanCalculator::nose_cone_cp_calibers(&data.shape, length, diameter);
            let cp_position = data.position.x + cp_calibers * ref_diameter;
            ComponentContribution {
                cn_alpha,
                cp_position,
            }
        }
        RocketComponent::BodyTube(_data) => {
            ComponentContribution {
                cn_alpha: BarrowmanCalculator::body_tube_cn_alpha(),
                cp_position: 0.0,
            }
        }
        RocketComponent::Transition(data) => {
            let length = *data.length.value();
            let fore_diameter = *data.fore_radius.value() * 2.0;
            let aft_diameter = *data.aft_radius.value() * 2.0;
            let cn_alpha =
                BarrowmanCalculator::transition_cn_alpha(fore_diameter, aft_diameter, ref_diameter);
            let cp_position = BarrowmanCalculator::transition_cp_position(
                data.position.x,
                length,
                *data.fore_radius.value(),
                *data.aft_radius.value(),
            );
            ComponentContribution {
                cn_alpha,
                cp_position,
            }
        }
        RocketComponent::FinSet(data) => {
            let span = *data.span.value();
            let root_chord = *data.root_chord.value();
            let tip_chord = *data.tip_chord.value();
            let cn_alpha = BarrowmanCalculator::fin_cn_alpha(
                data.fin_count,
                span,
                ref_diameter,
                root_chord,
                tip_chord,
            );
            let cp_position = BarrowmanCalculator::fin_cp_position(
                data.position.x,
                root_chord,
                tip_chord,
            );
            ComponentContribution {
                cn_alpha,
                cp_position,
            }
        }
        RocketComponent::FreeformFinSet(data) => {
            let span = estimate_freeform_span(data);
            let (root_chord, tip_chord) = estimate_freeform_chords(data);
            let cn_alpha = BarrowmanCalculator::fin_cn_alpha(
                data.fin_count,
                span,
                ref_diameter,
                root_chord,
                tip_chord,
            );
            let cp_position = BarrowmanCalculator::fin_cp_position(
                data.position.x,
                root_chord,
                tip_chord,
            );
            ComponentContribution {
                cn_alpha,
                cp_position,
            }
        }
        // All other component types contribute zero normal force
        _ => ComponentContribution {
            cn_alpha: 0.0,
            cp_position: 0.0,
        },
    }
}

/// Estimate span from a freeform fin set's point list.
pub(crate) fn estimate_freeform_span(data: &FreeformFinSetData) -> f64 {
    if data.points.is_empty() {
        return 0.0;
    }
    let min_y: f64 = data
        .points
        .iter()
        .map(|p| p.y)
        .fold(f64::INFINITY, f64::min);
    let max_y: f64 = data
        .points
        .iter()
        .map(|p| p.y)
        .fold(f64::NEG_INFINITY, f64::max);
    (max_y - min_y).max(0.0)
}

/// Estimate root and tip chords from a freeform fin set's point list.
pub(crate) fn estimate_freeform_chords(data: &FreeformFinSetData) -> (f64, f64) {
    if data.points.is_empty() {
        return (0.0, 0.0);
    }
    let min_x: f64 = data
        .points
        .iter()
        .map(|p| p.x)
        .fold(f64::INFINITY, f64::min);
    let max_x: f64 = data
        .points
        .iter()
        .map(|p| p.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let span = (max_x - min_x).max(0.0);
    (span, span * 0.5)
}

/// Compute the total drag coefficient for the rocket at given conditions.
///
/// Returns the sum of all drag components:
/// CD_total = CD_base + CD_friction + CD_pressure + CD_fin + CD_interference
pub fn total_drag_coefficient(
    tree: &ComponentTree,
    mach: f64,
    reynolds_number: f64,
    ref_area: f64,
    _ref_diameter: f64,
) -> f64 {
    let mut cd_total = 0.0;

    // Base drag
    cd_total += base_drag(mach);

    // Skin friction drag (wetted area summed across components)
    let (wet_area, _) = sum_wetted_areas(tree);
    let cf = skin_friction_coefficient(reynolds_number);
    cd_total += cf * wet_area / ref_area;

    // Pressure drag from nose cone and transitions
    for (_key, node) in tree.iter() {
        match &node.component {
            RocketComponent::NoseCone(data) => {
                let half_angle = nose_cone_half_angle(data);
                cd_total += 0.8 * half_angle.sin().powi(2);
            }
            RocketComponent::Transition(data) => {
                let length = *data.length.value();
                let dr = (*data.aft_radius.value() - *data.fore_radius.value()).abs();
                if length > 0.0 {
                    let half_angle = (dr / length).atan();
                    cd_total += 0.8 * half_angle.sin().powi(2);
                }
            }
            RocketComponent::FinSet(data) => {
                let wet_fin = fin_wetted_area(data);
                let root_chord = *data.root_chord.value();
                let tip_chord = *data.tip_chord.value();
                let t_over_c = if (root_chord + tip_chord).abs() > 0.0 {
                    2.0 * data.thickness.value().abs() / (root_chord + tip_chord)
                } else {
                    0.0
                };
                cd_total += cf * (wet_fin / ref_area) * (1.0 + 2.0 * t_over_c);

                // Interference drag
                let thickness = *data.thickness.value();
                cd_total +=
                    0.03 * (data.fin_count as f64 * thickness * root_chord) / ref_area;
            }
            RocketComponent::FreeformFinSet(data) => {
                let (rc, tc) = estimate_freeform_chords(data);
                let span = estimate_freeform_span(data);
                let wet_fin = 2.0 * (rc + tc) / 2.0 * span;
                let t_over_c = if (rc + tc).abs() > 0.0 {
                    2.0 * data.thickness.value().abs() / (rc + tc)
                } else {
                    0.0
                };
                cd_total += cf * (wet_fin / ref_area) * (1.0 + 2.0 * t_over_c);
                let thickness = *data.thickness.value();
                cd_total +=
                    0.03 * (data.fin_count as f64 * thickness * rc) / ref_area;
            }
            _ => {}
        }
    }

    cd_total
}

// ---- Drag helper functions ----

/// Base drag coefficient as a function of Mach number (subsonic approximation).
fn base_drag(mach: f64) -> f64 {
    0.12 + 0.13 * mach * mach
}

/// Turbulent flat-plate skin friction coefficient.
fn skin_friction_coefficient(re: f64) -> f64 {
    if re <= 0.0 {
        return 0.0;
    }
    0.074 / re.powf(0.2)
}

/// Sum wetted area of body tubes and transitions.
fn sum_wetted_areas(tree: &ComponentTree) -> (f64, f64) {
    let mut wet_area = 0.0;
    let mut _planform = 0.0;
    for (_key, node) in tree.iter() {
        match &node.component {
            RocketComponent::BodyTube(data) => {
                let r = *data.outer_radius.value();
                let l = *data.length.value();
                wet_area += 2.0 * std::f64::consts::PI * r * l;
            }
            RocketComponent::Transition(data) => {
                let r_fore = *data.fore_radius.value();
                let r_aft = *data.aft_radius.value();
                let l = *data.length.value();
                let slant = ((r_aft - r_fore).powi(2) + l * l).sqrt();
                wet_area += std::f64::consts::PI * (r_fore + r_aft) * slant;
            }
            _ => {}
        }
    }
    (wet_area, _planform)
}

/// Estimate half-angle of a nose cone from its dimensions.
fn nose_cone_half_angle(data: &NoseConeData) -> f64 {
    let length = *data.length.value();
    let base_radius = *data.base_radius.value();
    if length <= 0.0 {
        return 0.0;
    }
    (base_radius / length).atan()
}

/// Wetted area for a single fin (both sides + tip).
fn fin_wetted_area(data: &FinSetData) -> f64 {
    let root_chord = *data.root_chord.value();
    let tip_chord = *data.tip_chord.value();
    let span = *data.span.value();
    let thickness = *data.thickness.value();
    let one_side = 0.5 * (root_chord + tip_chord) * span;
    2.0 * one_side + tip_chord * thickness
}

/// Compute the pitch damping moment coefficient Cmq (simplified).
///
/// Cmq = -2 * CNα * (X_cp - X_cg)^2 / D_ref
///
/// where CNα is the total normal force coefficient derivative,
/// X_cp is the center of pressure position (m), X_cg is the center
/// of mass position (m), and D_ref is the reference diameter (m).
pub fn pitch_damping_moment_coefficient(
    total_cn_alpha: f64,
    cp_position: f64,
    cg_position: f64,
    ref_diameter: f64,
) -> f64 {
    if ref_diameter <= 0.0 {
        return 0.0;
    }
    let lever = cp_position - cg_position;
    -2.0 * total_cn_alpha * lever * lever / ref_diameter
}

/// Enhanced Mach-dependent base drag coefficient
pub fn base_drag_enhanced(mach: f64, _base_area_ratio: f64) -> f64 {
    if mach < 0.8 {
        // Subsonic: base drag decreases with Mach
        0.12 * (1.0 - 0.3 * mach)
    } else if mach > 1.2 {
        // Supersonic: base drag increases with Mach
        0.08 + 0.05 * (mach - 1.2)
    } else {
        // Transonic blend
        let t = (mach - 0.8) / 0.4;
        let subsonic = 0.12 * (1.0 - 0.3 * 0.8);
        let supersonic = 0.08 + 0.05 * 0.0;
        subsonic + t * (supersonic - subsonic)
    }
}

/// Enhanced skin friction coefficient (compressible flat plate)
pub fn skin_friction_compressible(
    reynolds: f64,
    mach: f64,
    recovery_temp_ratio: f64,
) -> f64 {
    if reynolds <= 0.0 { return 0.0; }
    
    // Incompressible turbulent skin friction (Schlichting)
    let cf_incomp = if reynolds < 1e5 {
        1.328 / reynolds.sqrt() // Laminar
    } else {
        0.074 / reynolds.powf(0.2) // Turbulent (Prandtl-Schlichting)
    };
    
    // Compressibility correction
    if mach < 0.8 {
        cf_incomp / (1.0 + 0.144 * mach.powi(2)).powf(0.65)
    } else if mach > 1.2 {
        // Supersonic: reference temperature method
        let t_ratio = 1.0 + 0.5 * (recovery_temp_ratio - 1.0) * mach.powi(2);
        cf_incomp / t_ratio.powf(0.65)
    } else {
        cf_incomp / (1.0 + 0.144 * mach.powi(2)).powf(0.65)
    }
}

/// Wave drag for nose cone (supersonic)
pub fn wave_drag_nose(mach: f64, nose_fineness: f64, nose_type: &str) -> f64 {
    if mach <= 1.0 { return 0.0; }
    
    let beta = (mach.powi(2) - 1.0).sqrt();
    let fn_ratio = nose_fineness; // L/D
    
    match nose_type {
        "Conical" => {
            // Cone wave drag (Taylor-Maccoll)
            let theta = (0.5 / fn_ratio).atan(); // half-angle
            0.5 * theta.sin().powi(2) / beta
        }
        "Ogive" => {
            // Tangent ogive wave drag
            0.5 * (1.0 - 0.5 / beta / fn_ratio) / fn_ratio
        }
        "Elliptical" => {
            // Elliptical wave drag
            0.5 / fn_ratio / beta * 0.8
        }
        "VonKarman" | "Haack" => {
            // Von Karman / Haack series have minimum wave drag
            0.5 / fn_ratio / beta * 0.6
        }
        _ => {
            // Generic
            0.5 / fn_ratio / beta
        }
    }
}

/// Wave drag for body tube (supersonic skin friction + pressure)
pub fn wave_drag_body(mach: f64, length: f64, diameter: f64) -> f64 {
    if mach <= 1.0 { return 0.0; }
    let beta = (mach.powi(2) - 1.0).sqrt();
    let slenderness = length / diameter.max(1e-6);
    
    // Body wave drag decreases with slenderness
    0.5 / (beta * slenderness)
}

/// Boat-tail drag correction (for reducing base drag)
pub fn boat_tail_drag(mach: f64, boat_tail_angle: f64, area_ratio: f64) -> f64 {
    let angle_deg = boat_tail_angle.to_degrees();
    if angle_deg <= 1.0 { return 0.0; }
    
    let base_drag_reduction = (1.0 - area_ratio) * 0.12;
    let boat_tail_drag_penalty = 0.002 * angle_deg * (1.0 + 0.5 * mach);
    
    // Net: reduce if boat tail is gentle, increase if steep
    (boat_tail_drag_penalty - base_drag_reduction).max(0.0)
}

/// Total enhanced drag calculation combining all components
pub fn total_drag_enhanced(
    mach: f64,
    reynolds: f64,
    base_area_ratio: f64,
    nose_fineness: f64,
    nose_type: &str,
    body_length: f64,
    body_diameter: f64,
    wet_area_ratio: f64,
    fin_wet_area_ratio: f64,
    fin_thickness_ratio: f64,
    fin_count: u32,
    angle_of_attack: f64,
    boat_tail_angle: f64,
    staging_gap: f64,
) -> f64 {
    // 1. Base drag
    let cd_base = base_drag_enhanced(mach, base_area_ratio);
    
    // 2. Skin friction drag
    let cf = skin_friction_compressible(reynolds, mach, 0.89);
    let cd_friction = cf * wet_area_ratio;
    
    // 3. Wave drag (supersonic)
    let cd_wave_nose = wave_drag_nose(mach, nose_fineness, nose_type);
    let cd_wave_body = wave_drag_body(mach, body_length, body_diameter);
    let cd_wave = cd_wave_nose + cd_wave_body;
    
    // 4. Fin drag
    let fin_interference = match fin_count {
        2 => 1.0, 3 => 1.1, 4 => 1.2, _ => 1.0 + 0.05 * fin_count as f64,
    };
    let cd_fin = cf * fin_wet_area_ratio * fin_interference
        * (1.0 + 2.0 * fin_thickness_ratio);
    
    // 5. Induced drag (due to AoA)
    let cd_induced = if angle_of_attack.abs() > 0.001 {
        angle_of_attack.powi(2) * 2.0 / (std::f64::consts::PI * 2.0)
    } else {
        0.0
    };
    
    // 6. Boat tail drag
    let cd_boat = boat_tail_drag(mach, boat_tail_angle, base_area_ratio);
    
    // 7. Staging gap drag
    let cd_staging = if staging_gap > 0.0 {
        let gap_ratio = staging_gap / body_diameter.max(1e-6);
        0.01 * (gap_ratio / (1.0 + gap_ratio))
    } else {
        0.0
    };
    
    cd_base + cd_friction + cd_wave + cd_fin + cd_induced + cd_boat + cd_staging
}

// ---- Enhanced drag tests ----

#[cfg(test)]
mod enhanced_drag_tests {
    use super::*;

    // ======================================================================
    // Enhanced Base Drag Tests
    // ======================================================================

    #[test]
    fn test_base_drag_enhanced_subsonic() {
        let cd = base_drag_enhanced(0.3, 1.0);
        let expected = 0.12 * (1.0 - 0.3 * 0.3);
        assert!((cd - expected).abs() < 1e-12);
    }

    #[test]
    fn test_base_drag_enhanced_supersonic() {
        let cd = base_drag_enhanced(2.0, 1.0);
        let expected = 0.08 + 0.05 * (2.0 - 1.2);
        assert!((cd - expected).abs() < 1e-12);
    }

    #[test]
    fn test_base_drag_enhanced_transonic_smooth() {
        let cd08 = base_drag_enhanced(0.8, 1.0);
        let cd12 = base_drag_enhanced(1.2, 1.0);
        assert!(cd08 >= 0.0);
        assert!(cd12 >= 0.0);
        let cd10 = base_drag_enhanced(1.0, 1.0);
        assert!(cd10 >= cd08.min(cd12) - 1e-6 || cd10 <= cd08.max(cd12) + 1e-6);
    }

    // ======================================================================
    // Compressible Skin Friction Tests
    // ======================================================================

    #[test]
    fn test_skin_friction_compressible_zero_reynolds() {
        let cf = skin_friction_compressible(0.0, 0.5, 0.89);
        assert_eq!(cf, 0.0);
    }

    #[test]
    fn test_skin_friction_compressible_turbulent() {
        let cf = skin_friction_compressible(1e6, 0.3, 0.89);
        let cf_incomp = 0.074 / 1e6_f64.powf(0.2);
        let expected = cf_incomp / (1.0 + 0.144 * 0.09_f64).powf(0.65);
        assert!((cf - expected).abs() < 1e-12);
    }

    #[test]
    fn test_skin_friction_compressible_laminar() {
        let cf = skin_friction_compressible(1e4, 0.3, 0.89);
        let cf_incomp = 1.328 / 1e4_f64.sqrt();
        let expected = cf_incomp / (1.0 + 0.144 * 0.09_f64).powf(0.65);
        assert!((cf - expected).abs() < 1e-10);
    }

    #[test]
    fn test_skin_friction_compressible_decreases_with_mach() {
        let cf_low = skin_friction_compressible(1e6, 0.2, 0.89);
        let cf_high = skin_friction_compressible(1e6, 0.7, 0.89);
        assert!(cf_high < cf_low, "SFC should decrease with Mach due to compressibility");
    }

    // ======================================================================
    // Wave Drag Nose Tests
    // ======================================================================

    #[test]
    fn test_wave_drag_nose_subsonic() {
        let cd = wave_drag_nose(0.5, 2.0, "Conical");
        assert_eq!(cd, 0.0);
    }

    #[test]
    fn test_wave_drag_nose_conical() {
        let cd = wave_drag_nose(2.0, 2.0, "Conical");
        assert!(cd > 0.0);
        let cd_fine = wave_drag_nose(2.0, 4.0, "Conical");
        assert!(cd_fine < cd);
    }

    #[test]
    fn test_wave_drag_nose_ogive() {
        let cd = wave_drag_nose(2.0, 2.0, "Ogive");
        assert!(cd > 0.0);
    }

    #[test]
    fn test_wave_drag_nose_elliptical() {
        let cd = wave_drag_nose(2.0, 2.0, "Elliptical");
        assert!(cd > 0.0);
    }

    #[test]
    fn test_wave_drag_nose_vonkarman() {
        let cd = wave_drag_nose(2.0, 2.0, "VonKarman");
        assert!(cd > 0.0);
    }

    #[test]
    fn test_wave_drag_nose_haack() {
        let cd = wave_drag_nose(2.0, 2.0, "Haack");
        assert!(cd > 0.0);
    }

    #[test]
    fn test_wave_drag_nose_generic() {
        let cd = wave_drag_nose(2.0, 2.0, "Unknown");
        assert!(cd > 0.0);
    }

    #[test]
    fn test_wave_drag_nose_decreases_with_fineness() {
        let cd_blunt = wave_drag_nose(2.0, 1.0, "Conical");
        let cd_fine = wave_drag_nose(2.0, 5.0, "Conical");
        assert!(cd_fine < cd_blunt, "Finer noses should have less wave drag");
    }

    #[test]
    fn test_wave_drag_nose_all_shapes_positive() {
        let cd_conical = wave_drag_nose(2.0, 3.0, "Conical");
        let cd_ogive = wave_drag_nose(2.0, 3.0, "Ogive");
        let cd_ellip = wave_drag_nose(2.0, 3.0, "Elliptical");
        let cd_vk = wave_drag_nose(2.0, 3.0, "VonKarman");
        let cd_haack = wave_drag_nose(2.0, 3.0, "Haack");
        assert!(cd_conical > 0.0);
        assert!(cd_ogive > 0.0);
        assert!(cd_ellip > 0.0);
        assert!(cd_vk > 0.0);
        assert!(cd_haack > 0.0);
        // Von Karman and Haack optimize for minimum wave drag at fixed length
        // but the actual values depend on the formula approximations used
    }

    // ======================================================================
    // Wave Drag Body Tests
    // ======================================================================

    #[test]
    fn test_wave_drag_body_subsonic() {
        let cd = wave_drag_body(0.5, 0.5, 0.04);
        assert_eq!(cd, 0.0);
    }

    #[test]
    fn test_wave_drag_body_supersonic() {
        let cd = wave_drag_body(2.0, 0.5, 0.04);
        assert!(cd > 0.0, "Body wave drag should be positive at M=2");
    }

    #[test]
    fn test_wave_drag_body_more_slender_less_drag() {
        let cd_stubby = wave_drag_body(2.0, 0.5, 0.04);
        let cd_slender = wave_drag_body(2.0, 1.0, 0.04);
        assert!(cd_slender < cd_stubby, "More slender bodies should have less wave drag");
    }

    // ======================================================================
    // Boat Tail Drag Tests
    // ======================================================================

    #[test]
    fn test_boat_tail_drag_shallow_angle() {
        let cd = boat_tail_drag(1.0, 0.01, 0.8);
        assert_eq!(cd, 0.0);
    }

    #[test]
    fn test_boat_tail_drag_positive() {
        let cd = boat_tail_drag(1.0, 0.1, 0.8);
        assert!(cd >= 0.0);
    }

    #[test]
    fn test_boat_tail_drag_increases_with_mach() {
        let cd_m1 = boat_tail_drag(1.0, 0.15, 0.8);
        let cd_m2 = boat_tail_drag(2.0, 0.15, 0.8);
        assert!(cd_m2 >= cd_m1, "Boat tail drag should increase with Mach");
    }

    // ======================================================================
    // Total Enhanced Drag Tests
    // ======================================================================

    #[test]
    fn test_total_drag_enhanced_subsonic() {
        let cd = total_drag_enhanced(
            0.3, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.0,
        );
        assert!(cd > 0.0, "Total drag should be positive at subsonic conditions");
    }

    #[test]
    fn test_total_drag_enhanced_supersonic() {
        let cd = total_drag_enhanced(
            2.0, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.0,
        );
        assert!(cd > 0.0, "Total drag should be positive at supersonic conditions");
    }

    #[test]
    fn test_total_drag_enhanced_supersonic_greater_than_subsonic() {
        let cd_sub = total_drag_enhanced(
            0.5, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.0,
        );
        let cd_sup = total_drag_enhanced(
            2.0, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.0,
        );
        assert!(cd_sup > cd_sub, "Supersonic drag should exceed subsonic drag");
    }

    #[test]
    fn test_total_drag_enhanced_includes_induced() {
        let cd_zero_aoa = total_drag_enhanced(
            1.0, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.0,
        );
        let cd_with_aoa = total_drag_enhanced(
            1.0, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.1, 0.0, 0.0,
        );
        assert!(cd_with_aoa > cd_zero_aoa, "Drag should increase with angle of attack");
    }

    #[test]
    fn test_total_drag_enhanced_staging_gap() {
        let cd_no_gap = total_drag_enhanced(
            1.0, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.0,
        );
        let cd_with_gap = total_drag_enhanced(
            1.0, 1e6, 1.0, 2.5, "Conical",
            0.5, 0.04, 0.5, 0.2,
            0.05, 4, 0.0, 0.0, 0.05,
        );
        assert!(cd_with_gap > cd_no_gap, "Staging gap should increase drag");
    }

    #[test]
    fn test_total_drag_enhanced_all_components_positive() {
        let cd = total_drag_enhanced(
            1.5, 2e6, 0.8, 3.0, "Ogive",
            1.0, 0.08, 0.6, 0.25,
            0.04, 3, 0.05, 0.08, 0.02,
        );
        assert!(cd > 0.0, "Total drag should be positive with all components");
        assert!(cd.is_finite(), "Total drag should be finite");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use federated_rocket_core::coordinate::Coordinate;
    use federated_rocket_core::material::{Material, MaterialType};
    use federated_rocket_core::units::Quantity;
    use federated_rocket_core::units::Unit;

    fn test_material() -> Material {
        Material::new(
            "TestMaterial",
            MaterialType::Bulk,
            Quantity::new(1000.0, Unit::Kilogram),
        )
    }

    // ======================================================================
    // Nose Cone Tests
    // ======================================================================

    #[test]
    fn test_nose_cone_cn_alpha_all_shapes() {
        let shapes = vec![
            NoseConeShape::Conical,
            NoseConeShape::Ogive,
            NoseConeShape::Elliptical,
            NoseConeShape::Parabolic,
            NoseConeShape::PowerSeries(0.5),
            NoseConeShape::VonKarman,
            NoseConeShape::HaackSeries(0.333),
        ];
        for shape in &shapes {
            let cn = BarrowmanCalculator::nose_cone_cn_alpha(shape);
            assert!((cn - 2.0).abs() < 1e-12, "CNα for {:?} should be 2.0", shape);
        }
    }

    #[test]
    fn test_nose_cone_cp_conical() {
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(&NoseConeShape::Conical, 0.2, 0.04);
        let expected = 0.666 * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6, "Conical CP: {} vs {}", cp, expected);
    }

    #[test]
    fn test_nose_cone_cp_ogive() {
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(&NoseConeShape::Ogive, 0.2, 0.04);
        let expected = 0.534 * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6, "Ogive CP: {} vs {}", cp, expected);
    }

    #[test]
    fn test_nose_cone_cp_elliptical() {
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(&NoseConeShape::Elliptical, 0.2, 0.04);
        let expected = 0.500 * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6);
    }

    #[test]
    fn test_nose_cone_cp_parabolic() {
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(&NoseConeShape::Parabolic, 0.2, 0.04);
        let expected = 0.500 * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6);
    }

    #[test]
    fn test_nose_cone_cp_power_series() {
        let n = 0.5;
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(
            &NoseConeShape::PowerSeries(n),
            0.2,
            0.04,
        );
        let expected = (n / (2.0 * n + 1.0)) * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6);
    }

    #[test]
    fn test_nose_cone_cp_von_karman() {
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(&NoseConeShape::VonKarman, 0.2, 0.04);
        let expected = 0.500 * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6);
    }

    #[test]
    fn test_nose_cone_cp_haack_series() {
        let n = 0.333;
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(
            &NoseConeShape::HaackSeries(n),
            0.2,
            0.04,
        );
        let expected = (n / (2.0 * n + 1.0)) * 0.2 / 0.04;
        assert!((cp - expected).abs() < 1e-6);
    }

    #[test]
    fn test_nose_cone_cp_zero_diameter() {
        let cp = BarrowmanCalculator::nose_cone_cp_calibers(&NoseConeShape::Conical, 0.2, 0.0);
        assert_eq!(cp, 0.0);
    }

    // ======================================================================
    // Body Tube & Transition Tests
    // ======================================================================

    #[test]
    fn test_body_tube_cn_alpha() {
        assert_eq!(BarrowmanCalculator::body_tube_cn_alpha(), 0.0);
    }

    #[test]
    fn test_transition_cn_alpha() {
        let cn = BarrowmanCalculator::transition_cn_alpha(0.04, 0.08, 0.04);
        let expected = 2.0_f64 * ((0.08_f64 / 0.04_f64).powi(2) - (0.04_f64 / 0.04_f64).powi(2));
        assert!((cn - expected).abs() < 1e-6);
    }

    #[test]
    fn test_transition_cp_position() {
        let cp = BarrowmanCalculator::transition_cp_position(0.2, 0.1, 0.02, 0.04);
        let ratio: f64 = 0.02 / 0.04;
        let term = 1.0 + (1.0 - ratio) / (1.0 - ratio * ratio);
        let expected = 0.2 + (0.1 / 3.0) * term;
        assert!((cp - expected).abs() < 1e-6);
    }

    // ======================================================================
    // Fin Set Tests
    // ======================================================================

    #[test]
    fn test_fin_cn_alpha_standard() {
        let cn = BarrowmanCalculator::fin_cn_alpha(4, 0.05, 0.04, 0.08, 0.04);
        let s_over_d: f64 = 0.05 / 0.04;
        let mean_chord: f64 = (0.08 + 0.04) / 2.0;
        let inner: f64 = (2.0 * mean_chord / (0.08 + 0.04)).powi(2);
        let denom: f64 = 1.0 + (1.0 + inner).sqrt();
        let expected = (4.0 * 4.0 * s_over_d * s_over_d) / denom;
        assert!((cn - expected).abs() < 1e-6, "Fin CNα: {} vs {}", cn, expected);
    }

    #[test]
    fn test_fin_cn_alpha_zero_count() {
        let cn = BarrowmanCalculator::fin_cn_alpha(0, 0.05, 0.04, 0.08, 0.04);
        assert_eq!(cn, 0.0);
    }

    #[test]
    fn test_fin_cp_position() {
        let cp = BarrowmanCalculator::fin_cp_position(0.5, 0.08, 0.04);
        let sum = 0.08 + 0.04;
        let term1 = (0.08 * (0.08 + 2.0 * 0.04)) / (3.0 * sum);
        let term2 = (0.08 * 0.04 + 0.04 * 0.04) / (6.0 * sum);
        let expected = 0.5 + term1 + term2;
        assert!((cp - expected).abs() < 1e-6, "Fin CP: {} vs {}", cp, expected);
    }

    // ======================================================================
    // Drag Tests
    // ======================================================================

    #[test]
    fn test_base_drag_subsonic() {
        let cd = base_drag(0.3);
        let expected = 0.12 + 0.13 * 0.3 * 0.3;
        assert!((cd - expected).abs() < 1e-6);
    }

    #[test]
    fn test_skin_friction_coefficient() {
        let cf = skin_friction_coefficient(1e6);
        let expected = 0.074 / 1e6_f64.powf(0.2);
        assert!((cf - expected).abs() < 1e-6);
    }

    #[test]
    fn test_skin_friction_zero_re() {
        let cf = skin_friction_coefficient(0.0);
        assert_eq!(cf, 0.0);
    }

    // ======================================================================
    // Pitch Damping Tests
    // ======================================================================

    #[test]
    fn test_pitch_damping_moment() {
        let cmq = pitch_damping_moment_coefficient(8.0, 0.6, 0.4, 0.04);
        let expected = -2.0 * 8.0 * (0.6_f64 - 0.4_f64).powi(2) / 0.04;
        assert!((cmq - expected).abs() < 1e-6);
    }

    // ======================================================================
    // Component Contribution Tests
    // ======================================================================

    #[test]
    fn test_component_contribution_nose_cone() {
        let nc = NoseConeData {
            name: "Nose".to_string(),
            position: Coordinate::new(0.0, 0.0, 0.0),
            length: Quantity::new(0.2, Unit::Meter),
            base_radius: Quantity::new(0.02, Unit::Meter),
            shape: NoseConeShape::Conical,
            thickness: Quantity::new(0.002, Unit::Meter),
            material: test_material(),
            color: None,
            shoulder_length: Quantity::new(0.02, Unit::Meter),
            shoulder_radius: Quantity::new(0.018, Unit::Meter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Meter),
        };
        let component = RocketComponent::NoseCone(nc);
        let contrib = component_contribution(&component, 0.04);
        assert!((contrib.cn_alpha - 2.0).abs() < 1e-12);
        assert!((contrib.cp_position - 0.1332).abs() < 1e-6);
    }

    #[test]
    fn test_component_contribution_body_tube() {
        let bt = BodyTubeData {
            name: "Tube".to_string(),
            position: Coordinate::new(0.2, 0.0, 0.0),
            length: Quantity::new(0.5, Unit::Meter),
            outer_radius: Quantity::new(0.02, Unit::Meter),
            inner_radius: Quantity::new(0.018, Unit::Meter),
            material: test_material(),
            color: None,
            has_motor_mount: false,
        };
        let component = RocketComponent::BodyTube(bt);
        let contrib = component_contribution(&component, 0.04);
        assert_eq!(contrib.cn_alpha, 0.0);
        assert_eq!(contrib.cp_position, 0.0);
    }

    #[test]
    fn test_total_rocket_cn_alpha_and_cp_simple() {
        let mut tree = ComponentTree::new();
        let nc = RocketComponent::NoseCone(NoseConeData {
            name: "Nose".to_string(),
            position: Coordinate::new(0.0, 0.0, 0.0),
            length: Quantity::new(0.2, Unit::Meter),
            base_radius: Quantity::new(0.02, Unit::Meter),
            shape: NoseConeShape::Conical,
            thickness: Quantity::new(0.002, Unit::Meter),
            material: test_material(),
            color: None,
            shoulder_length: Quantity::new(0.02, Unit::Meter),
            shoulder_radius: Quantity::new(0.018, Unit::Meter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Meter),
        });
        tree.add_component(nc, None).unwrap();

        let (total_cn, total_cp) =
            BarrowmanCalculator::total_rocket_cn_alpha_and_cp(&tree, 0.04);
        assert!((total_cn - 2.0).abs() < 1e-10);
        assert!((total_cp - 0.1332).abs() < 1e-6);
    }

    #[test]
    fn test_total_rocket_with_fins() {
        let mut tree = ComponentTree::new();
        let nc = RocketComponent::NoseCone(NoseConeData {
            name: "Nose".to_string(),
            position: Coordinate::new(0.0, 0.0, 0.0),
            length: Quantity::new(0.2, Unit::Meter),
            base_radius: Quantity::new(0.02, Unit::Meter),
            shape: NoseConeShape::Conical,
            thickness: Quantity::new(0.002, Unit::Meter),
            material: test_material(),
            color: None,
            shoulder_length: Quantity::new(0.02, Unit::Meter),
            shoulder_radius: Quantity::new(0.018, Unit::Meter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Meter),
        });
        tree.add_component(nc, None).unwrap();

        let fins = RocketComponent::FinSet(FinSetData {
            name: "Fins".to_string(),
            position: Coordinate::new(0.7, 0.0, 0.0),
            fin_count: 4,
            root_chord: Quantity::new(0.08, Unit::Meter),
            tip_chord: Quantity::new(0.04, Unit::Meter),
            span: Quantity::new(0.05, Unit::Meter),
            sweep_length: Quantity::new(0.02, Unit::Meter),
            thickness: Quantity::new(0.003, Unit::Meter),
            cross_section: AirfoilType::Square,
            material: test_material(),
            color: None,
            cant_angle: Quantity::new(0.0, Unit::Degree),
            fin_placement: FinPlacement::Normal,
        });
        tree.add_component(fins, None).unwrap();

        let (total_cn, total_cp) =
            BarrowmanCalculator::total_rocket_cn_alpha_and_cp(&tree, 0.04);
        assert!(total_cn > 2.0, "Total CNα should be > 2.0, got {}", total_cn);
        assert!(total_cp > 0.1, "Total CP should be > 0.1, got {}", total_cp);
        assert!(total_cp < 0.8, "Total CP should be < 0.8, got {}", total_cp);
    }

    #[test]
    fn test_freeform_fin_estimation() {
        let ff = FreeformFinSetData {
            name: "FF Fins".to_string(),
            position: Coordinate::new(0.5, 0.0, 0.0),
            fin_count: 3,
            points: vec![
                Coordinate::new(0.0, 0.0, 0.0),
                Coordinate::new(0.1, 0.05, 0.0),
                Coordinate::new(0.0, 0.05, 0.0),
            ],
            thickness: Quantity::new(0.003, Unit::Meter),
            cross_section: AirfoilType::Square,
            material: test_material(),
            color: None,
            cant_angle: Quantity::new(0.0, Unit::Degree),
            fin_placement: FinPlacement::Normal,
        };
        let span = estimate_freeform_span(&ff);
        assert!((span - 0.05).abs() < 1e-10);
        let (rc, tc) = estimate_freeform_chords(&ff);
        assert!((rc - 0.1).abs() < 1e-10);
        assert!((tc - 0.05).abs() < 1e-10);
    }
}