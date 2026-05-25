pub mod units;
pub mod coordinate;
pub mod material;

pub use units::*;
pub use coordinate::*;
pub use material::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_re_exports() {
        // Verify that the key types are accessible at the crate root
        let _q = Quantity::new(1.0, Unit::Meter);
        let _c = Coordinate::new(0.0, 0.0, 0.0);
        let _m = Material::new("Test", MaterialType::Bulk, Quantity::new(1000.0, Unit::Kilogram));
    }

    #[test]
    fn test_doc_roundtrip_meter_to_cm() {
        // Exactly replicate the doc example for Quantity
        let length = Quantity::new(100.0, Unit::Centimeter);
        assert!((length.as_unit(Unit::Meter) - 1.0).abs() < 1e-12);
    }
}
