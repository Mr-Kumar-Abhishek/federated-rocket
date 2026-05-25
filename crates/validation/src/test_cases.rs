use crate::reference::*;
use federated_rocket_core::component::*;
use federated_rocket_core::component_tree::*;
use federated_rocket_core::material::{get_material, Material, MaterialType};
use federated_rocket_core::units::{Quantity, Unit};
use federated_rocket_core::coordinate::Coordinate;

/// Standard test case for validation
pub struct ValidationTestCase {
    pub name: &'static str,
    pub description: &'static str,
    pub build_tree: fn() -> ComponentTree,
    pub motor_designation: &'static str,
    pub reference: Option<ReferenceSimulation>,
    pub tolerances: ValidationTolerances,
}

/// Test Case 1: Simple Estes Alpha-like rocket (18mm, C6-5 motor)
pub fn test_case_simple_rocket() -> ValidationTestCase {
    ValidationTestCase {
        name: "simple_rocket",
        description: "Basic 18mm rocket with nose cone, body tube, 3 fins, C6-5 motor",
        build_tree: build_simple_rocket,
        motor_designation: "Estes C6-5",
        reference: None, // Will be loaded from reference data file
        tolerances: ValidationTolerances::default(),
    }
}

/// Build a simple 3FNC (3-Fin-Nose-Cone) rocket
pub fn build_simple_rocket() -> ComponentTree {
    let mut tree = ComponentTree::new();

    // Body tube: 18mm diameter, 30cm long
    let body_tube = RocketComponent::BodyTube(BodyTubeData {
        name: "Main Body".to_string(),
        position: Coordinate::origin(),
        length: Quantity::new(30.0, Unit::Centimeter),
        outer_radius: Quantity::new(9.525, Unit::Millimeter), // BT-50
        inner_radius: Quantity::new(8.731, Unit::Millimeter),
        material: get_material(Material::KRAFT_TUBE).unwrap(),
        color: Some("white".to_string()),
        has_motor_mount: true,
    });
    let body_key = tree.add_component(body_tube, None).unwrap();
    tree.set_root(body_key).unwrap();

    // Nose cone: ogive, 7.5cm long
    let nose = RocketComponent::NoseCone(NoseConeData {
        name: "Nose".to_string(),
        position: Coordinate::new(0.0, 0.0, 30.0), // at top of body tube
        length: Quantity::new(7.5, Unit::Centimeter),
        base_radius: Quantity::new(9.525, Unit::Millimeter),
        shape: NoseConeShape::Ogive,
        thickness: Quantity::new(0.5, Unit::Millimeter),
        material: get_material(Material::POLYSTYRENE).unwrap(),
        color: Some("red".to_string()),
        shoulder_length: Quantity::new(1.0, Unit::Centimeter),
        shoulder_radius: Quantity::new(8.731, Unit::Millimeter),
        is_blunted: false,
        blunt_radius: Quantity::new(0.0, Unit::Millimeter),
    });
    tree.add_component(nose, Some(body_key)).unwrap();

    // Fins: 3 trapezoidal fins
    let fins = RocketComponent::FinSet(FinSetData {
        name: "Fins".to_string(),
        position: Coordinate::new(0.0, 0.0, 5.0), // 5cm from bottom
        fin_count: 3,
        root_chord: Quantity::new(7.0, Unit::Centimeter),
        tip_chord: Quantity::new(3.0, Unit::Centimeter),
        span: Quantity::new(3.0, Unit::Centimeter),
        sweep_length: Quantity::new(2.0, Unit::Centimeter),
        thickness: Quantity::new(1.0, Unit::Millimeter),
        cross_section: AirfoilType::Square,
        material: get_material(Material::BALSA).unwrap(),
        color: Some("blue".to_string()),
        cant_angle: Quantity::new(0.0, Unit::Degree),
        fin_placement: FinPlacement::Normal,
    });
    tree.add_component(fins, Some(body_key)).unwrap();

    tree
}

/// Test Case 2: High-power rocket (54mm, J250 motor)
pub fn test_case_hpr_rocket() -> ValidationTestCase {
    ValidationTestCase {
        name: "hpr_rocket",
        description: "High-power 54mm rocket with dual deploy",
        build_tree: build_hpr_rocket,
        motor_designation: "Aerotech J250-15",
        reference: None,
        tolerances: ValidationTolerances {
            altitude_tolerance: 0.5, // 0.5% for HPR
            velocity_tolerance: 0.5,
            ..Default::default()
        },
    }
}

/// Build a high-power rocket
pub fn build_hpr_rocket() -> ComponentTree {
    let mut tree = ComponentTree::new();

    let body = RocketComponent::BodyTube(BodyTubeData {
        name: "Airframe".to_string(),
        position: Coordinate::origin(),
        length: Quantity::new(120.0, Unit::Centimeter),
        outer_radius: Quantity::new(27.0, Unit::Millimeter), // 54mm
        inner_radius: Quantity::new(25.5, Unit::Millimeter),
        material: get_material(Material::FIBERGLASS).unwrap(),
        color: Some("yellow".to_string()),
        has_motor_mount: true,
    });
    let body_key = tree.add_component(body, None).unwrap();
    tree.set_root(body_key).unwrap();

    // 4:1 ogive nose
    let nose = RocketComponent::NoseCone(NoseConeData {
        name: "Nose".to_string(),
        position: Coordinate::origin(),
        length: Quantity::new(21.6, Unit::Centimeter),
        base_radius: Quantity::new(27.0, Unit::Millimeter),
        shape: NoseConeShape::VonKarman,
        thickness: Quantity::new(1.0, Unit::Millimeter),
        material: get_material(Material::FIBERGLASS).unwrap(),
        color: Some("yellow".to_string()),
        shoulder_length: Quantity::new(2.0, Unit::Centimeter),
        shoulder_radius: Quantity::new(25.5, Unit::Millimeter),
        is_blunted: false,
        blunt_radius: Quantity::new(0.0, Unit::Millimeter),
    });
    tree.add_component(nose, Some(body_key)).unwrap();

    // 4 trapezoidal fins
    let fins = RocketComponent::FinSet(FinSetData {
        name: "Fins".to_string(),
        position: Coordinate::new(0.0, 0.0, 20.0),
        fin_count: 4,
        root_chord: Quantity::new(20.0, Unit::Centimeter),
        tip_chord: Quantity::new(10.0, Unit::Centimeter),
        span: Quantity::new(15.0, Unit::Centimeter),
        sweep_length: Quantity::new(5.0, Unit::Centimeter),
        thickness: Quantity::new(3.0, Unit::Millimeter),
        cross_section: AirfoilType::Airfoil,
        material: Material::new("G10", MaterialType::Bulk, Quantity::new(1800.0, Unit::Kilogram)),
        color: Some("black".to_string()),
        cant_angle: Quantity::new(0.0, Unit::Degree),
        fin_placement: FinPlacement::Normal,
    });
    tree.add_component(fins, Some(body_key)).unwrap();

    tree
}

/// Test Case 3: Minimum diameter rocket (29mm, G motor)
pub fn test_case_min_diameter() -> ValidationTestCase {
    ValidationTestCase {
        name: "min_diameter",
        description: "Minimum diameter 29mm rocket with G80 motor",
        build_tree: build_min_diameter_rocket,
        motor_designation: "Aerotech G80-10",
        reference: None,
        tolerances: ValidationTolerances::default(),
    }
}

pub fn build_min_diameter_rocket() -> ComponentTree {
    let mut tree = ComponentTree::new();

    let body = RocketComponent::BodyTube(BodyTubeData {
        name: "Airframe".to_string(),
        position: Coordinate::origin(),
        length: Quantity::new(90.0, Unit::Centimeter),
        outer_radius: Quantity::new(14.5, Unit::Millimeter),
        inner_radius: Quantity::new(13.2, Unit::Millimeter),
        material: get_material(Material::PHENOLIC).unwrap(),
        color: Some("green".to_string()),
        has_motor_mount: true,
    });
    let body_key = tree.add_component(body, None).unwrap();
    tree.set_root(body_key).unwrap();

    let nose = RocketComponent::NoseCone(NoseConeData {
        name: "Nose".to_string(),
        position: Coordinate::origin(),
        length: Quantity::new(14.5, Unit::Centimeter),
        base_radius: Quantity::new(14.5, Unit::Millimeter),
        shape: NoseConeShape::Elliptical,
        thickness: Quantity::new(0.8, Unit::Millimeter),
        material: get_material(Material::PHENOLIC).unwrap(),
        color: Some("green".to_string()),
        shoulder_length: Quantity::new(1.5, Unit::Centimeter),
        shoulder_radius: Quantity::new(13.2, Unit::Millimeter),
        is_blunted: false,
        blunt_radius: Quantity::new(0.0, Unit::Millimeter),
    });
    tree.add_component(nose, Some(body_key)).unwrap();

    // 3 fins
    let fins = RocketComponent::FinSet(FinSetData {
        name: "Fins".to_string(),
        position: Coordinate::new(0.0, 0.0, 15.0),
        fin_count: 3,
        root_chord: Quantity::new(12.0, Unit::Centimeter),
        tip_chord: Quantity::new(5.0, Unit::Centimeter),
        span: Quantity::new(8.0, Unit::Centimeter),
        sweep_length: Quantity::new(3.0, Unit::Centimeter),
        thickness: Quantity::new(1.5, Unit::Millimeter),
        cross_section: AirfoilType::Round,
        material: get_material(Material::PLYWOOD).unwrap(),
        color: Some("white".to_string()),
        cant_angle: Quantity::new(0.0, Unit::Degree),
        fin_placement: FinPlacement::Normal,
    });
    tree.add_component(fins, Some(body_key)).unwrap();

    tree
}
