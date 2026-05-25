use crate::coordinate::Coordinate;
use crate::material::Material;
use crate::units::Quantity;
use serde::{Deserialize, Serialize};

// ============================================================================
// Supporting Enums
// ============================================================================

/// Nose cone shape profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoseConeShape {
    Conical,
    Ogive,
    Elliptical,
    Parabolic,
    PowerSeries(f64),
    VonKarman,
    HaackSeries(f64),
}

/// Transition shape profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionShape {
    Conical,
    Ogive,
    Elliptical,
    Parabolic,
    PowerSeries(f64),
}

/// Airfoil cross-section types for fins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AirfoilType {
    Square,
    Round,
    Wedge,
    Airfoil,
    Diamond,
    Hexagonal,
}

/// How fins are placed relative to the body tube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinPlacement {
    Normal,
    Inside,
    Fadec,
}

/// Deployment methods for recovery devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentType {
    Apogee,
    MotorEjection,
    Altimeter,
}

// ============================================================================
// Component Data Structs
// ============================================================================

/// Data for a body tube component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyTubeData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
    pub has_motor_mount: bool,
}

/// Data for a nose cone component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoseConeData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub base_radius: Quantity<f64>,
    pub shape: NoseConeShape,
    pub thickness: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
    pub shoulder_length: Quantity<f64>,
    pub shoulder_radius: Quantity<f64>,
    pub is_blunted: bool,
    pub blunt_radius: Quantity<f64>,
}

/// Data for a transition (conical or curved body section) component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub fore_radius: Quantity<f64>,
    pub aft_radius: Quantity<f64>,
    pub shape: TransitionShape,
    pub thickness: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
    pub shoulder_length: Quantity<f64>,
    pub shoulder_radius: Quantity<f64>,
}

/// Data for a parachute recovery component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParachuteData {
    pub name: String,
    pub position: Coordinate,
    pub diameter: Quantity<f64>,
    pub cd: f64,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a streamer recovery component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamerData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub width: Quantity<f64>,
    pub cd: f64,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a mass component (ballast, payload mass, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassComponentData {
    pub name: String,
    pub position: Coordinate,
    pub mass: Quantity<f64>,
    pub radius: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a bulkhead component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadData {
    pub name: String,
    pub position: Coordinate,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub thickness: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a centering ring component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CenteringRingData {
    pub name: String,
    pub position: Coordinate,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub length: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for an engine block component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineBlockData {
    pub name: String,
    pub position: Coordinate,
    pub radius: Quantity<f64>,
    pub length: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a launch lug component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchLugData {
    pub name: String,
    pub position: Coordinate,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub length: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a rail button component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RailButtonData {
    pub name: String,
    pub position: Coordinate,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub height: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a trapezoidal fin set component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinSetData {
    pub name: String,
    pub position: Coordinate,
    pub fin_count: u32,
    pub root_chord: Quantity<f64>,
    pub tip_chord: Quantity<f64>,
    pub span: Quantity<f64>,
    pub sweep_length: Quantity<f64>,
    pub thickness: Quantity<f64>,
    pub cross_section: AirfoilType,
    pub material: Material,
    pub color: Option<String>,
    pub cant_angle: Quantity<f64>,
    pub fin_placement: FinPlacement,
}

/// Data for a freeform fin set with user-defined points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeformFinSetData {
    pub name: String,
    pub position: Coordinate,
    pub fin_count: u32,
    pub points: Vec<Coordinate>,
    pub thickness: Quantity<f64>,
    pub cross_section: AirfoilType,
    pub material: Material,
    pub color: Option<String>,
    pub cant_angle: Quantity<f64>,
    pub fin_placement: FinPlacement,
}

/// Data for a pod (a side-mounted component cluster).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub radius: Quantity<f64>,
    pub color: Option<String>,
}

/// Data for an inner tube component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerTubeData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a tube coupler component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TubeCouplerData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub outer_radius: Quantity<f64>,
    pub inner_radius: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for a sleeve (outer wrap) component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleeveData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub outer_radius: Quantity<f64>,
    pub material: Material,
    pub color: Option<String>,
}

/// Data for an engine/motor component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineData {
    pub name: String,
    pub position: Coordinate,
    pub manufacturer: String,
    pub designation: String,
    pub diameter: Quantity<f64>,
    pub length: Quantity<f64>,
    pub total_impulse: Quantity<f64>,
    pub delay_time: Quantity<f64>,
    pub propellant_mass: Quantity<f64>,
    pub dry_mass: Quantity<f64>,
    pub color: Option<String>,
}

/// Data for a booster (stage) component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoosterData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub radius: Quantity<f64>,
    pub color: Option<String>,
    pub separation_event: Option<String>,
}

/// Data for a payload component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadData {
    pub name: String,
    pub position: Coordinate,
    pub length: Quantity<f64>,
    pub radius: Quantity<f64>,
    pub color: Option<String>,
}

/// Data for a recovery device (generic, used for ejection/deployment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryDeviceData {
    pub name: String,
    pub position: Coordinate,
    pub deployment_type: DeploymentType,
    pub color: Option<String>,
}

/// Data for a component assembly (grouping node).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAssemblyData {
    pub name: String,
    pub position: Coordinate,
    pub color: Option<String>,
}

// ============================================================================
// RocketComponent Tagged Enum
// ============================================================================

/// A tagged enum representing all possible rocket component types.
///
/// Each variant wraps a corresponding data struct that holds all
/// geometry, material, and configuration fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RocketComponent {
    BodyTube(BodyTubeData),
    NoseCone(NoseConeData),
    Transition(TransitionData),
    Parachute(ParachuteData),
    Streamer(StreamerData),
    MassComponent(MassComponentData),
    Bulkhead(BulkheadData),
    CenteringRing(CenteringRingData),
    EngineBlock(EngineBlockData),
    LaunchLug(LaunchLugData),
    RailButton(RailButtonData),
    FinSet(FinSetData),
    FreeformFinSet(FreeformFinSetData),
    Pod(PodData),
    InnerTube(InnerTubeData),
    TubeCoupler(TubeCouplerData),
    Sleeve(SleeveData),
    Engine(EngineData),
    Booster(BoosterData),
    Payload(PayloadData),
    RecoveryDevice(RecoveryDeviceData),
    ComponentAssembly(ComponentAssemblyData),
}

impl RocketComponent {
    /// Returns the human-readable name of this component.
    pub fn name(&self) -> &str {
        match self {
            RocketComponent::BodyTube(d) => &d.name,
            RocketComponent::NoseCone(d) => &d.name,
            RocketComponent::Transition(d) => &d.name,
            RocketComponent::Parachute(d) => &d.name,
            RocketComponent::Streamer(d) => &d.name,
            RocketComponent::MassComponent(d) => &d.name,
            RocketComponent::Bulkhead(d) => &d.name,
            RocketComponent::CenteringRing(d) => &d.name,
            RocketComponent::EngineBlock(d) => &d.name,
            RocketComponent::LaunchLug(d) => &d.name,
            RocketComponent::RailButton(d) => &d.name,
            RocketComponent::FinSet(d) => &d.name,
            RocketComponent::FreeformFinSet(d) => &d.name,
            RocketComponent::Pod(d) => &d.name,
            RocketComponent::InnerTube(d) => &d.name,
            RocketComponent::TubeCoupler(d) => &d.name,
            RocketComponent::Sleeve(d) => &d.name,
            RocketComponent::Engine(d) => &d.name,
            RocketComponent::Booster(d) => &d.name,
            RocketComponent::Payload(d) => &d.name,
            RocketComponent::RecoveryDevice(d) => &d.name,
            RocketComponent::ComponentAssembly(d) => &d.name,
        }
    }

    /// Returns the position (axial origin) of this component.
    pub fn position(&self) -> Coordinate {
        match self {
            RocketComponent::BodyTube(d) => d.position,
            RocketComponent::NoseCone(d) => d.position,
            RocketComponent::Transition(d) => d.position,
            RocketComponent::Parachute(d) => d.position,
            RocketComponent::Streamer(d) => d.position,
            RocketComponent::MassComponent(d) => d.position,
            RocketComponent::Bulkhead(d) => d.position,
            RocketComponent::CenteringRing(d) => d.position,
            RocketComponent::EngineBlock(d) => d.position,
            RocketComponent::LaunchLug(d) => d.position,
            RocketComponent::RailButton(d) => d.position,
            RocketComponent::FinSet(d) => d.position,
            RocketComponent::FreeformFinSet(d) => d.position,
            RocketComponent::Pod(d) => d.position,
            RocketComponent::InnerTube(d) => d.position,
            RocketComponent::TubeCoupler(d) => d.position,
            RocketComponent::Sleeve(d) => d.position,
            RocketComponent::Engine(d) => d.position,
            RocketComponent::Booster(d) => d.position,
            RocketComponent::Payload(d) => d.position,
            RocketComponent::RecoveryDevice(d) => d.position,
            RocketComponent::ComponentAssembly(d) => d.position,
        }
    }

    /// Sets the position of this component.
    pub fn set_position(&mut self, pos: Coordinate) {
        match self {
            RocketComponent::BodyTube(d) => d.position = pos,
            RocketComponent::NoseCone(d) => d.position = pos,
            RocketComponent::Transition(d) => d.position = pos,
            RocketComponent::Parachute(d) => d.position = pos,
            RocketComponent::Streamer(d) => d.position = pos,
            RocketComponent::MassComponent(d) => d.position = pos,
            RocketComponent::Bulkhead(d) => d.position = pos,
            RocketComponent::CenteringRing(d) => d.position = pos,
            RocketComponent::EngineBlock(d) => d.position = pos,
            RocketComponent::LaunchLug(d) => d.position = pos,
            RocketComponent::RailButton(d) => d.position = pos,
            RocketComponent::FinSet(d) => d.position = pos,
            RocketComponent::FreeformFinSet(d) => d.position = pos,
            RocketComponent::Pod(d) => d.position = pos,
            RocketComponent::InnerTube(d) => d.position = pos,
            RocketComponent::TubeCoupler(d) => d.position = pos,
            RocketComponent::Sleeve(d) => d.position = pos,
            RocketComponent::Engine(d) => d.position = pos,
            RocketComponent::Booster(d) => d.position = pos,
            RocketComponent::Payload(d) => d.position = pos,
            RocketComponent::RecoveryDevice(d) => d.position = pos,
            RocketComponent::ComponentAssembly(d) => d.position = pos,
        }
    }

    /// Returns a human-readable type name (e.g. "Body Tube", "Nose Cone").
    pub fn component_type(&self) -> &'static str {
        match self {
            RocketComponent::BodyTube(_) => "Body Tube",
            RocketComponent::NoseCone(_) => "Nose Cone",
            RocketComponent::Transition(_) => "Transition",
            RocketComponent::Parachute(_) => "Parachute",
            RocketComponent::Streamer(_) => "Streamer",
            RocketComponent::MassComponent(_) => "Mass Component",
            RocketComponent::Bulkhead(_) => "Bulkhead",
            RocketComponent::CenteringRing(_) => "Centering Ring",
            RocketComponent::EngineBlock(_) => "Engine Block",
            RocketComponent::LaunchLug(_) => "Launch Lug",
            RocketComponent::RailButton(_) => "Rail Button",
            RocketComponent::FinSet(_) => "Fin Set",
            RocketComponent::FreeformFinSet(_) => "Freeform Fin Set",
            RocketComponent::Pod(_) => "Pod",
            RocketComponent::InnerTube(_) => "Inner Tube",
            RocketComponent::TubeCoupler(_) => "Tube Coupler",
            RocketComponent::Sleeve(_) => "Sleeve",
            RocketComponent::Engine(_) => "Engine",
            RocketComponent::Booster(_) => "Booster",
            RocketComponent::Payload(_) => "Payload",
            RocketComponent::RecoveryDevice(_) => "Recovery Device",
            RocketComponent::ComponentAssembly(_) => "Component Assembly",
        }
    }

    /// Returns the minimum and maximum coordinates of the axis-aligned
    /// bounding box for this component.
    ///
    /// The bounding box is computed from the component's position and
    /// its primary dimensions. Components without explicit extents
    /// return a point-sized box at their position.
    pub fn bounding_box(&self) -> (Coordinate, Coordinate) {
        match self {
            RocketComponent::BodyTube(d) => {
                let l = *d.length.value();
                let r = *d.outer_radius.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::NoseCone(d) => {
                let l = *d.length.value();
                let sl = *d.shoulder_length.value();
                let r = *d.base_radius.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l + sl, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Transition(d) => {
                let l = *d.length.value();
                let sl = *d.shoulder_length.value();
                let r = d.fore_radius.value().max(*d.aft_radius.value());
                (
                    d.position,
                    Coordinate::new(d.position.x + l + sl, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Parachute(d) => {
                let r = *d.diameter.value() / 2.0;
                (
                    d.position,
                    Coordinate::new(d.position.x, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Streamer(d) => {
                let l = *d.length.value();
                let w = *d.width.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + w, d.position.z),
                )
            }
            RocketComponent::MassComponent(d) => {
                let r = *d.radius.value();
                (
                    d.position,
                    Coordinate::new(d.position.x, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Bulkhead(d) => {
                let r = *d.outer_radius.value();
                let t = *d.thickness.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + t, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::CenteringRing(d) => {
                let r = *d.outer_radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::EngineBlock(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::LaunchLug(d) => {
                let r = *d.outer_radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::RailButton(d) => {
                let r = *d.outer_radius.value();
                let h = *d.height.value();
                (
                    d.position,
                    Coordinate::new(d.position.x, d.position.y + r, d.position.z + h),
                )
            }
            RocketComponent::FinSet(d) => {
                let rc = *d.root_chord.value();
                let sp = *d.span.value();
                let t = *d.thickness.value() / 2.0;
                (
                    d.position,
                    Coordinate::new(d.position.x + rc, d.position.y + sp, d.position.z + t),
                )
            }
            RocketComponent::FreeformFinSet(d) => {
                // Estimate bounding box from the point list
                let mut min = d.position;
                let mut max = d.position;
                for p in &d.points {
                    min = Coordinate::new(
                        min.x.min(p.x),
                        min.y.min(p.y),
                        min.z.min(p.z),
                    );
                    max = Coordinate::new(
                        max.x.max(p.x),
                        max.y.max(p.y),
                        max.z.max(p.z),
                    );
                }
                (min, max)
            }
            RocketComponent::Pod(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::InnerTube(d) => {
                let r = *d.outer_radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::TubeCoupler(d) => {
                let r = *d.outer_radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Sleeve(d) => {
                let r = *d.outer_radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Engine(d) => {
                let r = *d.diameter.value() / 2.0;
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Booster(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::Payload(d) => {
                let r = *d.radius.value();
                let l = *d.length.value();
                (
                    d.position,
                    Coordinate::new(d.position.x + l, d.position.y + r, d.position.z + r),
                )
            }
            RocketComponent::RecoveryDevice(d) => {
                // No explicit dimensions; return a point box
                (d.position, d.position)
            }
            RocketComponent::ComponentAssembly(d) => {
                // No explicit dimensions; return a point box
                (d.position, d.position)
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;
    use crate::material::{Material, MaterialType};
    use crate::units::{Quantity, Unit};

    fn test_material() -> Material {
        Material::new("TestMaterial", MaterialType::Bulk, Quantity::new(1000.0, Unit::Kilogram))
    }

    #[test]
    fn test_body_tube_creation() {
        let tube = BodyTubeData {
            name: "Main Tube".to_string(),
            position: Coordinate::origin(),
            length: Quantity::new(100.0, Unit::Centimeter),
            outer_radius: Quantity::new(2.0, Unit::Centimeter),
            inner_radius: Quantity::new(1.9, Unit::Centimeter),
            material: test_material(),
            color: Some("#FFFFFF".to_string()),
            has_motor_mount: true,
        };
        let component = RocketComponent::BodyTube(tube);
        assert_eq!(component.name(), "Main Tube");
        assert_eq!(component.component_type(), "Body Tube");
        assert_eq!(component.position(), Coordinate::origin());
    }

    #[test]
    fn test_nose_cone_creation() {
        let nc = NoseConeData {
            name: "Ogive Nose".to_string(),
            position: Coordinate::new(1.0, 0.0, 0.0),
            length: Quantity::new(20.0, Unit::Centimeter),
            base_radius: Quantity::new(2.0, Unit::Centimeter),
            shape: NoseConeShape::Ogive,
            thickness: Quantity::new(0.2, Unit::Centimeter),
            material: test_material(),
            color: None,
            shoulder_length: Quantity::new(2.0, Unit::Centimeter),
            shoulder_radius: Quantity::new(1.9, Unit::Centimeter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Centimeter),
        };
        let component = RocketComponent::NoseCone(nc);
        assert_eq!(component.name(), "Ogive Nose");
        assert_eq!(component.component_type(), "Nose Cone");
    }

    #[test]
    fn test_fin_set_creation() {
        let fins = FinSetData {
            name: "Main Fins".to_string(),
            position: Coordinate::new(0.5, 0.0, 0.0),
            fin_count: 4,
            root_chord: Quantity::new(10.0, Unit::Centimeter),
            tip_chord: Quantity::new(5.0, Unit::Centimeter),
            span: Quantity::new(6.0, Unit::Centimeter),
            sweep_length: Quantity::new(2.0, Unit::Centimeter),
            thickness: Quantity::new(0.3, Unit::Centimeter),
            cross_section: AirfoilType::Airfoil,
            material: test_material(),
            color: Some("#FF0000".to_string()),
            cant_angle: Quantity::new(0.0, Unit::Degree),
            fin_placement: FinPlacement::Normal,
        };
        let component = RocketComponent::FinSet(fins);
        assert_eq!(component.name(), "Main Fins");
        assert_eq!(component.component_type(), "Fin Set");
        assert_eq!(component.position().x, 0.5);
    }

    #[test]
    fn test_set_position() {
        let tube = BodyTubeData {
            name: "Tube".to_string(),
            position: Coordinate::origin(),
            length: Quantity::new(50.0, Unit::Centimeter),
            outer_radius: Quantity::new(2.0, Unit::Centimeter),
            inner_radius: Quantity::new(1.8, Unit::Centimeter),
            material: test_material(),
            color: None,
            has_motor_mount: false,
        };
        let mut component = RocketComponent::BodyTube(tube);
        component.set_position(Coordinate::new(10.0, 0.0, 0.0));
        assert_eq!(component.position().x, 10.0);
    }

    #[test]
    fn test_bounding_box_body_tube() {
        let tube = BodyTubeData {
            name: "Tube".to_string(),
            position: Coordinate::origin(),
            length: Quantity::new(1.0, Unit::Meter),
            outer_radius: Quantity::new(2.0, Unit::Centimeter),
            inner_radius: Quantity::new(1.8, Unit::Centimeter),
            material: test_material(),
            color: None,
            has_motor_mount: false,
        };
        let component = RocketComponent::BodyTube(tube);
        let (min, max) = component.bounding_box();
        assert_eq!(min.x, 0.0);
        assert!((max.x - 1.0).abs() < 1e-6);
        assert!((max.y - 0.02).abs() < 1e-6);
        assert!((max.z - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_parachute_creation() {
        let chute = ParachuteData {
            name: "Main Chute".to_string(),
            position: Coordinate::new(0.8, 0.0, 0.0),
            diameter: Quantity::new(24.0, Unit::Inch),
            cd: 2.2,
            material: test_material(),
            color: Some("#FF00FF".to_string()),
        };
        let component = RocketComponent::Parachute(chute);
        assert_eq!(component.name(), "Main Chute");
        assert_eq!(component.component_type(), "Parachute");
    }

    #[test]
    fn test_streamer_creation() {
        let streamer = StreamerData {
            name: "Streamer".to_string(),
            position: Coordinate::origin(),
            length: Quantity::new(100.0, Unit::Centimeter),
            width: Quantity::new(10.0, Unit::Centimeter),
            cd: 1.5,
            material: test_material(),
            color: None,
        };
        let component = RocketComponent::Streamer(streamer);
        assert_eq!(component.name(), "Streamer");
    }

    #[test]
    fn test_engine_creation() {
        let engine = EngineData {
            name: "Estes A8".to_string(),
            position: Coordinate::origin(),
            manufacturer: "Estes".to_string(),
            designation: "A8".to_string(),
            diameter: Quantity::new(18.0, Unit::Millimeter),
            length: Quantity::new(70.0, Unit::Millimeter),
            total_impulse: Quantity::new(2.5, Unit::NewtonSecond),
            delay_time: Quantity::new(4.0, Unit::Second),
            propellant_mass: Quantity::new(6.0, Unit::Gram),
            dry_mass: Quantity::new(12.0, Unit::Gram),
            color: None,
        };
        let component = RocketComponent::Engine(engine);
        assert_eq!(component.name(), "Estes A8");
        assert_eq!(component.component_type(), "Engine");
    }

    #[test]
    fn test_nose_cone_shapes() {
        let _conical = NoseConeShape::Conical;
        let _ogive = NoseConeShape::Ogive;
        let _elliptical = NoseConeShape::Elliptical;
        let _parabolic = NoseConeShape::Parabolic;
        let _power = NoseConeShape::PowerSeries(0.5);
        let _von_karman = NoseConeShape::VonKarman;
        let _haack = NoseConeShape::HaackSeries(0.333);
        match _power {
            NoseConeShape::PowerSeries(v) => assert!((v - 0.5).abs() < 1e-6),
            _ => panic!("Expected PowerSeries"),
        }
    }

    #[test]
    fn test_transition_shapes() {
        let _conical = TransitionShape::Conical;
        let _ogive = TransitionShape::Ogive;
        let _elliptical = TransitionShape::Elliptical;
        let _parabolic = TransitionShape::Parabolic;
        let _power = TransitionShape::PowerSeries(0.5);
    }

    #[test]
    fn test_airfoil_types() {
        let _square = AirfoilType::Square;
        let _round = AirfoilType::Round;
        let _wedge = AirfoilType::Wedge;
        let _airfoil = AirfoilType::Airfoil;
        let _diamond = AirfoilType::Diamond;
        let _hex = AirfoilType::Hexagonal;
    }

    #[test]
    fn test_fin_placements() {
        let _normal = FinPlacement::Normal;
        let _inside = FinPlacement::Inside;
        let _fadec = FinPlacement::Fadec;
    }

    #[test]
    fn test_deployment_types() {
        let _apogee = DeploymentType::Apogee;
        let _motor = DeploymentType::MotorEjection;
        let _alt = DeploymentType::Altimeter;
    }

    #[test]
    fn test_all_variants_createable() {
        // Quick smoke-test that every variant can be constructed as a RocketComponent
        let mt = test_material();
        let pos = Coordinate::origin();

        let _ = RocketComponent::BodyTube(BodyTubeData {
            name: s(), position: pos, length: q(), outer_radius: q(), inner_radius: q(),
            material: mt.clone(), color: None, has_motor_mount: false,
        });
        let _ = RocketComponent::NoseCone(NoseConeData {
            name: s(), position: pos, length: q(), base_radius: q(), shape: NoseConeShape::Conical,
            thickness: q(), material: mt.clone(), color: None,
            shoulder_length: q(), shoulder_radius: q(), is_blunted: false, blunt_radius: q(),
        });
        let _ = RocketComponent::Transition(TransitionData {
            name: s(), position: pos, length: q(), fore_radius: q(), aft_radius: q(),
            shape: TransitionShape::Conical, thickness: q(), material: mt.clone(), color: None,
            shoulder_length: q(), shoulder_radius: q(),
        });
        let _ = RocketComponent::Parachute(ParachuteData {
            name: s(), position: pos, diameter: q(), cd: 1.0, material: mt.clone(), color: None,
        });
        let _ = RocketComponent::Streamer(StreamerData {
            name: s(), position: pos, length: q(), width: q(), cd: 1.0, material: mt.clone(), color: None,
        });
        let _ = RocketComponent::MassComponent(MassComponentData {
            name: s(), position: pos, mass: q(), radius: q(), material: mt.clone(), color: None,
        });
        let _ = RocketComponent::Bulkhead(BulkheadData {
            name: s(), position: pos, outer_radius: q(), inner_radius: q(), thickness: q(),
            material: mt.clone(), color: None,
        });
        let _ = RocketComponent::CenteringRing(CenteringRingData {
            name: s(), position: pos, outer_radius: q(), inner_radius: q(), length: q(),
            material: mt.clone(), color: None,
        });
        let _ = RocketComponent::EngineBlock(EngineBlockData {
            name: s(), position: pos, radius: q(), length: q(), material: mt.clone(), color: None,
        });
        let _ = RocketComponent::LaunchLug(LaunchLugData {
            name: s(), position: pos, outer_radius: q(), inner_radius: q(), length: q(),
            material: mt.clone(), color: None,
        });
        let _ = RocketComponent::RailButton(RailButtonData {
            name: s(), position: pos, outer_radius: q(), inner_radius: q(), height: q(),
            material: mt.clone(), color: None,
        });
        let _ = RocketComponent::FinSet(FinSetData {
            name: s(), position: pos, fin_count: 3, root_chord: q(), tip_chord: q(), span: q(),
            sweep_length: q(), thickness: q(), cross_section: AirfoilType::Square,
            material: mt.clone(), color: None, cant_angle: q(), fin_placement: FinPlacement::Normal,
        });
        let _ = RocketComponent::FreeformFinSet(FreeformFinSetData {
            name: s(), position: pos, fin_count: 3, points: vec![Coordinate::origin()],
            thickness: q(), cross_section: AirfoilType::Square, material: mt.clone(), color: None,
            cant_angle: q(), fin_placement: FinPlacement::Normal,
        });
        let _ = RocketComponent::Pod(PodData {
            name: s(), position: pos, length: q(), radius: q(), color: None,
        });
        let _ = RocketComponent::InnerTube(InnerTubeData {
            name: s(), position: pos, length: q(), outer_radius: q(), inner_radius: q(),
            material: mt.clone(), color: None,
        });
        let _ = RocketComponent::TubeCoupler(TubeCouplerData {
            name: s(), position: pos, length: q(), outer_radius: q(), inner_radius: q(),
            material: mt.clone(), color: None,
        });
        let _ = RocketComponent::Sleeve(SleeveData {
            name: s(), position: pos, length: q(), outer_radius: q(), material: mt.clone(), color: None,
        });
        let _ = RocketComponent::Engine(EngineData {
            name: s(), position: pos, manufacturer: s(), designation: s(), diameter: q(), length: q(),
            total_impulse: q(), delay_time: q(), propellant_mass: q(), dry_mass: q(), color: None,
        });
        let _ = RocketComponent::Booster(BoosterData {
            name: s(), position: pos, length: q(), radius: q(), color: None, separation_event: None,
        });
        let _ = RocketComponent::Payload(PayloadData {
            name: s(), position: pos, length: q(), radius: q(), color: None,
        });
        let _ = RocketComponent::RecoveryDevice(RecoveryDeviceData {
            name: s(), position: pos, deployment_type: DeploymentType::Apogee, color: None,
        });
        let _ = RocketComponent::ComponentAssembly(ComponentAssemblyData {
            name: s(), position: pos, color: None,
        });
    }

    /// Helper: short string
    fn s() -> String { "test".to_string() }
    /// Helper: 1 meter quantity
    fn q() -> Quantity<f64> { Quantity::new(1.0, Unit::Meter) }
}
