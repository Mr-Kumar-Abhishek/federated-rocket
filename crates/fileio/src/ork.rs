use federated_rocket_core::component::*;
use federated_rocket_core::component_tree::*;
use federated_rocket_core::coordinate::Coordinate;
use federated_rocket_core::material::{Material, MaterialType};
use federated_rocket_core::units::{Quantity, Unit};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::io::{Cursor as IoCursor, Read, Write};
use std::path::Path;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum OrkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parse error: {0}")]
    XmlParse(String),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
    #[error("Missing element: {0}")]
    MissingElement(String),
}

// ============================================================================
// Constants
// ============================================================================

const INCH_TO_METER: f64 = 0.0254;
const METER_TO_INCH: f64 = 1.0 / 0.0254;
const G_PER_CM3_TO_KG_PER_M3: f64 = 1000.0;
const KG_PER_M3_TO_G_PER_CM3: f64 = 1.0 / 1000.0;

// ============================================================================
// OpenRocket File Handler
// ============================================================================

pub struct OpenRocketFile;

impl OpenRocketFile {
    pub fn load(path: &Path) -> Result<ComponentTree, OrkError> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let xml_content = extract_xml(&mut archive)?;
        parse_ork_xml(&xml_content)
    }

    pub fn save(path: &Path, tree: &ComponentTree) -> Result<(), OrkError> {
        let xml_content = generate_ork_xml(tree)?;
        let file = std::fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        zip.start_file("rocket.ork", opts)?;
        zip.write_all(xml_content.as_bytes())?;
        zip.finish()?;
        Ok(())
    }

    pub fn test_roundtrip(path: &Path) -> Result<bool, OrkError> {
        let tree = Self::load(path)?;
        let tmp = std::env::temp_dir().join("roundtrip_test.ork");
        Self::save(&tmp, &tree)?;
        let reloaded = Self::load(&tmp)?;
        let _ = std::fs::remove_file(&tmp);
        Ok(tree.component_count() == reloaded.component_count())
    }
}

fn extract_xml(archive: &mut zip::ZipArchive<std::fs::File>) -> Result<String, OrkError> {
    for i in 0..archive.len() {
        let mut f = archive.by_index(i)?;
        let name = f.name();
        if name.ends_with(".ork") || name.ends_with(".xml") {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            return Ok(s);
        }
    }
    if archive.len() > 0 {
        let mut f = archive.by_index(0)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        return Ok(s);
    }
    Err(OrkError::InvalidFormat("empty zip".into()))
}

// ============================================================================
// Parsing
// ============================================================================

#[derive(Debug)]
struct ParsedComp {
    component: RocketComponent,
    children: Vec<ParsedComp>,
}

fn is_comp_elem(name: &str) -> bool {
    matches!(
        name,
        "BodyTube"
            | "NoseCone"
            | "Transition"
            | "FinSet"
            | "FreeformFinSet"
            | "Parachute"
            | "Streamer"
            | "MassComponent"
            | "Bulkhead"
            | "CenteringRing"
            | "EngineBlock"
            | "LaunchLug"
            | "InnerTube"
            | "TubeCoupler"
            | "Pod"
            | "Booster"
            | "Payload"
            | "ComponentAssembly"
            | "Sleeve"
            | "RailButton"
    )
}

fn parse_ork_xml(xml: &str) -> Result<ComponentTree, OrkError> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut saw_openrocket = false;

    let mut tree = ComponentTree::new();
    let mut stack: Vec<ParsedComp> = Vec::new();
    let mut current: Option<ParsedComp> = None;
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                text.clear();
                let n = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if n == "OpenRocketDocument" {
                    saw_openrocket = true;
                } else if n == "Subcomponents" {
                    if let Some(c) = current.take() {
                        stack.push(c);
                    }
                } else if is_comp_elem(&n) {
                    if !saw_openrocket {
                        return Err(OrkError::InvalidFormat(
                            "not a valid OpenRocket XML document".into(),
                        ));
                    }
                    current = Some(ParsedComp {
                        component: default_component(&n),
                        children: Vec::new(),
                    });
                }
            }
            Ok(Event::End(ref e)) => {
                let n = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if n == "Subcomponents" {
                    if let Some(p) = stack.pop() {
                        current = Some(p);
                    }
                } else if is_comp_elem(&n) {
                    if let Some(comp) = current.take() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(comp);
                        } else {
                            add_to_tree(&mut tree, &comp, None);
                        }
                    }
                } else if let Some(ref mut c) = current {
                    apply_field(&n, text.trim(), c);
                }
                text.clear();
            }
            Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"Motor" {
                    // Could parse motor config here
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Ok(t) = e.unescape() {
                    text = t.to_string();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OrkError::XmlParse(format!("parse error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if !saw_openrocket && tree.component_count() == 0 {
        return Err(OrkError::InvalidFormat(
            "not a valid OpenRocket XML document".into(),
        ));
    }

    Ok(tree)
}

fn add_to_tree(tree: &mut ComponentTree, pc: &ParsedComp, parent: Option<ComponentKey>) {
    let key = tree
        .add_component(pc.component.clone(), parent)
        .unwrap_or_else(|_| tree.add_component(pc.component.clone(), None).unwrap());
    for child in &pc.children {
        add_to_tree(tree, child, Some(key));
    }
}

// ============================================================================
// Default Components
// ============================================================================

fn default_component(name: &str) -> RocketComponent {
    macro_rules! q {
        () => {
            Quantity::new(0.0, Unit::Meter)
        };
    }
    let mat_cardboard = Material::new(
        "Cardboard",
        MaterialType::Bulk,
        Quantity::new(700.0, Unit::Kilogram),
    );
    let mat_balsa = Material::new(
        "Balsa",
        MaterialType::Bulk,
        Quantity::new(160.0, Unit::Kilogram),
    );

    match name {
        "BodyTube" => RocketComponent::BodyTube(BodyTubeData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            outer_radius: q!(),
            inner_radius: q!(),
            material: mat_cardboard,
            color: None,
            has_motor_mount: false,
        }),
        "NoseCone" => RocketComponent::NoseCone(NoseConeData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            base_radius: q!(),
            shape: NoseConeShape::Conical,
            thickness: q!(),
            material: Material::new(
                "Polystyrene",
                MaterialType::Bulk,
                Quantity::new(1050.0, Unit::Kilogram),
            ),
            color: None,
            shoulder_length: q!(),
            shoulder_radius: q!(),
            is_blunted: false,
            blunt_radius: q!(),
        }),
        "Transition" => RocketComponent::Transition(TransitionData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            fore_radius: q!(),
            aft_radius: q!(),
            shape: TransitionShape::Conical,
            thickness: q!(),
            material: mat_cardboard,
            color: None,
            shoulder_length: q!(),
            shoulder_radius: q!(),
        }),
        "FinSet" => RocketComponent::FinSet(FinSetData {
            name: String::new(),
            position: Coordinate::origin(),
            fin_count: 3,
            root_chord: q!(),
            tip_chord: q!(),
            span: q!(),
            sweep_length: q!(),
            thickness: Quantity::new(0.003, Unit::Meter),
            cross_section: AirfoilType::Square,
            material: mat_balsa,
            color: None,
            cant_angle: Quantity::new(0.0, Unit::Degree),
            fin_placement: FinPlacement::Normal,
        }),
        "FreeformFinSet" => RocketComponent::FreeformFinSet(FreeformFinSetData {
            name: String::new(),
            position: Coordinate::origin(),
            fin_count: 3,
            points: Vec::new(),
            thickness: Quantity::new(0.003, Unit::Meter),
            cross_section: AirfoilType::Square,
            material: mat_balsa,
            color: None,
            cant_angle: Quantity::new(0.0, Unit::Degree),
            fin_placement: FinPlacement::Normal,
        }),
        "Parachute" => RocketComponent::Parachute(ParachuteData {
            name: String::new(),
            position: Coordinate::origin(),
            diameter: q!(),
            cd: 0.8,
            material: Material::new(
                "Nylon",
                MaterialType::Bulk,
                Quantity::new(1150.0, Unit::Kilogram),
            ),
            color: None,
        }),
        "Streamer" => RocketComponent::Streamer(StreamerData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            width: q!(),
            cd: 1.0,
            material: Material::new(
                "Mylar",
                MaterialType::Surface,
                Quantity::new(1390.0, Unit::Kilogram),
            ),
            color: None,
        }),
        "MassComponent" => RocketComponent::MassComponent(MassComponentData {
            name: String::new(),
            position: Coordinate::origin(),
            mass: Quantity::new(0.0, Unit::Kilogram),
            radius: q!(),
            material: Material::new(
                "Lead",
                MaterialType::Bulk,
                Quantity::new(11340.0, Unit::Kilogram),
            ),
            color: None,
        }),
        "Bulkhead" => RocketComponent::Bulkhead(BulkheadData {
            name: String::new(),
            position: Coordinate::origin(),
            outer_radius: q!(),
            inner_radius: q!(),
            thickness: Quantity::new(0.003, Unit::Meter),
            material: Material::new(
                "Plywood",
                MaterialType::Bulk,
                Quantity::new(600.0, Unit::Kilogram),
            ),
            color: None,
        }),
        "CenteringRing" => RocketComponent::CenteringRing(CenteringRingData {
            name: String::new(),
            position: Coordinate::origin(),
            outer_radius: q!(),
            inner_radius: q!(),
            length: Quantity::new(0.005, Unit::Meter),
            material: Material::new(
                "Plywood",
                MaterialType::Bulk,
                Quantity::new(600.0, Unit::Kilogram),
            ),
            color: None,
        }),
        "EngineBlock" => RocketComponent::EngineBlock(EngineBlockData {
            name: String::new(),
            position: Coordinate::origin(),
            radius: q!(),
            length: Quantity::new(0.01, Unit::Meter),
            material: mat_cardboard,
            color: None,
        }),
        "LaunchLug" => RocketComponent::LaunchLug(LaunchLugData {
            name: String::new(),
            position: Coordinate::origin(),
            outer_radius: q!(),
            inner_radius: q!(),
            length: q!(),
            material: Material::new(
                "Plastic",
                MaterialType::Bulk,
                Quantity::new(1040.0, Unit::Kilogram),
            ),
            color: None,
        }),
        "InnerTube" => RocketComponent::InnerTube(InnerTubeData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            outer_radius: q!(),
            inner_radius: q!(),
            material: mat_cardboard,
            color: None,
        }),
        "TubeCoupler" => RocketComponent::TubeCoupler(TubeCouplerData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            outer_radius: q!(),
            inner_radius: q!(),
            material: mat_cardboard,
            color: None,
        }),
        "Pod" => RocketComponent::Pod(PodData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            radius: q!(),
            color: None,
        }),
        "Booster" => RocketComponent::Booster(BoosterData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            radius: q!(),
            color: None,
            separation_event: None,
        }),
        "Payload" => RocketComponent::Payload(PayloadData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            radius: q!(),
            color: None,
        }),
        "ComponentAssembly" => RocketComponent::ComponentAssembly(ComponentAssemblyData {
            name: String::new(),
            position: Coordinate::origin(),
            color: None,
        }),
        "Sleeve" => RocketComponent::Sleeve(SleeveData {
            name: String::new(),
            position: Coordinate::origin(),
            length: q!(),
            outer_radius: q!(),
            material: mat_cardboard,
            color: None,
        }),
        "RailButton" => RocketComponent::RailButton(RailButtonData {
            name: String::new(),
            position: Coordinate::origin(),
            outer_radius: q!(),
            inner_radius: q!(),
            height: q!(),
            material: Material::new(
                "Plastic",
                MaterialType::Bulk,
                Quantity::new(1040.0, Unit::Kilogram),
            ),
            color: None,
        }),
        _ => RocketComponent::ComponentAssembly(ComponentAssemblyData {
            name: name.to_string(),
            position: Coordinate::origin(),
            color: None,
        }),
    }
}

// ============================================================================
// Field Application
// ============================================================================

fn parse_len(v: &str) -> Quantity<f64> {
    Quantity::new(v.parse::<f64>().unwrap_or(0.0) * INCH_TO_METER, Unit::Meter)
}

fn parse_color(v: &str) -> Option<String> {
    let parts: Vec<f64> = v
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() >= 3 {
        let r = (parts[0].clamp(0.0, 1.0) * 255.0) as u8;
        let g = (parts[1].clamp(0.0, 1.0) * 255.0) as u8;
        let b = (parts[2].clamp(0.0, 1.0) * 255.0) as u8;
        Some(format!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        None
    }
}

fn set_mat(m: &mut Material, v: &str) {
    if let Ok(d) = v.parse::<f64>() {
        *m = Material::new(
            &m.name,
            m.material_type,
            Quantity::new(d * G_PER_CM3_TO_KG_PER_M3, Unit::Kilogram),
        );
    }
}

fn set_name_color(name: &mut String, color: &mut Option<String>, elem: &str, v: &str) -> bool {
    match elem {
        "Name" => {
            *name = v.to_string();
            true
        }
        "Color" => {
            *color = parse_color(v);
            true
        }
        _ => false,
    }
}

fn apply_field(elem: &str, v: &str, pc: &mut ParsedComp) {
    let c = &mut pc.component;
    match c {
        RocketComponent::BodyTube(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "OuterRadius" => d.outer_radius = parse_len(v),
                "OuterDiameter" => {
                    d.outer_radius = Quantity::new(
                        v.parse::<f64>().unwrap_or(0.0) * 0.5 * INCH_TO_METER,
                        Unit::Meter,
                    )
                }
                "InnerRadius" => d.inner_radius = parse_len(v),
                "InnerDiameter" => {
                    d.inner_radius = Quantity::new(
                        v.parse::<f64>().unwrap_or(0.0) * 0.5 * INCH_TO_METER,
                        Unit::Meter,
                    )
                }
                "MotorMount" => d.has_motor_mount = v.eq_ignore_ascii_case("true"),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::NoseCone(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "BaseRadius" | "OuterRadius" => d.base_radius = parse_len(v),
                "Thickness" => d.thickness = parse_len(v),
                "Shape" => {
                    d.shape = match v {
                        "Conical" => NoseConeShape::Conical,
                        "Ogive" => NoseConeShape::Ogive,
                        "Elliptical" => NoseConeShape::Elliptical,
                        "Parabolic" => NoseConeShape::Parabolic,
                        "Power Series" | "PowerSeries" | "Power" => NoseConeShape::PowerSeries(0.5),
                        "Von Karman" | "VonKarman" => NoseConeShape::VonKarman,
                        "Haack Series" | "HaackSeries" | "Haack" => {
                            NoseConeShape::HaackSeries(0.333)
                        }
                        _ => NoseConeShape::Conical,
                    }
                }
                "ShapeParameter" => {
                    if let Ok(p) = v.parse::<f64>() {
                        match d.shape {
                            NoseConeShape::PowerSeries(_) => {
                                d.shape = NoseConeShape::PowerSeries(p)
                            }
                            NoseConeShape::HaackSeries(_) => {
                                d.shape = NoseConeShape::HaackSeries(p)
                            }
                            _ => {}
                        }
                    }
                }
                "ShoulderLength" | "AftShoulderLength" => d.shoulder_length = parse_len(v),
                "ShoulderRadius" | "AftShoulderRadius" => d.shoulder_radius = parse_len(v),
                "IsBlunted" | "Blunted" => d.is_blunted = v.eq_ignore_ascii_case("true"),
                "BluntRadius" => d.blunt_radius = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::Transition(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "ForeRadius" => d.fore_radius = parse_len(v),
                "AftRadius" => d.aft_radius = parse_len(v),
                "Thickness" => d.thickness = parse_len(v),
                "Shape" => {
                    d.shape = match v {
                        "Conical" => TransitionShape::Conical,
                        "Ogive" => TransitionShape::Ogive,
                        "Elliptical" => TransitionShape::Elliptical,
                        "Parabolic" => TransitionShape::Parabolic,
                        "Power Series" | "PowerSeries" | "Power" => {
                            TransitionShape::PowerSeries(0.5)
                        }
                        _ => TransitionShape::Conical,
                    }
                }
                "ShapeParameter" => {
                    if let Ok(p) = v.parse::<f64>() {
                        if let TransitionShape::PowerSeries(_) = d.shape {
                            d.shape = TransitionShape::PowerSeries(p);
                        }
                    }
                }
                "ShoulderLength" | "AftShoulderLength" => d.shoulder_length = parse_len(v),
                "ShoulderRadius" | "AftShoulderRadius" => d.shoulder_radius = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::FinSet(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "FinCount" => {
                    d.fin_count = v.parse().unwrap_or(3);
                }
                "RootChord" => d.root_chord = parse_len(v),
                "TipChord" => d.tip_chord = parse_len(v),
                "Height" | "Span" => d.span = parse_len(v),
                "SweepLength" | "Sweep" => d.sweep_length = parse_len(v),
                "Thickness" => d.thickness = parse_len(v),
                "CrossSection" | "AirfoilType" => {
                    d.cross_section = match v {
                        "Square" => AirfoilType::Square,
                        "Round" => AirfoilType::Round,
                        "Airfoil" | "Airsdfoil" => AirfoilType::Airfoil,
                        "Wedge" => AirfoilType::Wedge,
                        "Diamond" => AirfoilType::Diamond,
                        "Hexagonal" => AirfoilType::Hexagonal,
                        _ => AirfoilType::Square,
                    }
                }
                "CantAngle" | "Cant" => {
                    d.cant_angle = Quantity::new(v.parse().unwrap_or(0.0), Unit::Degree)
                }
                "FinPlacement" | "Placement" => {
                    d.fin_placement = match v {
                        "Inside" => FinPlacement::Inside,
                        "Fadec" => FinPlacement::Fadec,
                        _ => FinPlacement::Normal,
                    }
                }
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::FreeformFinSet(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "FinCount" => {
                    d.fin_count = v.parse().unwrap_or(3);
                }
                "Thickness" => d.thickness = parse_len(v),
                "CrossSection" | "AirfoilType" => {
                    d.cross_section = match v {
                        "Square" => AirfoilType::Square,
                        "Round" => AirfoilType::Round,
                        "Airfoil" | "Airsdfoil" => AirfoilType::Airfoil,
                        "Wedge" => AirfoilType::Wedge,
                        "Diamond" => AirfoilType::Diamond,
                        "Hexagonal" => AirfoilType::Hexagonal,
                        _ => AirfoilType::Square,
                    }
                }
                "CantAngle" | "Cant" => {
                    d.cant_angle = Quantity::new(v.parse().unwrap_or(0.0), Unit::Degree)
                }
                "FinPlacement" | "Placement" => {
                    d.fin_placement = match v {
                        "Inside" => FinPlacement::Inside,
                        "Fadec" => FinPlacement::Fadec,
                        _ => FinPlacement::Normal,
                    }
                }
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::Parachute(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Diameter" => d.diameter = parse_len(v),
                "CD" | "Cd" => {
                    d.cd = v.parse().unwrap_or(0.8);
                }
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::Streamer(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "Width" => d.width = parse_len(v),
                "CD" | "Cd" => {
                    d.cd = v.parse().unwrap_or(1.0);
                }
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::MassComponent(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Mass" => {
                    d.mass = Quantity::new(v.parse::<f64>().unwrap_or(0.0) * 0.001, Unit::Kilogram)
                }
                "Radius" => d.radius = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::Bulkhead(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "OuterRadius" => d.outer_radius = parse_len(v),
                "InnerRadius" => d.inner_radius = parse_len(v),
                "Thickness" | "Length" => d.thickness = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::CenteringRing(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "OuterRadius" => d.outer_radius = parse_len(v),
                "InnerRadius" => d.inner_radius = parse_len(v),
                "Length" => d.length = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::EngineBlock(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "OuterRadius" | "Radius" => d.radius = parse_len(v),
                "Length" => d.length = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::LaunchLug(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "OuterRadius" => d.outer_radius = parse_len(v),
                "InnerRadius" => d.inner_radius = parse_len(v),
                "Length" => d.length = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::InnerTube(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "OuterRadius" => d.outer_radius = parse_len(v),
                "InnerRadius" => d.inner_radius = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::TubeCoupler(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "OuterRadius" => d.outer_radius = parse_len(v),
                "InnerRadius" => d.inner_radius = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::Pod(d) => {
            set_name_color(&mut d.name, &mut d.color, elem, v);
            match elem {
                "Length" => d.length = parse_len(v),
                "Radius" | "OuterRadius" => d.radius = parse_len(v),
                _ => {}
            }
        }
        RocketComponent::Booster(d) => {
            set_name_color(&mut d.name, &mut d.color, elem, v);
            match elem {
                "Length" => d.length = parse_len(v),
                "Radius" | "OuterRadius" => d.radius = parse_len(v),
                "SeparationEvent" => d.separation_event = Some(v.to_string()),
                _ => {}
            }
        }
        RocketComponent::Payload(d) => {
            set_name_color(&mut d.name, &mut d.color, elem, v);
            match elem {
                "Length" => d.length = parse_len(v),
                "Radius" | "OuterRadius" => d.radius = parse_len(v),
                _ => {}
            }
        }
        RocketComponent::Sleeve(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "Length" => d.length = parse_len(v),
                "OuterRadius" => d.outer_radius = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::RailButton(d) => {
            if set_name_color(&mut d.name, &mut d.color, elem, v) {
                return;
            }
            match elem {
                "OuterRadius" => d.outer_radius = parse_len(v),
                "InnerRadius" => d.inner_radius = parse_len(v),
                "Height" | "Length" => d.height = parse_len(v),
                "Material" => set_mat(&mut d.material, v),
                _ => {}
            }
        }
        RocketComponent::Engine(d) => {
            set_name_color(&mut d.name, &mut d.color, elem, v);
            match elem {
                "Manufacturer" => d.manufacturer = v.to_string(),
                "Designation" | "Code" => d.designation = v.to_string(),
                "Diameter" => d.diameter = parse_len(v),
                "Length" => d.length = parse_len(v),
                "TotalImpulse" => {
                    d.total_impulse = Quantity::new(v.parse().unwrap_or(0.0), Unit::NewtonSecond)
                }
                "DelayTime" => d.delay_time = Quantity::new(v.parse().unwrap_or(0.0), Unit::Second),
                "PropellantMass" => {
                    d.propellant_mass =
                        Quantity::new(v.parse::<f64>().unwrap_or(0.0) * 0.001, Unit::Kilogram)
                }
                "DryMass" => {
                    d.dry_mass =
                        Quantity::new(v.parse::<f64>().unwrap_or(0.0) * 0.001, Unit::Kilogram)
                }
                _ => {}
            }
        }
        RocketComponent::RecoveryDevice(d) => {
            set_name_color(&mut d.name, &mut d.color, elem, v);
        }
        RocketComponent::ComponentAssembly(d) => {
            set_name_color(&mut d.name, &mut d.color, elem, v);
        }
    }
}

// ============================================================================
// XML Generation
// ============================================================================

type XmlWriter = Writer<IoCursor<Vec<u8>>>;

fn generate_ork_xml(tree: &ComponentTree) -> Result<String, OrkError> {
    let mut w = Writer::new_with_indent(IoCursor::new(Vec::new()), b' ', 2);
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;

    let mut root = BytesStart::new("OpenRocketDocument");
    root.push_attribute(("xmlns:OpenRocket", "http://openrocket.sourceforge.net"));
    w.write_event(Event::Start(root))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;

    write_elem(&mut w, "Version", "1.6")?;
    let rname = tree
        .root()
        .and_then(|k| tree.get(k))
        .map(|n| n.component.name().to_string())
        .unwrap_or_else(|| "Rocket".into());
    write_elem(&mut w, "Name", &rname)?;
    write_elem(&mut w, "Designer", "")?;

    w.write_event(Event::Start(BytesStart::new("MotorConfiguration")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new("MotorConfiguration")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;

    w.write_event(Event::Start(BytesStart::new("Subcomponents")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    if let Some(rk) = tree.root() {
        if let Some(node) = tree.get(rk) {
            write_component(&mut w, tree, rk, &node.component)?;
        }
    }
    w.write_event(Event::End(BytesEnd::new("Subcomponents")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new("OpenRocketDocument")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;

    let inner = w.into_inner().into_inner();
    String::from_utf8(inner).map_err(|e| OrkError::XmlParse(format!("utf8: {}", e)))
}

fn write_elem(w: &mut XmlWriter, name: &str, value: &str) -> Result<(), OrkError> {
    w.write_event(Event::Start(BytesStart::new(name)))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    w.write_event(Event::Text(BytesText::new(value)))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    Ok(())
}

fn write_in(w: &mut XmlWriter, name: &str, q: &Quantity<f64>) -> Result<(), OrkError> {
    let v = q.as_unit(Unit::Meter) * METER_TO_INCH;
    write_elem(w, name, &format_float(v))
}

fn format_float(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

fn write_mat(w: &mut XmlWriter, name: &str, m: &Material) -> Result<(), OrkError> {
    let t = match m.material_type {
        MaterialType::Bulk => "bulk",
        MaterialType::Surface => "surface",
    };
    let mut e = BytesStart::new(name);
    e.push_attribute(("type", t));
    e.push_attribute(("name", m.name.as_str()));
    w.write_event(Event::Start(e))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    let d = *m.density.value() * KG_PER_M3_TO_G_PER_CM3;
    w.write_event(Event::Text(BytesText::new(&format_float(d))))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    Ok(())
}

fn color_str(c: &Option<String>) -> String {
    match c {
        Some(s) if s.len() >= 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(128);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(128);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(128);
            format!(
                "{:.3} {:.3} {:.3}",
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0
            )
        }
        _ => "0.5 0.5 0.5".into(),
    }
}

fn write_color(w: &mut XmlWriter, c: &Option<String>) -> Result<(), OrkError> {
    write_elem(w, "Color", &color_str(c))
}

fn write_subcomponents(
    w: &mut XmlWriter,
    tree: &ComponentTree,
    key: ComponentKey,
) -> Result<(), OrkError> {
    let children = tree.children(key);
    if children.is_empty() {
        return Ok(());
    }
    w.write_event(Event::Start(BytesStart::new("Subcomponents")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    for ck in &children {
        if let Some(node) = tree.get(*ck) {
            write_component(w, tree, *ck, &node.component)?;
        }
    }
    w.write_event(Event::End(BytesEnd::new("Subcomponents")))
        .map_err(|e| OrkError::XmlParse(e.to_string()))?;
    Ok(())
}

fn nose_shape(s: &NoseConeShape) -> &str {
    match s {
        NoseConeShape::Conical => "Conical",
        NoseConeShape::Ogive => "Ogive",
        NoseConeShape::Elliptical => "Elliptical",
        NoseConeShape::Parabolic => "Parabolic",
        NoseConeShape::PowerSeries(_) => "Power Series",
        NoseConeShape::VonKarman => "Von Karman",
        NoseConeShape::HaackSeries(_) => "Haack Series",
    }
}

fn trans_shape(s: &TransitionShape) -> &str {
    match s {
        TransitionShape::Conical => "Conical",
        TransitionShape::Ogive => "Ogive",
        TransitionShape::Elliptical => "Elliptical",
        TransitionShape::Parabolic => "Parabolic",
        TransitionShape::PowerSeries(_) => "Power Series",
    }
}

fn foil_name(f: &AirfoilType) -> &str {
    match f {
        AirfoilType::Square => "Square",
        AirfoilType::Round => "Round",
        AirfoilType::Wedge => "Wedge",
        AirfoilType::Airfoil => "Airfoil",
        AirfoilType::Diamond => "Diamond",
        AirfoilType::Hexagonal => "Hexagonal",
    }
}

fn write_component(
    w: &mut XmlWriter,
    tree: &ComponentTree,
    key: ComponentKey,
    comp: &RocketComponent,
) -> Result<(), OrkError> {
    match comp {
        RocketComponent::BodyTube(d) => {
            w.write_event(Event::Start(BytesStart::new("BodyTube")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_elem(
                w,
                "MotorMount",
                if d.has_motor_mount { "true" } else { "false" },
            )?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("BodyTube")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::NoseCone(d) => {
            w.write_event(Event::Start(BytesStart::new("NoseCone")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "BaseRadius", &d.base_radius)?;
            write_in(w, "Thickness", &d.thickness)?;
            write_elem(w, "Shape", nose_shape(&d.shape))?;
            if let NoseConeShape::PowerSeries(p) = d.shape {
                write_elem(w, "ShapeParameter", &format_float(p))?;
            }
            if let NoseConeShape::HaackSeries(p) = d.shape {
                write_elem(w, "ShapeParameter", &format_float(p))?;
            }
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_in(w, "ShoulderLength", &d.shoulder_length)?;
            write_in(w, "ShoulderRadius", &d.shoulder_radius)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("NoseCone")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Transition(d) => {
            w.write_event(Event::Start(BytesStart::new("Transition")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "ForeRadius", &d.fore_radius)?;
            write_in(w, "AftRadius", &d.aft_radius)?;
            write_in(w, "Thickness", &d.thickness)?;
            write_elem(w, "Shape", trans_shape(&d.shape))?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_in(w, "ShoulderLength", &d.shoulder_length)?;
            write_in(w, "ShoulderRadius", &d.shoulder_radius)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Transition")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::FinSet(d) => {
            w.write_event(Event::Start(BytesStart::new("FinSet")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_elem(w, "FinCount", &d.fin_count.to_string())?;
            write_in(w, "RootChord", &d.root_chord)?;
            write_in(w, "TipChord", &d.tip_chord)?;
            write_in(w, "Height", &d.span)?;
            write_in(w, "SweepLength", &d.sweep_length)?;
            write_in(w, "Thickness", &d.thickness)?;
            write_elem(w, "CrossSection", foil_name(&d.cross_section))?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            if d.cant_angle.as_unit(Unit::Degree).abs() > 0.001 {
                write_elem(
                    w,
                    "CantAngle",
                    &format_float(d.cant_angle.as_unit(Unit::Degree)),
                )?;
            }
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("FinSet")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::FreeformFinSet(d) => {
            w.write_event(Event::Start(BytesStart::new("FreeformFinSet")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_elem(w, "FinCount", &d.fin_count.to_string())?;
            write_in(w, "Thickness", &d.thickness)?;
            write_elem(w, "CrossSection", foil_name(&d.cross_section))?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("FreeformFinSet")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Parachute(d) => {
            w.write_event(Event::Start(BytesStart::new("Parachute")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Diameter", &d.diameter)?;
            write_elem(w, "CD", &format_float(d.cd))?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Parachute")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Streamer(d) => {
            w.write_event(Event::Start(BytesStart::new("Streamer")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "Width", &d.width)?;
            write_elem(w, "CD", &format_float(d.cd))?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Streamer")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::MassComponent(d) => {
            w.write_event(Event::Start(BytesStart::new("MassComponent")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_elem(w, "Mass", &format_float(d.mass.as_unit(Unit::Gram)))?;
            write_in(w, "Radius", &d.radius)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("MassComponent")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Bulkhead(d) => {
            w.write_event(Event::Start(BytesStart::new("Bulkhead")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_in(w, "Length", &d.thickness)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Bulkhead")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::CenteringRing(d) => {
            w.write_event(Event::Start(BytesStart::new("CenteringRing")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_in(w, "Length", &d.length)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("CenteringRing")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::EngineBlock(d) => {
            w.write_event(Event::Start(BytesStart::new("EngineBlock")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "OuterRadius", &d.radius)?;
            write_in(w, "Length", &d.length)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("EngineBlock")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::LaunchLug(d) => {
            w.write_event(Event::Start(BytesStart::new("LaunchLug")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_in(w, "Length", &d.length)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("LaunchLug")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::InnerTube(d) => {
            w.write_event(Event::Start(BytesStart::new("InnerTube")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("InnerTube")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::TubeCoupler(d) => {
            w.write_event(Event::Start(BytesStart::new("TubeCoupler")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("TubeCoupler")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Pod(d) => {
            w.write_event(Event::Start(BytesStart::new("Pod")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "Radius", &d.radius)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Pod")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Booster(d) => {
            w.write_event(Event::Start(BytesStart::new("Booster")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "Radius", &d.radius)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Booster")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Payload(d) => {
            w.write_event(Event::Start(BytesStart::new("Payload")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "Radius", &d.radius)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Payload")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Sleeve(d) => {
            w.write_event(Event::Start(BytesStart::new("Sleeve")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "Length", &d.length)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Sleeve")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::RailButton(d) => {
            w.write_event(Event::Start(BytesStart::new("RailButton")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_in(w, "OuterRadius", &d.outer_radius)?;
            write_in(w, "InnerRadius", &d.inner_radius)?;
            write_in(w, "Height", &d.height)?;
            write_mat(w, "Material", &d.material)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("RailButton")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::Engine(d) => {
            w.write_event(Event::Start(BytesStart::new("Engine")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_elem(w, "Manufacturer", &d.manufacturer)?;
            write_elem(w, "Designation", &d.designation)?;
            write_in(w, "Diameter", &d.diameter)?;
            write_in(w, "Length", &d.length)?;
            write_elem(
                w,
                "TotalImpulse",
                &format_float(d.total_impulse.as_unit(Unit::NewtonSecond)),
            )?;
            write_elem(
                w,
                "DelayTime",
                &format_float(d.delay_time.as_unit(Unit::Second)),
            )?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("Engine")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::RecoveryDevice(d) => {
            w.write_event(Event::Start(BytesStart::new("RecoveryDevice")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("RecoveryDevice")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
        RocketComponent::ComponentAssembly(d) => {
            w.write_event(Event::Start(BytesStart::new("ComponentAssembly")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
            write_elem(w, "Name", &d.name)?;
            write_color(w, &d.color)?;
            write_subcomponents(w, tree, key)?;
            w.write_event(Event::End(BytesEnd::new("ComponentAssembly")))
                .map_err(|e| OrkError::XmlParse(e.to_string()))?;
        }
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;

    fn make_test_ork_xml() -> String {
        r#"<?xml version='1.0' encoding='utf-8'?>
<OpenRocketDocument xmlns:OpenRocket="http://openrocket.sourceforge.net">
  <Version>1.6</Version>
  <Name>Test Rocket</Name>
  <Designer/>
  <MotorConfiguration/>
  <Subcomponents>
    <BodyTube>
      <Name>Main Body</Name>
      <Length>30.0</Length>
      <OuterRadius>0.5</OuterRadius>
      <InnerRadius>0.475</InnerRadius>
      <Material type="bulk" name="Cardboard">0.5</Material>
      <Color>0.5 0.5 0.5</Color>
      <Finish>Normal</Finish>
      <Filled>false</Filled>
      <MotorMount>false</MotorMount>
      <Subcomponents>
        <NoseCone>
          <Name>Nose</Name>
          <Length>5.0</Length>
          <Shape>Conical</Shape>
          <Thickness>0.02</Thickness>
          <Material type="bulk" name="Polystyrene">0.04</Material>
        </NoseCone>
      </Subcomponents>
    </BodyTube>
  </Subcomponents>
</OpenRocketDocument>"#
            .to_string()
    }

    fn create_test_ork_bytes() -> Vec<u8> {
        let xml = make_test_ork_xml();
        let mut buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut buf);
        zip.start_file(
            "rocket.ork",
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
        )
        .unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
        zip.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn test_ork_parse_minimal() {
        let xml = make_test_ork_xml();
        let tree = parse_ork_xml(&xml).expect("Should parse");
        assert!(
            tree.component_count() >= 1,
            "Expected at least 1 component, got {}",
            tree.component_count()
        );
    }

    #[test]
    fn test_ork_roundtrip_memory() {
        let bytes = create_test_ork_bytes();
        let tmp = std::env::temp_dir().join("test_roundtrip.ork");
        std::fs::write(&tmp, &bytes).unwrap();
        let result = OpenRocketFile::test_roundtrip(&tmp).expect("roundtrip failed");
        assert!(result, "roundtrip should succeed");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_ork_save_and_reload() {
        // Build a simple tree
        let mut tree = ComponentTree::new();
        let tube = RocketComponent::BodyTube(BodyTubeData {
            name: "Test Tube".into(),
            position: Coordinate::origin(),
            length: Quantity::new(0.762, Unit::Meter), // 30 inches
            outer_radius: Quantity::new(0.0127, Unit::Meter), // 0.5 inches
            inner_radius: Quantity::new(0.012065, Unit::Meter), // 0.475 inches
            material: Material::new(
                "Cardboard",
                MaterialType::Bulk,
                Quantity::new(500.0, Unit::Kilogram),
            ),
            color: None,
            has_motor_mount: false,
        });
        let body_key = tree.add_component(tube, None).unwrap();

        let nose = RocketComponent::NoseCone(NoseConeData {
            name: "Nose".into(),
            position: Coordinate::origin(),
            length: Quantity::new(0.127, Unit::Meter), // 5 inches
            base_radius: Quantity::new(0.0127, Unit::Meter),
            shape: NoseConeShape::Conical,
            thickness: Quantity::new(0.000508, Unit::Meter),
            material: Material::new(
                "Polystyrene",
                MaterialType::Bulk,
                Quantity::new(40.0, Unit::Kilogram),
            ),
            color: None,
            shoulder_length: Quantity::new(0.0, Unit::Meter),
            shoulder_radius: Quantity::new(0.0, Unit::Meter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Meter),
        });
        tree.add_component(nose, Some(body_key)).unwrap();

        let tmp = std::env::temp_dir().join("test_save.ork");
        OpenRocketFile::save(&tmp, &tree).expect("save should succeed");
        let loaded = OpenRocketFile::load(&tmp).expect("load should succeed");
        assert_eq!(loaded.component_count(), tree.component_count());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_ork_error_invalid_xml() {
        let result = parse_ork_xml("not xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_ork_error_empty_zip() {
        let tmp = std::env::temp_dir().join("empty.ork");
        let file = std::fs::File::create(&tmp).unwrap();
        let zip = zip::ZipWriter::new(file);
        zip.finish().unwrap();
        let result = OpenRocketFile::load(&tmp);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_ork_parse_field_values() {
        let xml = make_test_ork_xml();
        let tree = parse_ork_xml(&xml).expect("Should parse");
        // Find the body tube component
        for (_key, node) in tree.iter() {
            if let RocketComponent::BodyTube(ref data) = node.component {
                assert_eq!(data.name, "Main Body");
                // 30 inches = 0.762 m
                assert!(
                    (data.length.as_unit(Unit::Meter) - 0.762).abs() < 1e-6,
                    "Expected ~0.762m, got {}m",
                    data.length.as_unit(Unit::Meter)
                );
                // 0.5 inches = 0.0127 m
                assert!((data.outer_radius.as_unit(Unit::Meter) - 0.0127).abs() < 1e-6);
            }
        }
    }
}
