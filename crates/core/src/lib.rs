pub mod component;
pub mod component_tree;
pub mod coordinate;
pub mod material;
pub mod units;

pub use component::*;
pub use component_tree::*;
pub use coordinate::*;
pub use material::*;
pub use units::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_re_exports() {
        // Verify that the key types are accessible at the crate root
        let _q = Quantity::new(1.0, Unit::Meter);
        let _c = Coordinate::new(0.0, 0.0, 0.0);
        let _m = Material::new(
            "Test",
            MaterialType::Bulk,
            Quantity::new(1000.0, Unit::Kilogram),
        );
    }

    #[test]
    fn test_doc_roundtrip_meter_to_cm() {
        // Exactly replicate the doc example for Quantity
        let length = Quantity::new(100.0, Unit::Centimeter);
        assert!((length.as_unit(Unit::Meter) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_component_re_export() {
        // Verify that key component types are accessible at the crate root
        let _tube = RocketComponent::BodyTube(BodyTubeData {
            name: "Test".to_string(),
            position: Coordinate::origin(),
            length: Quantity::new(1.0, Unit::Meter),
            outer_radius: Quantity::new(0.02, Unit::Meter),
            inner_radius: Quantity::new(0.018, Unit::Meter),
            material: Material::new(
                "Cardboard",
                MaterialType::Bulk,
                Quantity::new(600.0, Unit::Kilogram),
            ),
            color: None,
            has_motor_mount: false,
        });
    }

    #[test]
    fn test_component_tree_re_export() {
        let mut tree = ComponentTree::new();
        let key = tree
            .add_component(
                RocketComponent::ComponentAssembly(ComponentAssemblyData {
                    name: "Rocket".to_string(),
                    position: Coordinate::origin(),
                    color: None,
                }),
                None,
            )
            .unwrap();
        assert!(tree.root() == Some(key));
    }
}
