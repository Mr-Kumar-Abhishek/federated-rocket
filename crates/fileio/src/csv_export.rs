use std::path::Path;

// ============================================================================
// Export Data Types
// ============================================================================

/// A trajectory point for CSV export.
///
/// Downstream crates can convert their simulation state into this
/// format for export without requiring a direct dependency on the
/// simulation crate.
#[derive(Debug, Clone)]
pub struct ExportPoint {
    pub time: f64,
    pub altitude: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub mach: f64,
    pub angle_of_attack: f64,
    pub dynamic_pressure: f64,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
}

/// A flight event for CSV export.
#[derive(Debug, Clone)]
pub struct ExportEvent {
    pub time: f64,
    pub altitude: f64,
    pub event_type: String,
    pub description: String,
}

/// A thrust curve point for CSV export.
#[derive(Debug, Clone)]
pub struct ThrustCurvePoint {
    pub time: f64,
    pub thrust: f64,
    pub mass: f64,
    pub pressure: Option<f64>,
}

// ============================================================================
// CSV Export Handler
// ============================================================================

/// CSV export for simulation data.
pub struct CsvExport;

impl CsvExport {
    /// Export trajectory data to a CSV file.
    ///
    /// Columns: time (s), altitude (m), velocity (m/s), acceleration (m/s²),
    /// mach, angle_of_attack (deg), dynamic_pressure (Pa), position_x (m),
    /// position_y (m), position_z (m).
    pub fn export_trajectory(
        path: &Path,
        trajectory: &[ExportPoint],
    ) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);

        // Write header
        writer.write_record(&[
            "time_s",
            "altitude_m",
            "velocity_m_s",
            "acceleration_m_s2",
            "mach",
            "angle_of_attack_deg",
            "dynamic_pressure_pa",
            "position_x_m",
            "position_y_m",
            "position_z_m",
        ])?;

        // Write data rows
        for point in trajectory {
            writer.write_record(&[
                format_float(point.time),
                format_float(point.altitude),
                format_float(point.velocity),
                format_float(point.acceleration),
                format_float(point.mach),
                format_float(point.angle_of_attack),
                format_float(point.dynamic_pressure),
                format_float(point.position_x),
                format_float(point.position_y),
                format_float(point.position_z),
            ])?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Export flight events to a CSV file.
    ///
    /// Columns: time (s), altitude (m), event_type, description.
    pub fn export_events(path: &Path, events: &[ExportEvent]) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);

        writer.write_record(&["time_s", "altitude_m", "event_type", "description"])?;

        for event in events {
            let t = format_float(event.time);
            let a = format_float(event.altitude);
            writer.write_record(&[
                t.as_str(),
                a.as_str(),
                event.event_type.as_str(),
                event.description.as_str(),
            ])?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Export a motor thrust curve to a CSV file.
    ///
    /// Columns: time (s), thrust (N), mass (kg), pressure (Pa, optional).
    pub fn export_motor_curve(
        path: &Path,
        curve: &[ThrustCurvePoint],
    ) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);

        writer.write_record(&["time_s", "thrust_n", "mass_kg", "pressure_pa"])?;

        for point in curve {
            let t = format_float(point.time);
            let th = format_float(point.thrust);
            let m = format_float(point.mass);
            let p = point.pressure.map(|v| format_float(v)).unwrap_or_default();
            writer.write_record(&[t.as_str(), th.as_str(), m.as_str(), p.as_str()])?;
        }

        writer.flush()?;
        Ok(())
    }
}

fn format_float(v: f64) -> String {
    if v.is_nan() || v.is_infinite() {
        String::new()
    } else if v.fract() == 0.0 && v.abs() < 1e12 {
        format!("{:.1}", v)
    } else {
        format!("{:.6}", v)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_export_trajectory() {
        let trajectory = vec![
            ExportPoint {
                time: 0.0,
                altitude: 0.0,
                velocity: 0.0,
                acceleration: 0.0,
                mach: 0.0,
                angle_of_attack: 0.0,
                dynamic_pressure: 0.0,
                position_x: 0.0,
                position_y: 0.0,
                position_z: 0.0,
            },
            ExportPoint {
                time: 1.0,
                altitude: 10.0,
                velocity: 20.0,
                acceleration: 9.8,
                mach: 0.05,
                angle_of_attack: 0.5,
                dynamic_pressure: 500.0,
                position_x: 1.0,
                position_y: 0.0,
                position_z: 10.0,
            },
        ];

        let tmp = std::env::temp_dir().join("trajectory.csv");
        CsvExport::export_trajectory(&tmp, &trajectory).expect("export should succeed");

        let mut contents = String::new();
        std::fs::File::open(&tmp)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert!(contents.contains("time_s"));
        assert!(contents.contains("0.0"));
        assert!(contents.contains("20.0"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_export_events() {
        let events = vec![
            ExportEvent {
                time: 0.0,
                altitude: 0.0,
                event_type: "LAUNCH".into(),
                description: "Liftoff".into(),
            },
            ExportEvent {
                time: 2.5,
                altitude: 150.0,
                event_type: "BURNOUT".into(),
                description: "Motor burnout".into(),
            },
        ];

        let tmp = std::env::temp_dir().join("events.csv");
        CsvExport::export_events(&tmp, &events).expect("export should succeed");

        let mut contents = String::new();
        std::fs::File::open(&tmp)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert!(contents.contains("event_type"));
        assert!(contents.contains("LAUNCH"));
        assert!(contents.contains("BURNOUT"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_export_motor_curve() {
        let curve = vec![
            ThrustCurvePoint {
                time: 0.0,
                thrust: 0.0,
                mass: 0.1,
                pressure: None,
            },
            ThrustCurvePoint {
                time: 0.1,
                thrust: 50.0,
                mass: 0.09,
                pressure: Some(5000.0),
            },
            ThrustCurvePoint {
                time: 0.5,
                thrust: 0.0,
                mass: 0.08,
                pressure: None,
            },
        ];

        let tmp = std::env::temp_dir().join("motor.csv");
        CsvExport::export_motor_curve(&tmp, &curve).expect("export should succeed");

        let mut contents = String::new();
        std::fs::File::open(&tmp)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert!(contents.contains("thrust_n"));
        assert!(contents.contains("50"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_export_empty_trajectory() {
        let tmp = std::env::temp_dir().join("empty_traj.csv");
        CsvExport::export_trajectory(&tmp, &[]).expect("empty export should succeed");
        let mut contents = String::new();
        std::fs::File::open(&tmp)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        // Only header row
        assert!(contents.starts_with("time_s"));
        assert_eq!(contents.lines().count(), 1);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_export_trajectory_invalid_values() {
        let trajectory = vec![ExportPoint {
            time: f64::NAN,
            altitude: f64::INFINITY,
            velocity: 0.0,
            acceleration: 0.0,
            mach: 0.0,
            angle_of_attack: 0.0,
            dynamic_pressure: 0.0,
            position_x: 0.0,
            position_y: 0.0,
            position_z: 0.0,
        }];
        let tmp = std::env::temp_dir().join("nan.csv");
        let result = CsvExport::export_trajectory(&tmp, &trajectory);
        assert!(result.is_ok(), "NaN/Inf should not cause failure");
        let _ = std::fs::remove_file(&tmp);
    }
}
