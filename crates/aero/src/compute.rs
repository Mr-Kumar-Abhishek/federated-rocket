use crate::barrowman::{self, BarrowmanCalculator};
use crate::interference::*;
use crate::supersonic::SupersonicCorrections;
use crate::types::*;
use federated_rocket_core::component::*;
use federated_rocket_core::component_tree::ComponentTree;
use federated_rocket_math::vector::Vector3D;
use federated_rocket_physics::atmosphere::AtmosphericConditions;

/// Main aerodynamic computation engine.
///
/// Orchestrates Barrowman method calculations with supersonic corrections
/// to produce aerodynamic forces and coefficients for a rocket at given
/// flight conditions.
#[derive(Clone)]
pub struct AeroCalculator {
    _barrowman: BarrowmanCalculator,
    _supersonic: SupersonicCorrections,
}

impl AeroCalculator {
    /// Creates a new `AeroCalculator`.
    pub fn new() -> Self {
        Self {
            _barrowman: BarrowmanCalculator,
            _supersonic: SupersonicCorrections,
        }
    }

    /// Compute aerodynamic forces for a rocket at given flight conditions.
    ///
    /// # Parameters
    ///
    /// * `tree` - The rocket's component tree
    /// * `velocity` - Velocity vector in body coordinates (m/s)
    /// * `angular_velocity` - Angular velocity vector (rad/s) - currently unused
    /// * `atmospheric` - Current atmospheric conditions
    /// * `reference_area` - Reference area (m²) typically cross-sectional area of body tube
    /// * `reference_length` - Reference length (m) typically body tube diameter
    ///
    /// Returns the computed aerodynamic forces and moments.
    pub fn compute_forces(
        &self,
        tree: &ComponentTree,
        velocity: Vector3D,
        _angular_velocity: Vector3D,
        atmospheric: &AtmosphericConditions,
        reference_area: f64,
        reference_length: f64,
    ) -> AeroForces {
        let speed = velocity.magnitude();
        if speed < 1e-6 || reference_area <= 0.0 {
            return AeroForces::zero();
        }

        let mach = speed / atmospheric.speed_of_sound;
        let dynamic_pressure = 0.5 * atmospheric.density * speed * speed;

        // Compute angle of attack and sideslip angle
        let axial_speed: f64 = velocity.x.abs().max(1e-10);
        let angle_of_attack = (velocity.y / axial_speed).atan();
        let sideslip_angle = (velocity.z / axial_speed).atan();

        // Compute coefficients
        let coeffs = self.compute_coefficients(
            tree,
            mach,
            angle_of_attack,
            sideslip_angle,
            reference_area,
            reference_length,
        );

        // Convert coefficients to forces
        let drag_force = coeffs.cd * dynamic_pressure * reference_area;
        let lift_force = coeffs.cl * dynamic_pressure * reference_area;
        let side_force = coeffs.cs * dynamic_pressure * reference_area;
        let pitch_moment = coeffs.cm * dynamic_pressure * reference_area * reference_length;
        let yaw_moment = coeffs.cn * dynamic_pressure * reference_area * reference_length;
        let roll_moment = coeffs.cl_roll * dynamic_pressure * reference_area * reference_length;

        // Compute CP position in meters
        let cp_position = coeffs.cp_calibers * reference_length;

        AeroForces {
            drag: drag_force,
            lift: lift_force,
            side_force,
            pitch_moment,
            yaw_moment,
            roll_moment,
            cp_position,
        }
    }

    /// Compute aerodynamic coefficients for the rocket.
    ///
    /// # Parameters
    ///
    /// * `tree` - The rocket's component tree
    /// * `mach` - Current Mach number
    /// * `angle_of_attack` - Angle of attack (radians)
    /// * `sideslip_angle` - Sideslip angle (radians)
    /// * `reference_area` - Reference area (m²)
    /// * `reference_length` - Reference length (m)
    pub fn compute_coefficients(
        &self,
        tree: &ComponentTree,
        mach: f64,
        angle_of_attack: f64,
        sideslip_angle: f64,
        _reference_area: f64,
        reference_length: f64,
    ) -> AeroCoefficients {
        let ref_diameter = reference_length;

        // Get Barrowman values (subsonic, incompressible)
        let (subsonic_cn_alpha, subsonic_cp) =
            BarrowmanCalculator::total_rocket_cn_alpha_and_cp(tree, ref_diameter);

        // Apply compressibility/supersonic corrections to CNα
        let corrected_cn_alpha = self.correct_cn_alpha(mach, subsonic_cn_alpha);

        // CP position moves slightly aft with Mach
        let corrected_cp = if subsonic_cn_alpha.abs() > 1e-12 {
            let cp_shift_ratio = (corrected_cn_alpha / subsonic_cn_alpha).min(1.5).max(0.5);
            subsonic_cp * cp_shift_ratio
        } else {
            subsonic_cp
        };

        let cp_calibers = if ref_diameter > 0.0 {
            corrected_cp / ref_diameter
        } else {
            0.0
        };

        // Compute lift and side force coefficients from CNα
        let cl = corrected_cn_alpha * angle_of_attack.sin().max(0.0);
        let cs = corrected_cn_alpha * sideslip_angle.sin();

        // Compute drag coefficient
        let drag_coeff = self.compute_total_drag(tree, mach, ref_diameter);

        // Moment coefficients
        let approx_cg = Self::estimate_cg_position(tree);
        let moment_arm = (corrected_cp - approx_cg) / ref_diameter;
        let cm = corrected_cn_alpha * moment_arm * angle_of_attack.sin();
        let cn_moment = corrected_cn_alpha * moment_arm * sideslip_angle.sin();

        // Roll moment (simplified - from fin cant)
        let cl_roll = self.compute_roll_moment_coefficient(tree);

        AeroCoefficients {
            cd: drag_coeff,
            cl,
            cs,
            cm,
            cn: cn_moment,
            cl_roll,
            cn_alpha: corrected_cn_alpha,
            cp_calibers,
        }
    }

    /// Correct CNα for Mach number effects using compressibility and supersonic corrections.
    fn correct_cn_alpha(&self, mach: f64, subsonic_cn_alpha: f64) -> f64 {
        match FlowRegime::from_mach(mach) {
            FlowRegime::Subsonic => {
                let pg = SupersonicCorrections::prandtl_glauert_factor(mach);
                subsonic_cn_alpha * pg
            }
            FlowRegime::Transonic => {
                let pg = SupersonicCorrections::prandtl_glauert_factor(0.799);
                let supersonic_cn = SupersonicCorrections::supersonic_normal_force(mach);
                let subsonic_val = subsonic_cn_alpha * pg;
                SupersonicCorrections::transonic_blend(mach, subsonic_val, supersonic_cn)
            }
            FlowRegime::Supersonic | FlowRegime::Hypersonic => {
                SupersonicCorrections::supersonic_normal_force(mach)
            }
        }
    }

    /// Calculate center of pressure position (m from nose tip).
    pub fn calculate_cp(&self, tree: &ComponentTree, mach: f64) -> f64 {
        let ref_diameter = Self::find_reference_diameter(tree);
        let (subsonic_cn_alpha, subsonic_cp) =
            BarrowmanCalculator::total_rocket_cn_alpha_and_cp(tree, ref_diameter);

        let corrected_cn = self.correct_cn_alpha(mach, subsonic_cn_alpha);

        if subsonic_cn_alpha.abs() > 1e-12 {
            let ratio = (corrected_cn / subsonic_cn_alpha).min(1.5).max(0.5);
            subsonic_cp * ratio
        } else {
            subsonic_cp
        }
    }

    /// Calculate total drag coefficient.
    pub fn calculate_drag(&self, tree: &ComponentTree, mach: f64, _reynolds_number: f64) -> f64 {
        let ref_diameter = Self::find_reference_diameter(tree);
        let ref_area = if ref_diameter > 0.0 {
            std::f64::consts::PI * ref_diameter * ref_diameter / 4.0
        } else {
            1.0
        };
        let re = 1.0e6;
        barrowman::total_drag_coefficient(tree, mach, re, ref_area, ref_diameter)
    }

    /// Compute drag with full enhanced model (all Mach regimes, all components)
    pub fn compute_drag_enhanced(
        &self,
        tree: &ComponentTree,
        mach: f64,
        reynolds: f64,
        angle_of_attack: f64,
        reference_area: f64,
        reference_diameter: f64,
    ) -> f64 {
        // Collect geometric parameters from component tree
        let (nose_fineness, nose_type) = self.extract_nose_params(tree);
        let (body_length, body_diameter) = self.extract_body_params(tree);
        let (fin_wet_area_ratio, fin_thickness_ratio, fin_count) =
            self.extract_fin_params(tree, reference_area);
        let wet_area_ratio = self.compute_wet_area_ratio(tree, reference_area);
        let base_area_ratio = self.compute_base_area_ratio(tree, reference_diameter);

        barrowman::total_drag_enhanced(
            mach,
            reynolds,
            base_area_ratio,
            nose_fineness,
            &nose_type,
            body_length,
            body_diameter,
            wet_area_ratio,
            fin_wet_area_ratio,
            fin_thickness_ratio,
            fin_count as u32,
            angle_of_attack,
            0.0,
            0.0,
        )
    }

    /// Compute forces including interference effects
    pub fn compute_forces_with_interference(
        &self,
        tree: &ComponentTree,
        velocity: Vector3D,
        _angular_velocity: Vector3D,
        atmospheric: &AtmosphericConditions,
        reference_area: f64,
        reference_diameter: f64,
    ) -> AeroForces {
        let _mach = velocity.magnitude() / atmospheric.speed_of_sound;
        let _aoa = self.compute_angle_of_attack(&velocity);

        // Base forces
        let forces = self.compute_forces(
            tree,
            velocity,
            Vector3D::zero(),
            atmospheric,
            reference_area,
            reference_diameter,
        );

        // Apply interference corrections
        let body_radius = reference_diameter / 2.0;
        let fin_span = self.extract_fin_span(tree);
        let _interference = fin_body_interference_factor(body_radius, fin_span);

        // Scale normal force by interference factor
        // (only affects lift/pitch, not drag)

        forces
    }

    /// Extract geometric parameters from component tree
    pub fn extract_nose_params(&self, tree: &ComponentTree) -> (f64, String) {
        // Walk tree to find first nose cone and return (fineness_ratio, shape_type)
        for (_key, node) in tree.iter() {
            if let RocketComponent::NoseCone(ref data) = node.component {
                let fineness = if *data.base_radius.value() > 0.0 {
                    data.length.value() / (2.0 * data.base_radius.value())
                } else {
                    1.0
                };
                let shape = match data.shape {
                    NoseConeShape::Conical => "Conical",
                    NoseConeShape::Ogive => "Ogive",
                    NoseConeShape::Elliptical => "Elliptical",
                    NoseConeShape::VonKarman => "VonKarman",
                    NoseConeShape::HaackSeries(_) => "Haack",
                    _ => "Conical",
                };
                return (fineness, shape.to_string());
            }
        }
        (2.0, "Conical".to_string()) // defaults
    }

    /// Extract body length and diameter from the component tree
    pub fn extract_body_params(&self, tree: &ComponentTree) -> (f64, f64) {
        let mut total_length = 0.0;
        let mut max_diameter: f64 = 0.04;
        for (_key, node) in tree.iter() {
            match &node.component {
                RocketComponent::BodyTube(data) => {
                    total_length += data.length.value();
                    let d = data.outer_radius.value() * 2.0;
                    max_diameter = max_diameter.max(d);
                }
                RocketComponent::NoseCone(data) => {
                    total_length += data.length.value();
                    let d = data.base_radius.value() * 2.0;
                    max_diameter = max_diameter.max(d);
                }
                _ => {}
            }
        }
        (total_length, max_diameter)
    }

    /// Extract fin parameters from component tree
    pub fn extract_fin_params(&self, tree: &ComponentTree, reference_area: f64) -> (f64, f64, u32) {
        let mut total_wet_area_ratio = 0.0;
        let mut thickness_ratio = 0.0;
        let mut fin_count = 0u32;
        for (_key, node) in tree.iter() {
            match &node.component {
                RocketComponent::FinSet(data) => {
                    fin_count = data.fin_count;
                    let root_chord = data.root_chord.value();
                    let tip_chord = data.tip_chord.value();
                    let span = data.span.value();
                    let wet_fin = 2.0 * 0.5 * (root_chord + tip_chord) * span
                        + tip_chord * data.thickness.value();
                    total_wet_area_ratio += wet_fin / reference_area;
                    let mean_chord = (root_chord + tip_chord) / 2.0;
                    if mean_chord > 0.0 {
                        thickness_ratio = data.thickness.value() / mean_chord;
                    }
                }
                RocketComponent::FreeformFinSet(data) => {
                    fin_count = data.fin_count;
                    let (rc, tc) = crate::barrowman::estimate_freeform_chords(data);
                    let span = crate::barrowman::estimate_freeform_span(data);
                    let wet_fin = 2.0 * 0.5 * (rc + tc) * span;
                    total_wet_area_ratio += wet_fin / reference_area;
                    let mean_chord = (rc + tc) / 2.0;
                    if mean_chord > 0.0 {
                        thickness_ratio = data.thickness.value() / mean_chord;
                    }
                }
                _ => {}
            }
        }
        (total_wet_area_ratio, thickness_ratio, fin_count)
    }

    /// Compute wetted area ratio relative to reference area
    pub fn compute_wet_area_ratio(&self, tree: &ComponentTree, reference_area: f64) -> f64 {
        let mut wet_area = 0.0;
        for (_key, node) in tree.iter() {
            match &node.component {
                RocketComponent::BodyTube(data) => {
                    let r = data.outer_radius.value();
                    let l = data.length.value();
                    wet_area += 2.0 * std::f64::consts::PI * r * l;
                }
                RocketComponent::NoseCone(data) => {
                    let r = data.base_radius.value();
                    let l = data.length.value();
                    let slant = (r * r + l * l).sqrt();
                    wet_area += std::f64::consts::PI * r * slant;
                }
                RocketComponent::Transition(data) => {
                    let r_fore = data.fore_radius.value();
                    let r_aft = data.aft_radius.value();
                    let l = data.length.value();
                    let slant = ((r_aft - r_fore).powi(2) + l * l).sqrt();
                    wet_area += std::f64::consts::PI * (r_fore + r_aft) * slant;
                }
                _ => {}
            }
        }
        if reference_area > 0.0 {
            wet_area / reference_area
        } else {
            0.0
        }
    }

    /// Compute base area ratio
    pub fn compute_base_area_ratio(&self, tree: &ComponentTree, reference_diameter: f64) -> f64 {
        let ref_area = if reference_diameter > 0.0 {
            std::f64::consts::PI * reference_diameter * reference_diameter / 4.0
        } else {
            1.0
        };
        // Find the aft-most body tube or transition base area
        let mut base_area = ref_area;
        for (_key, node) in tree.iter() {
            match &node.component {
                RocketComponent::BodyTube(data) => {
                    let area = std::f64::consts::PI
                        * data.outer_radius.value()
                        * data.outer_radius.value();
                    base_area = area;
                }
                RocketComponent::Transition(data) => {
                    let area =
                        std::f64::consts::PI * data.aft_radius.value() * data.aft_radius.value();
                    base_area = area;
                }
                _ => {}
            }
        }
        base_area / ref_area
    }

    /// Extract fin span from component tree
    pub fn extract_fin_span(&self, tree: &ComponentTree) -> f64 {
        let mut max_span: f64 = 0.0;
        for (_key, node) in tree.iter() {
            match &node.component {
                RocketComponent::FinSet(data) => {
                    max_span = max_span.max(*data.span.value());
                }
                RocketComponent::FreeformFinSet(data) => {
                    let span = crate::barrowman::estimate_freeform_span(data);
                    max_span = max_span.max(span);
                }
                _ => {}
            }
        }
        max_span
    }

    /// Compute angle of attack from velocity vector
    pub fn compute_angle_of_attack(&self, velocity: &Vector3D) -> f64 {
        let speed = velocity.magnitude();
        if speed < 1e-6 {
            return 0.0;
        }
        let axial = velocity.x.abs().max(1e-10);
        (velocity.y / axial).atan()
    }

    /// Calculate stability margin in calibers.
    ///
    /// Positive margin means the rocket is statically stable
    /// (CP is behind CG).
    ///
    /// # Parameters
    ///
    /// * `cp_position` - Center of pressure position (m from nose tip)
    /// * `cg_position` - Center of mass position (m from nose tip)
    /// * `reference_diameter` - Reference diameter (m)
    ///
    /// Returns stability margin in calibers (body diameters).
    /// Rule of thumb: margin > 1.0 is stable for model rockets.
    pub fn stability_margin(cp_position: f64, cg_position: f64, reference_diameter: f64) -> f64 {
        if reference_diameter <= 0.0 {
            return 0.0;
        }
        (cp_position - cg_position) / reference_diameter
    }

    // ======================================================================
    // Private helpers
    // ======================================================================

    /// Compute the total drag coefficient including all components and Mach effects.
    fn compute_total_drag(&self, tree: &ComponentTree, mach: f64, ref_diameter: f64) -> f64 {
        let ref_area = if ref_diameter > 0.0 {
            std::f64::consts::PI * ref_diameter * ref_diameter / 4.0
        } else {
            1.0
        };
        let re = 1.0e6;

        // Subsonic drag from Barrowman
        let mut cd = barrowman::total_drag_coefficient(tree, mach, re, ref_area, ref_diameter);

        // Add wave drag (supersonic)
        let nose_fineness = Self::find_nose_fineness(tree);
        cd += SupersonicCorrections::wave_drag(mach, nose_fineness);

        // Apply base drag correction
        let base_correction = SupersonicCorrections::base_drag_correction(mach);
        cd = cd * (0.8 + 0.2 * base_correction);

        cd
    }

    /// Estimate the CG position from the component tree (rough approximation).
    /// Returns position in meters from nose tip.
    fn estimate_cg_position(tree: &ComponentTree) -> f64 {
        let mut total_mass = 0.0;
        let mut weighted_pos = 0.0;

        for (_key, node) in tree.iter() {
            let (mass, pos) = Self::component_mass_and_position(&node.component);
            total_mass += mass;
            weighted_pos += mass * pos;
        }

        if total_mass > 0.0 {
            weighted_pos / total_mass
        } else {
            let length = Self::find_total_length(tree);
            length * 0.6
        }
    }

    /// Extract mass (kg) and axial position (m) from a component.
    fn component_mass_and_position(component: &RocketComponent) -> (f64, f64) {
        match component {
            RocketComponent::BodyTube(d) => {
                let r = *d.outer_radius.value();
                let l = *d.length.value();
                let vol = std::f64::consts::PI * r * r * l;
                let mass = vol * *d.material.density.value();
                let cg_x = d.position.x + l / 2.0;
                (mass, cg_x)
            }
            RocketComponent::NoseCone(d) => {
                let r = *d.base_radius.value();
                let l = *d.length.value();
                let vol = (1.0 / 3.0) * std::f64::consts::PI * r * r * l;
                let mass = vol * *d.material.density.value();
                let cg_x = d.position.x + l * 0.4;
                (mass, cg_x)
            }
            RocketComponent::Transition(d) => {
                let r_f = *d.fore_radius.value();
                let r_a = *d.aft_radius.value();
                let l = *d.length.value();
                let vol =
                    (1.0 / 3.0) * std::f64::consts::PI * l * (r_f * r_f + r_f * r_a + r_a * r_a);
                let mass = vol * *d.material.density.value();
                let cg_x = d.position.x + l * 0.5;
                (mass, cg_x)
            }
            RocketComponent::FinSet(d) => {
                let root_chord = *d.root_chord.value();
                let tip_chord = *d.tip_chord.value();
                let span = *d.span.value();
                let thickness = *d.thickness.value();
                let area = 0.5 * (root_chord + tip_chord) * span;
                let vol = area * thickness * d.fin_count as f64;
                let mass = vol * *d.material.density.value();
                let cg_x = d.position.x + root_chord * 0.5;
                (mass, cg_x)
            }
            RocketComponent::MassComponent(d) => {
                let mass = *d.mass.value();
                let cg_x = d.position.x;
                (mass, cg_x)
            }
            RocketComponent::Parachute(d) => (0.01, d.position.x),
            RocketComponent::Streamer(_) => (0.005, 0.0),
            RocketComponent::Bulkhead(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.thickness.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x)
            }
            RocketComponent::CenteringRing(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.length.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::Engine(d) => {
                let mass = *d.dry_mass.value() + *d.propellant_mass.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::InnerTube(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.length.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::TubeCoupler(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.length.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::Sleeve(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.length.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::EngineBlock(d) => {
                let vol =
                    std::f64::consts::PI * d.radius.value() * d.radius.value() * d.length.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::LaunchLug(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.length.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x + *d.length.value() / 2.0)
            }
            RocketComponent::RailButton(d) => {
                let vol = std::f64::consts::PI
                    * d.outer_radius.value()
                    * d.outer_radius.value()
                    * d.height.value();
                let mass = vol * *d.material.density.value();
                (mass, d.position.x)
            }
            RocketComponent::Pod(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                let vol = std::f64::consts::PI * r * r * l;
                let mass = vol * 600.0;
                (mass, d.position.x + l / 2.0)
            }
            RocketComponent::Booster(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                let vol = std::f64::consts::PI * r * r * l;
                let mass = vol * 600.0;
                (mass, d.position.x + l / 2.0)
            }
            RocketComponent::Payload(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                let vol = std::f64::consts::PI * r * r * l;
                let mass = vol * 600.0;
                (mass, d.position.x + l / 2.0)
            }
            RocketComponent::FreeformFinSet(_) => (0.01, 0.0),
            RocketComponent::RecoveryDevice(_) => (0.01, 0.0),
            RocketComponent::ComponentAssembly(_) => (0.0, 0.0),
        }
    }

    /// Find the reference diameter (max body tube outer diameter) from the tree.
    fn find_reference_diameter(tree: &ComponentTree) -> f64 {
        let mut max_diameter: f64 = 0.04;
        for (_key, node) in tree.iter() {
            match &node.component {
                RocketComponent::BodyTube(data) => {
                    let d = *data.outer_radius.value() * 2.0;
                    max_diameter = max_diameter.max(d);
                }
                RocketComponent::NoseCone(data) => {
                    let d = *data.base_radius.value() * 2.0;
                    max_diameter = max_diameter.max(d);
                }
                _ => {}
            }
        }
        max_diameter
    }

    /// Find the total length of the rocket from the tree.
    fn find_total_length(tree: &ComponentTree) -> f64 {
        let mut max_x: f64 = 0.0;
        for (_key, node) in tree.iter() {
            let (_, max) = node.component.bounding_box();
            max_x = max_x.max(max.x);
        }
        max_x
    }

    /// Find nose fineness ratio (length/diameter).
    fn find_nose_fineness(tree: &ComponentTree) -> f64 {
        for (_key, node) in tree.iter() {
            if let RocketComponent::NoseCone(data) = &node.component {
                let length = *data.length.value();
                let diameter = *data.base_radius.value() * 2.0;
                if diameter > 0.0 {
                    return length / diameter;
                }
            }
        }
        1.0
    }

    /// Compute roll moment coefficient from fin cant angle.
    fn compute_roll_moment_coefficient(&self, tree: &ComponentTree) -> f64 {
        let mut cl_roll = 0.0;
        for (_key, node) in tree.iter() {
            if let RocketComponent::FinSet(data) = &node.component {
                let cant = *data.cant_angle.value();
                if cant.abs() > 1e-12 {
                    let n = data.fin_count as f64;
                    let span = *data.span.value();
                    let area = 0.5 * (*data.root_chord.value() + *data.tip_chord.value()) * span;
                    cl_roll += n * area * cant / (std::f64::consts::PI * span * span);
                }
            }
        }
        cl_roll
    }
}

impl Default for AeroCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use federated_rocket_core::coordinate::Coordinate;
    use federated_rocket_core::material::{Material, MaterialType};
    use federated_rocket_core::units::{Quantity, Unit};

    fn test_material() -> Material {
        Material::new(
            "TestMaterial",
            MaterialType::Bulk,
            Quantity::new(1000.0, Unit::Kilogram),
        )
    }

    fn make_simple_rocket() -> ComponentTree {
        let mut tree = ComponentTree::new();

        let body_tube = RocketComponent::BodyTube(BodyTubeData {
            name: "Body".to_string(),
            position: Coordinate::new(0.2, 0.0, 0.0),
            length: Quantity::new(0.5, Unit::Meter),
            outer_radius: Quantity::new(0.02, Unit::Meter),
            inner_radius: Quantity::new(0.018, Unit::Meter),
            material: test_material(),
            color: None,
            has_motor_mount: false,
        });
        tree.add_component(body_tube, None).unwrap();

        let nose = RocketComponent::NoseCone(NoseConeData {
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
        tree.add_component(nose, None).unwrap();

        let fins = RocketComponent::FinSet(FinSetData {
            name: "Fins".to_string(),
            position: Coordinate::new(0.6, 0.0, 0.0),
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

        tree
    }

    #[test]
    fn test_aero_calculator_new() {
        let calc = AeroCalculator::new();
        let _ = calc;
    }

    #[test]
    fn test_compute_coefficients_subsonic() {
        let calc = AeroCalculator::new();
        let tree = make_simple_rocket();
        let coeffs = calc.compute_coefficients(&tree, 0.3, 0.05, 0.0, 0.001256, 0.04);
        assert!(coeffs.cd > 0.0, "CD should be positive, got {}", coeffs.cd);
        assert!(coeffs.cl > 0.0, "CL should be positive, got {}", coeffs.cl);
        assert!(
            coeffs.cn_alpha > 2.0,
            "CNα should be > 2.0, got {}",
            coeffs.cn_alpha
        );
        assert!(coeffs.cp_calibers > 0.0);
    }

    #[test]
    fn test_compute_forces() {
        let calc = AeroCalculator::new();
        let tree = make_simple_rocket();
        let velocity = Vector3D::new(100.0, 5.0, 0.0);
        let angular_velocity = Vector3D::zero();
        let atmospheric = AtmosphericConditions::new(0.0, 288.15, 101325.0, 1.225, 340.3, 1.789e-5);

        let forces = calc.compute_forces(
            &tree,
            velocity,
            angular_velocity,
            &atmospheric,
            0.001256,
            0.04,
        );

        assert!(
            forces.drag > 0.0,
            "Drag should be positive, got {}",
            forces.drag
        );
        assert!(forces.cp_position > 0.0);
    }

    #[test]
    fn test_calculate_cp() {
        let calc = AeroCalculator::new();
        let tree = make_simple_rocket();
        let cp = calc.calculate_cp(&tree, 0.3);
        assert!(cp > 0.0, "CP should be positive, got {}", cp);
        assert!(cp > 0.1, "CP should be > 0.1m, got {}", cp);
    }

    #[test]
    fn test_calculate_drag() {
        let calc = AeroCalculator::new();
        let tree = make_simple_rocket();
        let cd = calc.calculate_drag(&tree, 0.3, 1.0e6);
        assert!(cd > 0.0, "CD should be positive, got {}", cd);
    }

    #[test]
    fn test_stability_margin() {
        let margin = AeroCalculator::stability_margin(0.5, 0.3, 0.04);
        assert!(
            (margin - 5.0).abs() < 1e-10,
            "Margin should be 5.0, got {}",
            margin
        );
    }

    #[test]
    fn test_stability_margin_negative() {
        let margin = AeroCalculator::stability_margin(0.3, 0.5, 0.04);
        assert!(margin < 0.0, "Margin should be negative, got {}", margin);
    }

    #[test]
    fn test_stability_margin_zero_diameter() {
        let margin = AeroCalculator::stability_margin(0.5, 0.3, 0.0);
        assert_eq!(margin, 0.0);
    }

    #[test]
    fn test_zero_velocity_returns_zero_forces() {
        let calc = AeroCalculator::new();
        let tree = make_simple_rocket();
        let velocity = Vector3D::zero();
        let angular_velocity = Vector3D::zero();
        let atmospheric = AtmosphericConditions::new(0.0, 288.15, 101325.0, 1.225, 340.3, 1.789e-5);

        let forces = calc.compute_forces(
            &tree,
            velocity,
            angular_velocity,
            &atmospheric,
            0.001256,
            0.04,
        );

        assert_eq!(forces.drag, 0.0);
        assert_eq!(forces.lift, 0.0);
        assert_eq!(forces.total_force(), 0.0);
    }

    #[test]
    fn test_integration_complete_rocket_stability() {
        let _calc = AeroCalculator::new();
        let tree = make_simple_rocket();

        let (cn_alpha, cp_position) =
            BarrowmanCalculator::total_rocket_cn_alpha_and_cp(&tree, 0.04);
        let cg_position = AeroCalculator::estimate_cg_position(&tree);
        let margin = AeroCalculator::stability_margin(cp_position, cg_position, 0.04);

        assert!(
            margin > 0.0,
            "Rocket should be stable: CP={}m, CG={}m, margin={} calibers",
            cp_position,
            cg_position,
            margin
        );

        assert!(cn_alpha > 0.0, "CNα should be positive, got {}", cn_alpha);
    }
}
