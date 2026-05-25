use serde::{Deserialize, Serialize};

/// Aerodynamic forces and moments acting on the rocket.
///
/// All forces are in Newtons (N) and moments in Newton-meters (N·m),
/// expressed in the body-fixed coordinate frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AeroForces {
    /// Drag force (N) in body coordinates (positive = aft)
    pub drag: f64,
    /// Lift force (N) in body coordinates (positive = up)
    pub lift: f64,
    /// Side force (N) in body coordinates (positive = right)
    pub side_force: f64,
    /// Pitching moment (N·m) about center of mass
    pub pitch_moment: f64,
    /// Yawing moment (N·m) about center of mass
    pub yaw_moment: f64,
    /// Rolling moment (N·m) about center of mass
    pub roll_moment: f64,
    /// Center of pressure position (m) from nose tip
    pub cp_position: f64,
}

impl AeroForces {
    /// Returns a zeroed-out `AeroForces` (all fields set to 0.0).
    pub fn zero() -> Self {
        Self {
            drag: 0.0,
            lift: 0.0,
            side_force: 0.0,
            pitch_moment: 0.0,
            yaw_moment: 0.0,
            roll_moment: 0.0,
            cp_position: 0.0,
        }
    }

    /// Returns the magnitude of the total aerodynamic force vector.
    pub fn total_force(&self) -> f64 {
        (self.drag * self.drag
            + self.lift * self.lift
            + self.side_force * self.side_force)
            .sqrt()
    }
}

/// Non-dimensional aerodynamic coefficients.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AeroCoefficients {
    /// Drag coefficient (CD)
    pub cd: f64,
    /// Lift coefficient (CL)
    pub cl: f64,
    /// Side force coefficient (CS)
    pub cs: f64,
    /// Pitch moment coefficient (Cm)
    pub cm: f64,
    /// Yaw moment coefficient (Cn)
    pub cn: f64,
    /// Roll moment coefficient (Cl)
    pub cl_roll: f64,
    /// Normal force coefficient derivative (CNα) per radian
    pub cn_alpha: f64,
    /// Center of pressure position (calibers from nose tip)
    pub cp_calibers: f64,
}

impl AeroCoefficients {
    /// Returns a zeroed-out `AeroCoefficients` (all fields set to 0.0).
    pub fn zero() -> Self {
        Self {
            cd: 0.0,
            cl: 0.0,
            cs: 0.0,
            cm: 0.0,
            cn: 0.0,
            cl_roll: 0.0,
            cn_alpha: 0.0,
            cp_calibers: 0.0,
        }
    }
}

/// Classification of the flow regime based on Mach number.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlowRegime {
    /// Mach < 0.8 — compressibility effects are minimal
    Subsonic,
    /// 0.8 <= Mach <= 1.2 — mixed subsonic/supersonic flow
    Transonic,
    /// 1.2 < Mach < 5.0 — fully supersonic flow
    Supersonic,
    /// Mach >= 5.0 — high-temperature gas effects dominate
    Hypersonic,
}

impl FlowRegime {
    /// Classify the flow regime from a Mach number.
    pub fn from_mach(mach: f64) -> Self {
        if mach < 0.8 {
            FlowRegime::Subsonic
        } else if mach <= 1.2 {
            FlowRegime::Transonic
        } else if mach < 5.0 {
            FlowRegime::Supersonic
        } else {
            FlowRegime::Hypersonic
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aero_forces_zero() {
        let f = AeroForces::zero();
        assert_eq!(f.drag, 0.0);
        assert_eq!(f.lift, 0.0);
        assert_eq!(f.side_force, 0.0);
        assert_eq!(f.pitch_moment, 0.0);
        assert_eq!(f.yaw_moment, 0.0);
        assert_eq!(f.roll_moment, 0.0);
        assert_eq!(f.cp_position, 0.0);
        assert_eq!(f.total_force(), 0.0);
    }

    #[test]
    fn test_aero_forces_total_force() {
        let f = AeroForces {
            drag: 10.0,
            lift: 5.0,
            side_force: 2.0,
            ..AeroForces::zero()
        };
        let expected = (10.0_f64 * 10.0 + 5.0 * 5.0 + 2.0 * 2.0).sqrt();
        assert!((f.total_force() - expected).abs() < 1e-12);
    }

    #[test]
    fn test_aero_coefficients_zero() {
        let c = AeroCoefficients::zero();
        assert_eq!(c.cd, 0.0);
        assert_eq!(c.cl, 0.0);
        assert_eq!(c.cs, 0.0);
        assert_eq!(c.cm, 0.0);
        assert_eq!(c.cn, 0.0);
        assert_eq!(c.cl_roll, 0.0);
        assert_eq!(c.cn_alpha, 0.0);
        assert_eq!(c.cp_calibers, 0.0);
    }

    #[test]
    fn test_flow_regime_classification() {
        assert_eq!(FlowRegime::from_mach(0.3), FlowRegime::Subsonic);
        assert_eq!(FlowRegime::from_mach(0.7), FlowRegime::Subsonic);
        assert_eq!(FlowRegime::from_mach(0.8), FlowRegime::Transonic);
        assert_eq!(FlowRegime::from_mach(0.9), FlowRegime::Transonic);
        assert_eq!(FlowRegime::from_mach(1.0), FlowRegime::Transonic);
        assert_eq!(FlowRegime::from_mach(1.2), FlowRegime::Transonic);
        assert_eq!(FlowRegime::from_mach(1.5), FlowRegime::Supersonic);
        assert_eq!(FlowRegime::from_mach(3.0), FlowRegime::Supersonic);
        assert_eq!(FlowRegime::from_mach(5.0), FlowRegime::Hypersonic);
        assert_eq!(FlowRegime::from_mach(10.0), FlowRegime::Hypersonic);
    }

    #[test]
    fn test_aero_forces_serde_roundtrip() {
        let f = AeroForces {
            drag: 12.5,
            lift: 3.2,
            side_force: 0.1,
            pitch_moment: 0.8,
            yaw_moment: 0.05,
            roll_moment: 0.0,
            cp_position: 0.75,
        };
        let json = serde_json::to_string(&f).unwrap();
        let deser: AeroForces = serde_json::from_str(&json).unwrap();
        assert!((deser.drag - 12.5).abs() < 1e-12);
        assert!((deser.cp_position - 0.75).abs() < 1e-12);
    }

    #[test]
    fn test_aero_coefficients_serde_roundtrip() {
        let c = AeroCoefficients {
            cd: 0.45,
            cl: 0.12,
            cs: 0.0,
            cm: -0.05,
            cn: 0.0,
            cl_roll: 0.0,
            cn_alpha: 8.5,
            cp_calibers: 2.3,
        };
        let json = serde_json::to_string(&c).unwrap();
        let deser: AeroCoefficients = serde_json::from_str(&json).unwrap();
        assert!((deser.cd - 0.45).abs() < 1e-12);
        assert!((deser.cn_alpha - 8.5).abs() < 1e-12);
        assert!((deser.cp_calibers - 2.3).abs() < 1e-12);
    }
}