use federated_rocket_core::component::*;
use federated_rocket_core::component_tree::*;
use federated_rocket_core::coordinate::Coordinate;
use federated_rocket_core::material::{Material, MaterialType};
use federated_rocket_core::units::{Quantity, Unit};
use std::io::{BufRead, BufReader};
use std::path::Path;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RktError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// ============================================================================
// RockSim File Handler
// ============================================================================

/// RockSim file handler for .rkt format.
///
/// RockSim uses a structured text format with pipe-delimited fields
/// and section headers. This implementation supports basic reading
/// and writing of the RockSim format.
pub struct RockSimFile;

impl RockSimFile {
    /// Load a .rkt file from the given path and return a ComponentTree.
    pub fn load(path: &Path) -> Result<ComponentTree, RktError> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        parse_rocksim(reader)
    }

    /// Save a ComponentTree as a .rkt file at the given path.
    pub fn save(path: &Path, tree: &ComponentTree) -> Result<(), RktError> {
        let mut output = String::new();

        // Write header
        output.push_str("$ROCKET_NAME|");
        if let Some(root_key) = tree.root() {
            if let Some(node) = tree.get(root_key) {
                output.push_str(node.component.name());
            }
        }
        output.push_str("||||\n");

        // Write components
        if let Some(root_key) = tree.root() {
            write_rocksim_components(&mut output, tree, root_key, 0)?;
        }

        std::fs::write(path, output.as_bytes())?;
        Ok(())
    }
}

// ============================================================================
// RockSim Parsing
// ============================================================================

/// Parse a RockSim format file from a BufReader.
///
/// RockSim format lines are pipe-delimited with the first field
/// indicating the record type (prefixed with $).
fn parse_rocksim(reader: BufReader<std::fs::File>) -> Result<ComponentTree, RktError> {
    let mut tree = ComponentTree::new();
    // Track current parent in the assembly hierarchy
    let mut assemblies: Vec<ComponentKey> = Vec::new();

    for (_, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| RktError::Io(e))?;
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('|').collect();
        if fields.is_empty() {
            continue;
        }

        let record_type = fields[0];
        let name = if fields.len() > 1 { fields[1] } else { "" };

        match record_type {
            "$ROCKET_NAME" => {
                // Root rocket assembly
                let assembly = RocketComponent::ComponentAssembly(ComponentAssemblyData {
                    name: if name.is_empty() { "Rocket".to_string() } else { name.to_string() },
                    position: Coordinate::origin(),
                    color: None,
                });
                let key = tree.add_component(assembly, None)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
                assemblies.push(key);
            }
            "$NOSE_CONETYPE" | "$NOSE_CONE" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(0.0);
                let diameter_in = parse_field_f64(&fields, 3).unwrap_or(0.0);
                let shape_name = if fields.len() > 4 { fields[4] } else { "Conical" };

                let nose = RocketComponent::NoseCone(NoseConeData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    base_radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    shape: match shape_name {
                        "0" | "Conical" => NoseConeShape::Conical,
                        "2" | "Ogive" => NoseConeShape::Ogive,
                        "3" | "Elliptical" => NoseConeShape::Elliptical,
                        "4" | "Parabolic" => NoseConeShape::Parabolic,
                        _ => NoseConeShape::Conical,
                    },
                    thickness: Quantity::new(0.002, Unit::Meter),
                    material: Material::new("Polystyrene", MaterialType::Bulk, Quantity::new(1050.0, Unit::Kilogram)),
                    color: None,
                    shoulder_length: Quantity::new(0.0, Unit::Meter),
                    shoulder_radius: Quantity::new(0.0, Unit::Meter),
                    is_blunted: false,
                    blunt_radius: Quantity::new(0.0, Unit::Meter),
                });
                tree.add_component(nose, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$BODY_TUBE" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(0.0);
                let diameter_in = parse_field_f64(&fields, 3).unwrap_or(0.0);

                let tube = RocketComponent::BodyTube(BodyTubeData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    outer_radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    inner_radius: Quantity::new(diameter_in * 0.5 * 0.0254 - 0.001, Unit::Meter),
                    material: Material::new("Cardboard", MaterialType::Bulk, Quantity::new(700.0, Unit::Kilogram)),
                    color: None,
                    has_motor_mount: false,
                });
                tree.add_component(tube, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$FIN_SET" => {
                let parent = assemblies.last().copied();
                let fin_count = parse_field_f64(&fields, 2).unwrap_or(3.0) as u32;
                let root_chord_in = parse_field_f64(&fields, 3).unwrap_or(2.0);
                let tip_chord_in = parse_field_f64(&fields, 4).unwrap_or(1.0);
                let span_in = parse_field_f64(&fields, 5).unwrap_or(1.0);
                let sweep_in = parse_field_f64(&fields, 6).unwrap_or(0.0);

                let fins = RocketComponent::FinSet(FinSetData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    fin_count,
                    root_chord: Quantity::new(root_chord_in * 0.0254, Unit::Meter),
                    tip_chord: Quantity::new(tip_chord_in * 0.0254, Unit::Meter),
                    span: Quantity::new(span_in * 0.0254, Unit::Meter),
                    sweep_length: Quantity::new(sweep_in * 0.0254, Unit::Meter),
                    thickness: Quantity::new(0.003, Unit::Meter),
                    cross_section: AirfoilType::Square,
                    material: Material::new("Balsa", MaterialType::Bulk, Quantity::new(160.0, Unit::Kilogram)),
                    color: None,
                    cant_angle: Quantity::new(0.0, Unit::Degree),
                    fin_placement: FinPlacement::Normal,
                });
                tree.add_component(fins, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$PARACHUTE" => {
                let parent = assemblies.last().copied();
                let diameter_in = parse_field_f64(&fields, 2).unwrap_or(12.0);

                let chute = RocketComponent::Parachute(ParachuteData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    diameter: Quantity::new(diameter_in * 0.0254, Unit::Meter),
                    cd: 0.8,
                    material: Material::new("Nylon", MaterialType::Bulk, Quantity::new(1150.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(chute, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$LAUNCH_LUG" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(1.0);
                let diameter_in = parse_field_f64(&fields, 3).unwrap_or(0.25);

                let lug = RocketComponent::LaunchLug(LaunchLugData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    outer_radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    inner_radius: Quantity::new((diameter_in * 0.5 - 0.02) * 0.0254, Unit::Meter),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    material: Material::new("Plastic", MaterialType::Bulk, Quantity::new(1040.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(lug, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$MASS_COMPONENT" | "$MASS" => {
                let parent = assemblies.last().copied();
                let mass_oz = parse_field_f64(&fields, 2).unwrap_or(1.0);

                let mass = RocketComponent::MassComponent(MassComponentData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    mass: Quantity::new(mass_oz * 0.0283495, Unit::Kilogram),
                    radius: Quantity::new(0.01, Unit::Meter),
                    material: Material::new("Lead", MaterialType::Bulk, Quantity::new(11340.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(mass, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$BULKHEAD" => {
                let parent = assemblies.last().copied();
                let diameter_in = parse_field_f64(&fields, 2).unwrap_or(1.0);

                let bh = RocketComponent::Bulkhead(BulkheadData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    outer_radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    inner_radius: Quantity::new(0.0, Unit::Meter),
                    thickness: Quantity::new(0.003, Unit::Meter),
                    material: Material::new("Plywood", MaterialType::Bulk, Quantity::new(600.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(bh, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$STREAMER" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(12.0);
                let width_in = parse_field_f64(&fields, 3).unwrap_or(2.0);

                let streamer = RocketComponent::Streamer(StreamerData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    width: Quantity::new(width_in * 0.0254, Unit::Meter),
                    cd: 1.0,
                    material: Material::new("Mylar", MaterialType::Surface, Quantity::new(1390.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(streamer, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$INNER_TUBE" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(6.0);
                let diameter_in = parse_field_f64(&fields, 3).unwrap_or(0.5);

                let inner = RocketComponent::InnerTube(InnerTubeData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    outer_radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    inner_radius: Quantity::new((diameter_in * 0.5 - 0.02) * 0.0254, Unit::Meter),
                    material: Material::new("Cardboard", MaterialType::Bulk, Quantity::new(700.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(inner, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$ENGINE_BLOCK" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(0.5);
                let diameter_in = parse_field_f64(&fields, 3).unwrap_or(0.5);

                let eb = RocketComponent::EngineBlock(EngineBlockData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    material: Material::new("Cardboard", MaterialType::Bulk, Quantity::new(700.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(eb, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$CENTERING_RING" => {
                let parent = assemblies.last().copied();
                let od_in = parse_field_f64(&fields, 2).unwrap_or(1.0);
                let id_in = parse_field_f64(&fields, 3).unwrap_or(0.5);

                let cr = RocketComponent::CenteringRing(CenteringRingData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    outer_radius: Quantity::new(od_in * 0.5 * 0.0254, Unit::Meter),
                    inner_radius: Quantity::new(id_in * 0.5 * 0.0254, Unit::Meter),
                    length: Quantity::new(0.005, Unit::Meter),
                    material: Material::new("Plywood", MaterialType::Bulk, Quantity::new(600.0, Unit::Kilogram)),
                    color: None,
                });
                tree.add_component(cr, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$TRANSITION" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(2.0);
                let fore_diam_in = parse_field_f64(&fields, 3).unwrap_or(1.0);
                let aft_diam_in = parse_field_f64(&fields, 4).unwrap_or(0.5);

                let trans = RocketComponent::Transition(TransitionData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    fore_radius: Quantity::new(fore_diam_in * 0.5 * 0.0254, Unit::Meter),
                    aft_radius: Quantity::new(aft_diam_in * 0.5 * 0.0254, Unit::Meter),
                    shape: TransitionShape::Conical,
                    thickness: Quantity::new(0.002, Unit::Meter),
                    material: Material::new("Cardboard", MaterialType::Bulk, Quantity::new(700.0, Unit::Kilogram)),
                    color: None,
                    shoulder_length: Quantity::new(0.0, Unit::Meter),
                    shoulder_radius: Quantity::new(0.0, Unit::Meter),
                });
                tree.add_component(trans, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
            }
            "$POD" => {
                let parent = assemblies.last().copied();
                let length_in = parse_field_f64(&fields, 2).unwrap_or(6.0);
                let diameter_in = parse_field_f64(&fields, 3).unwrap_or(0.5);

                let pod = RocketComponent::Pod(PodData {
                    name: name.to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(length_in * 0.0254, Unit::Meter),
                    radius: Quantity::new(diameter_in * 0.5 * 0.0254, Unit::Meter),
                    color: None,
                });
                let pod_key = tree.add_component(pod, parent)
                    .map_err(|e| RktError::Parse(format!("Tree error: {:?}", e)))?;
                assemblies.push(pod_key);
            }
            // Lines that don't match known types are ignored (comments, etc.)
            _ => {}
        }
    }

    Ok(tree)
}

/// Parse a field from the fields vector as f64, using the index if available.
fn parse_field_f64(fields: &[&str], index: usize) -> Option<f64> {
    fields.get(index).and_then(|s| s.trim().parse::<f64>().ok())
}

// ============================================================================
// RockSim Writing
// ============================================================================

/// Write components in RockSim format recursively.
fn write_rocksim_components(
    output: &mut String,
    tree: &ComponentTree,
    key: ComponentKey,
    _depth: usize,
) -> Result<(), RktError> {
    let node = tree.get(key).ok_or_else(|| {
        RktError::InvalidFormat("Component not found during write".to_string())
    })?;

    match &node.component {
        RocketComponent::BodyTube(data) => {
            let od_in = data.outer_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            let len_in = data.length.as_unit(Unit::Meter) / 0.0254;
            output.push_str(&format!("$BODY_TUBE|{}|{}|{}|||\n", data.name, len_in, od_in));
        }
        RocketComponent::NoseCone(data) => {
            let len_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let od_in = data.base_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            let shape_code = match data.shape {
                NoseConeShape::Conical => "0",
                NoseConeShape::Ogive => "2",
                NoseConeShape::Elliptical => "3",
                NoseConeShape::Parabolic => "4",
                _ => "0",
            };
            output.push_str(&format!("$NOSE_CONE|{}|{}|{}|{}||\n", data.name, len_in, od_in, shape_code));
        }
        RocketComponent::FinSet(data) => {
            let rc_in = data.root_chord.as_unit(Unit::Meter) / 0.0254;
            let tc_in = data.tip_chord.as_unit(Unit::Meter) / 0.0254;
            let sp_in = data.span.as_unit(Unit::Meter) / 0.0254;
            let sw_in = data.sweep_length.as_unit(Unit::Meter) / 0.0254;
            output.push_str(&format!(
                "$FIN_SET|{}|{}|{}|{}|{}|{}||\n",
                data.name, data.fin_count, rc_in, tc_in, sp_in, sw_in
            ));
        }
        RocketComponent::Parachute(data) => {
            let d_in = data.diameter.as_unit(Unit::Meter) / 0.0254;
            output.push_str(&format!("$PARACHUTE|{}|{}|||\n", data.name, d_in));
        }
        RocketComponent::Streamer(data) => {
            let l_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let w_in = data.width.as_unit(Unit::Meter) / 0.0254;
            output.push_str(&format!("$STREAMER|{}|{}|{}||\n", data.name, l_in, w_in));
        }
        RocketComponent::MassComponent(data) => {
            let mass_oz = data.mass.as_unit(Unit::Ounce);
            output.push_str(&format!("$MASS_COMPONENT|{}|{}|||\n", data.name, mass_oz));
        }
        RocketComponent::LaunchLug(data) => {
            let l_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let od_in = data.outer_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$LAUNCH_LUG|{}|{}|{}||\n", data.name, l_in, od_in));
        }
        RocketComponent::Bulkhead(data) => {
            let od_in = data.outer_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$BULKHEAD|{}|{}|||\n", data.name, od_in));
        }
        RocketComponent::Transition(data) => {
            let l_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let fd_in = data.fore_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            let ad_in = data.aft_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$TRANSITION|{}|{}|{}|{}||\n", data.name, l_in, fd_in, ad_in));
        }
        RocketComponent::InnerTube(data) => {
            let l_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let od_in = data.outer_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$INNER_TUBE|{}|{}|{}||\n", data.name, l_in, od_in));
        }
        RocketComponent::EngineBlock(data) => {
            let l_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let d_in = data.radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$ENGINE_BLOCK|{}|{}|{}||\n", data.name, l_in, d_in));
        }
        RocketComponent::CenteringRing(data) => {
            let od_in = data.outer_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            let id_in = data.inner_radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$CENTERING_RING|{}|{}|{}||\n", data.name, od_in, id_in));
        }
        RocketComponent::Pod(data) => {
            let l_in = data.length.as_unit(Unit::Meter) / 0.0254;
            let d_in = data.radius.as_unit(Unit::Meter) / 0.0254 * 2.0;
            output.push_str(&format!("$POD|{}|{}|{}||\n", data.name, l_in, d_in));
        }
        // Skip assembly nodes in output (they're implicit)
        _ => {}
    }

    // Write children recursively
    for child_key in tree.children(key) {
        write_rocksim_components(output, tree, child_key, _depth + 1)?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_rocksim() -> String {
        r#"$ROCKET_NAME|Test Rocket||||
$NOSE_CONE|Nose|5.0|1.0|0|
$BODY_TUBE|Main Body|30.0|1.0||
$FIN_SET|Fins|4|2.0|1.0|2.5|1.0|
$PARACHUTE|Main Chute|24.0|||
$LAUNCH_LUG|Launch Lug|2.0|0.25||
"#
        .to_string()
    }

    #[test]
    fn test_rkt_parse_minimal() {
        let rkt = make_test_rocksim();
        let cursor = std::io::Cursor::new(rkt.as_bytes());
        let _reader = BufReader::new(cursor);
        // Temporarily adapt for testing - parse_string doesn't exist
        // Instead test via a temp file
        let tmp = std::env::temp_dir().join("test_rocket.rkt");
        std::fs::write(&tmp, rkt.as_bytes()).unwrap();
        let tree = RockSimFile::load(&tmp).expect("Should parse RockSim");
        assert!(tree.component_count() >= 2, "Expected at least 2 components, got {}", tree.component_count());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_rkt_save_and_reload() {
        let mut tree = ComponentTree::new();
        let rocket = RocketComponent::ComponentAssembly(ComponentAssemblyData {
            name: "My Rocket".into(), position: Coordinate::origin(), color: None,
        });
        let root = tree.add_component(rocket, None).unwrap();

        let tube = RocketComponent::BodyTube(BodyTubeData {
            name: "Body".into(), position: Coordinate::origin(),
            length: Quantity::new(0.5, Unit::Meter), outer_radius: Quantity::new(0.02, Unit::Meter),
            inner_radius: Quantity::new(0.018, Unit::Meter),
            material: Material::new("Cardboard", MaterialType::Bulk, Quantity::new(700.0, Unit::Kilogram)),
            color: None, has_motor_mount: false,
        });
        tree.add_component(tube, Some(root)).unwrap();

        let nose = RocketComponent::NoseCone(NoseConeData {
            name: "Nose".into(), position: Coordinate::origin(),
            length: Quantity::new(0.1, Unit::Meter), base_radius: Quantity::new(0.02, Unit::Meter),
            shape: NoseConeShape::Conical, thickness: Quantity::new(0.002, Unit::Meter),
            material: Material::new("Polystyrene", MaterialType::Bulk, Quantity::new(1050.0, Unit::Kilogram)),
            color: None, shoulder_length: Quantity::new(0.0, Unit::Meter),
            shoulder_radius: Quantity::new(0.0, Unit::Meter), is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Meter),
        });
        tree.add_component(nose, Some(root)).unwrap();

        let tmp = std::env::temp_dir().join("test_save.rkt");
        RockSimFile::save(&tmp, &tree).expect("save should succeed");
        let loaded = RockSimFile::load(&tmp).expect("load should succeed");
        assert_eq!(loaded.component_count(), tree.component_count());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_rkt_error_empty() {
        let tmp = std::env::temp_dir().join("empty.rkt");
        std::fs::write(&tmp, "").unwrap();
        let tree = RockSimFile::load(&tmp).expect("Empty file should parse as empty tree");
        assert_eq!(tree.component_count(), 0);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_rkt_parse_all_component_types() {
        let rkt = r#"$ROCKET_NAME|Full Rocket||||
$NOSE_CONE|Nose|5.0|1.0|0|
$BODY_TUBE|Body|20.0|1.0||
$TRANSITION|Trans|2.0|1.0|0.5|
$FIN_SET|Fins|3|2.0|1.0|2.0|0.5|
$PARACHUTE|Main|24.0|||
$STREAMER|Streamer|36.0|3.0||
$LAUNCH_LUG|Lug|2.0|0.25||
$BULKHEAD|BH|1.0|||
$CENTERING_RING|CR|1.0|0.5||
$ENGINE_BLOCK|EB|0.5|0.5||
$INNER_TUBE|IT|6.0|0.5||
$MASS_COMPONENT|Mass|2.0|||
"#;
        let tmp = std::env::temp_dir().join("full_test.rkt");
        std::fs::write(&tmp, rkt).unwrap();
        let tree = RockSimFile::load(&tmp).expect("Should parse all types");
        assert!(tree.component_count() >= 2, "Expected components, got {}", tree.component_count());
        let _ = std::fs::remove_file(&tmp);
    }
}
