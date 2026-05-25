use crate::units::{Quantity, Unit};
use serde::{Deserialize, Serialize};

/// The type of a material.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaterialType {
    /// A bulk material (e.g., wood, metal, plastic) with a density but no inherent thickness.
    Bulk,
    /// A surface material (e.g., paint, film, covering) with both density and thickness.
    Surface,
}

/// A material with a name, type, density, and optional thickness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Human-readable name of the material.
    pub name: String,
    /// Whether this is a bulk or surface material.
    pub material_type: MaterialType,
    /// Density in kg/m³ (SI).
    pub density: Quantity<f64>,
    /// Thickness for surface materials; `None` for bulk materials.
    pub thickness: Option<Quantity<f64>>,
}

impl Material {
    /// Creates a new bulk `Material`.
    ///
    /// # Examples
    ///
    /// ```
    /// use federated_rocket_core::material::{Material, MaterialType};
    /// use federated_rocket_core::units::{Quantity, Unit};
    ///
    /// let balsa = Material::new("Balsa", MaterialType::Bulk, Quantity::new(160.0, Unit::Kilogram));
    /// assert!(!balsa.is_surface());
    /// ```
    pub fn new(name: &str, material_type: MaterialType, density: Quantity<f64>) -> Self {
        Material {
            name: name.to_string(),
            material_type,
            density,
            thickness: None,
        }
    }

    /// Creates a new surface `Material` with a given thickness.
    ///
    /// # Examples
    ///
    /// ```
    /// use federated_rocket_core::material::Material;
    /// use federated_rocket_core::units::{Quantity, Unit};
    ///
    /// let paint = Material::new_surface("Paint", Quantity::new(1200.0, Unit::Kilogram), Quantity::new(0.05, Unit::Millimeter));
    /// assert!(paint.is_surface());
    /// ```
    pub fn new_surface(name: &str, density: Quantity<f64>, thickness: Quantity<f64>) -> Self {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Surface,
            density,
            thickness: Some(thickness),
        }
    }

    /// Returns `true` if this is a surface material.
    pub fn is_surface(&self) -> bool {
        self.material_type == MaterialType::Surface
    }

    /// Convenience method to create a bulk material from a name and density value in kg/m³.
    fn bulk(name: &str, density_kg_m3: f64) -> Self {
        Material::new(
            name,
            MaterialType::Bulk,
            Quantity::new(density_kg_m3, Unit::Kilogram),
        )
    }

    /// Convenience method to create a surface material from a name, density (kg/m³),
    /// and thickness value in mm.
    fn surface(name: &str, density_kg_m3: f64, thickness_mm: f64) -> Self {
        Material::new_surface(
            name,
            Quantity::new(density_kg_m3, Unit::Kilogram),
            Quantity::new(thickness_mm, Unit::Millimeter),
        )
    }
}

// ---------------------------------------------------------------------------
// Pre-defined constants for common model rocketry materials
// ---------------------------------------------------------------------------

impl Material {
    // ---- Bulk woods ----

    /// Birch wood (~670 kg/m³).
    pub const BIRCH: &'static str = "Birch";
    /// Basswood (~415 kg/m³).
    pub const BASSWOOD: &'static str = "Basswood";
    /// Balsa wood (~160 kg/m³).
    pub const BALSA: &'static str = "Balsa";
    /// Pine wood (~500 kg/m³).
    pub const PINE: &'static str = "Pine";
    /// Plywood (~600 kg/m³).
    pub const PLYWOOD: &'static str = "Plywood";

    // ---- Tube/construction materials ----

    /// Standard cardboard (~700 kg/m³).
    pub const CARDBOARD: &'static str = "Cardboard";
    /// Kraft paper tube (~700 kg/m³).
    pub const KRAFT_TUBE: &'static str = "Kraft Tube";
    /// Blue tube (heavy kraft tube, ~700 kg/m³).
    pub const BLUE_TUBE: &'static str = "Blue Tube";
    /// Phenolic tube (~1300 kg/m³).
    pub const PHENOLIC: &'static str = "Phenolic";

    // ---- Composites ----

    /// Fiberglass (~1850 kg/m³).
    pub const FIBERGLASS: &'static str = "Fiberglass";
    /// Carbon fiber (~1600 kg/m³).
    pub const CARBON_FIBER: &'static str = "Carbon Fiber";

    // ---- Metals ----

    /// Aluminum alloy (~2700 kg/m³).
    pub const ALUMINUM: &'static str = "Aluminum";
    /// Steel (~7850 kg/m³).
    pub const STEEL: &'static str = "Steel";
    /// Brass (~8500 kg/m³).
    pub const BRASS: &'static str = "Brass";
    /// Copper (~8960 kg/m³).
    pub const COPPER: &'static str = "Copper";
    /// Lead (~11340 kg/m³).
    pub const LEAD: &'static str = "Lead";

    // ---- Plastics ----

    /// Nylon (~1150 kg/m³).
    pub const NYLON: &'static str = "Nylon";
    /// ABS (~1040 kg/m³).
    pub const ABS: &'static str = "ABS";
    /// PLA (~1240 kg/m³).
    pub const PLA: &'static str = "PLA";
    /// PETG (~1270 kg/m³).
    pub const PETG: &'static str = "PETG";
    /// PVC (~1400 kg/m³).
    pub const PVC: &'static str = "PVC";
    /// Polystyrene (~1050 kg/m³).
    pub const POLYSTYRENE: &'static str = "Polystyrene";
    /// Polyurethane foam (~30 kg/m³).
    pub const POLYURETHANE_FOAM: &'static str = "Polyurethane Foam";
    /// Styrofoam (~50 kg/m³).
    pub const STYROFOAM: &'static str = "Styrofoam";

    // ---- Miscellaneous ----

    /// Cork (~240 kg/m³).
    pub const CORK: &'static str = "Cork";
    /// Rubber (~1200 kg/m³).
    pub const RUBBER: &'static str = "Rubber";
    /// Paper (~800 kg/m³).
    pub const PAPER: &'static str = "Paper";
    /// Epoxy (~1300 kg/m³).
    pub const EPOXY: &'static str = "Epoxy";
    /// Wax (~900 kg/m³).
    pub const WAX: &'static str = "Wax";
    /// Concrete (~2400 kg/m³).
    pub const CONCRETE: &'static str = "Concrete";

    // ---- Surface materials ----

    /// Gloss paper (density ~800 kg/m³, thickness 0.1 mm).
    pub const GLOSS_PAPER: &'static str = "Gloss Paper";
    /// Mylar film (density ~1390 kg/m³, thickness 0.025 mm).
    pub const MYLAR: &'static str = "Mylar";
    /// Metalized Mylar (density ~1390 kg/m³, thickness 0.025 mm).
    pub const MYLAR_METALIZED: &'static str = "Mylar (Metalized)";
    /// Vinyl film (density ~1400 kg/m³, thickness 0.1 mm).
    pub const VINYL: &'static str = "Vinyl";
    /// Monokote iron-on covering (density ~900 kg/m³, thickness 0.05 mm).
    pub const MONOKOTE: &'static str = "Monokote";
    /// Ultracote iron-on covering (density ~900 kg/m³, thickness 0.05 mm).
    pub const ULTRACOTE: &'static str = "Ultracote";
    /// Paint coating (density ~1200 kg/m³, thickness 0.05 mm).
    pub const PAINT: &'static str = "Paint";
    /// Chrome paint coating (density ~1200 kg/m³, thickness 0.05 mm).
    pub const CHROME_PAINT: &'static str = "Chrome Paint";
}

/// Returns a `Material` with the given well-known name.
///
/// This function provides access to the pre-defined material constants with
/// realistic density and thickness values.
///
/// # Examples
///
/// ```
/// use federated_rocket_core::material::{Material, get_material};
///
/// let balsa = get_material(Material::BALSA).unwrap();
/// assert_eq!(balsa.name, "Balsa");
/// assert!(!balsa.is_surface());
///
/// let mylar = get_material(Material::MYLAR).unwrap();
/// assert!(mylar.is_surface());
/// ```
pub fn get_material(name: &str) -> Option<Material> {
    match name {
        // Bulk woods
        Material::BIRCH => Some(Material::bulk("Birch", 670.0)),
        Material::BASSWOOD => Some(Material::bulk("Basswood", 415.0)),
        Material::BALSA => Some(Material::bulk("Balsa", 160.0)),
        Material::PINE => Some(Material::bulk("Pine", 500.0)),
        Material::PLYWOOD => Some(Material::bulk("Plywood", 600.0)),

        // Tube / construction
        Material::CARDBOARD => Some(Material::bulk("Cardboard", 700.0)),
        Material::KRAFT_TUBE => Some(Material::bulk("Kraft Tube", 700.0)),
        Material::BLUE_TUBE => Some(Material::bulk("Blue Tube", 700.0)),
        Material::PHENOLIC => Some(Material::bulk("Phenolic", 1300.0)),

        // Composites
        Material::FIBERGLASS => Some(Material::bulk("Fiberglass", 1850.0)),
        Material::CARBON_FIBER => Some(Material::bulk("Carbon Fiber", 1600.0)),

        // Metals
        Material::ALUMINUM => Some(Material::bulk("Aluminum", 2700.0)),
        Material::STEEL => Some(Material::bulk("Steel", 7850.0)),
        Material::BRASS => Some(Material::bulk("Brass", 8500.0)),
        Material::COPPER => Some(Material::bulk("Copper", 8960.0)),
        Material::LEAD => Some(Material::bulk("Lead", 11340.0)),

        // Plastics
        Material::NYLON => Some(Material::bulk("Nylon", 1150.0)),
        Material::ABS => Some(Material::bulk("ABS", 1040.0)),
        Material::PLA => Some(Material::bulk("PLA", 1240.0)),
        Material::PETG => Some(Material::bulk("PETG", 1270.0)),
        Material::PVC => Some(Material::bulk("PVC", 1400.0)),
        Material::POLYSTYRENE => Some(Material::bulk("Polystyrene", 1050.0)),
        Material::POLYURETHANE_FOAM => Some(Material::bulk("Polyurethane Foam", 30.0)),
        Material::STYROFOAM => Some(Material::bulk("Styrofoam", 50.0)),

        // Miscellaneous
        Material::CORK => Some(Material::bulk("Cork", 240.0)),
        Material::RUBBER => Some(Material::bulk("Rubber", 1200.0)),
        Material::PAPER => Some(Material::bulk("Paper", 800.0)),
        Material::EPOXY => Some(Material::bulk("Epoxy", 1300.0)),
        Material::WAX => Some(Material::bulk("Wax", 900.0)),
        Material::CONCRETE => Some(Material::bulk("Concrete", 2400.0)),

        // Surface materials
        Material::GLOSS_PAPER => Some(Material::surface("Gloss Paper", 800.0, 0.1)),
        Material::MYLAR => Some(Material::surface("Mylar", 1390.0, 0.025)),
        Material::MYLAR_METALIZED => Some(Material::surface("Mylar (Metalized)", 1390.0, 0.025)),
        Material::VINYL => Some(Material::surface("Vinyl", 1400.0, 0.1)),
        Material::MONOKOTE => Some(Material::surface("Monokote", 900.0, 0.05)),
        Material::ULTRACOTE => Some(Material::surface("Ultracote", 900.0, 0.05)),
        Material::PAINT => Some(Material::surface("Paint", 1200.0, 0.05)),
        Material::CHROME_PAINT => Some(Material::surface("Chrome Paint", 1200.0, 0.05)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_material() {
        let balsa = get_material(Material::BALSA).unwrap();
        assert_eq!(balsa.name, "Balsa");
        assert_eq!(balsa.material_type, MaterialType::Bulk);
        assert!(!balsa.is_surface());
        assert!(balsa.thickness.is_none());

        // Density ~160 kg/m³
        let density_kg_m3 = balsa.density.as_unit(Unit::Kilogram);
        assert!((density_kg_m3 - 160.0).abs() < 1e-9);
    }

    #[test]
    fn test_surface_material() {
        let paint = get_material(Material::PAINT).unwrap();
        assert_eq!(paint.name, "Paint");
        assert_eq!(paint.material_type, MaterialType::Surface);
        assert!(paint.is_surface());
        assert!(paint.thickness.is_some());

        let thickness_mm = paint.thickness.unwrap().as_unit(Unit::Millimeter);
        assert!((thickness_mm - 0.05).abs() < 1e-9);

        let density_kg_m3 = paint.density.as_unit(Unit::Kilogram);
        assert!((density_kg_m3 - 1200.0).abs() < 1e-9);
    }

    #[test]
    fn test_new_bulk() {
        let mat = Material::new("Test", MaterialType::Bulk, Quantity::new(500.0, Unit::Kilogram));
        assert_eq!(mat.name, "Test");
        assert!(!mat.is_surface());
        assert!(mat.thickness.is_none());
    }

    #[test]
    fn test_new_surface() {
        let mat = Material::new_surface(
            "Test Surface",
            Quantity::new(1000.0, Unit::Kilogram),
            Quantity::new(0.5, Unit::Millimeter),
        );
        assert_eq!(mat.name, "Test Surface");
        assert!(mat.is_surface());
        assert!(mat.thickness.is_some());
    }

    #[test]
    fn test_all_known_materials_exist() {
        let known = [
            Material::BIRCH,
            Material::BASSWOOD,
            Material::BALSA,
            Material::PINE,
            Material::PLYWOOD,
            Material::CARDBOARD,
            Material::KRAFT_TUBE,
            Material::BLUE_TUBE,
            Material::PHENOLIC,
            Material::FIBERGLASS,
            Material::CARBON_FIBER,
            Material::ALUMINUM,
            Material::STEEL,
            Material::BRASS,
            Material::COPPER,
            Material::LEAD,
            Material::NYLON,
            Material::ABS,
            Material::PLA,
            Material::PETG,
            Material::PVC,
            Material::POLYSTYRENE,
            Material::POLYURETHANE_FOAM,
            Material::STYROFOAM,
            Material::CORK,
            Material::RUBBER,
            Material::PAPER,
            Material::EPOXY,
            Material::WAX,
            Material::CONCRETE,
            Material::GLOSS_PAPER,
            Material::MYLAR,
            Material::MYLAR_METALIZED,
            Material::VINYL,
            Material::MONOKOTE,
            Material::ULTRACOTE,
            Material::PAINT,
            Material::CHROME_PAINT,
        ];
        for name in &known {
            assert!(
                get_material(name).is_some(),
                "Missing material: {}",
                name
            );
        }
    }

    #[test]
    fn test_unknown_material() {
        assert!(get_material("NonExistent").is_none());
    }
}