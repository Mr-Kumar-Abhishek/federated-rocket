use clap::Args;
use federated_rocket_fileio::csv_export::{CsvExport, ExportPoint};
use federated_rocket_physics::atmosphere::StandardAtmosphere;
use federated_rocket_physics::gravity::ConstantGravity;
use federated_rocket_physics::wind::{ConstantWind, NoWind, WindModel};
use federated_rocket_simulation::engine::{SimulationConfig, SimulationEngine};
use federated_rocket_simulation::events::EventConfig;
use federated_rocket_simulation::motor::MotorModel;
use federated_rocket_simulation::state::FlightState;

#[derive(Args)]
pub struct SimulateArgs {
    /// Path to .ork or .rkt rocket design file
    #[arg(short, long)]
    pub file: String,

    /// Motor designation (e.g., "Estes C6-5") — overrides file motor
    #[arg(short, long)]
    pub motor: Option<String>,

    /// Output trajectory CSV file
    #[arg(short, long)]
    pub output: Option<String>,

    /// Maximum simulation time (seconds)
    #[arg(long, default_value = "120.0")]
    pub max_time: f64,

    /// Output interval (seconds)
    #[arg(long, default_value = "0.1")]
    pub output_interval: f64,

    /// Time step (seconds)
    #[arg(long, default_value = "0.001")]
    pub time_step: f64,

    /// Launch rod clearance altitude (meters)
    #[arg(long, default_value = "2.0")]
    pub rod_clear: f64,

    /// Launch site altitude ASL (meters)
    #[arg(long, default_value = "0.0")]
    pub launch_altitude: f64,

    /// Wind speed (m/s)
    #[arg(long, default_value = "0.0")]
    pub wind_speed: f64,

    /// Wind direction (degrees azimuth, 0=North)
    #[arg(long, default_value = "0.0")]
    pub wind_direction: f64,

    /// JSON output
    #[arg(long)]
    pub json: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn run(args: SimulateArgs) -> anyhow::Result<()> {
    // 1. Load rocket design from file
    let path = std::path::Path::new(&args.file);
    let tree = federated_rocket_fileio::format_detect::load_rocket_file(path)
        .map_err(|e| anyhow::anyhow!("Failed to load rocket file: {}", e))?;

    // 2. Setup atmosphere, gravity, wind models
    let atmosphere = StandardAtmosphere;
    let gravity = ConstantGravity;
    let wind: Box<dyn WindModel> = if args.wind_speed > 0.0 {
        Box::new(ConstantWind::new(args.wind_speed, args.wind_direction))
    } else {
        Box::new(NoWind)
    };

    // 3. Setup motor
    let motor: Option<MotorModel> = if let Some(ref motor_str) = args.motor {
        // Parse "Manufacturer Designation" format
        let parts: Vec<&str> = motor_str.splitn(2, ' ').collect();
        if parts.len() == 2 {
            // Try to find motor from embedded database
            let motors = federated_rocket_motor_db::embedded::embedded_motors();
            let matched = motors.iter().find(|m| {
                m.manufacturer.to_lowercase() == parts[0].to_lowercase()
                    && m.designation.to_lowercase() == parts[1].to_lowercase()
            });
            if let Some(motor_entry) = matched {
                // Convert motor-db Motor into simulation MotorModel
                let mut model = MotorModel::new(
                    motor_entry.manufacturer.clone(),
                    motor_entry.designation.clone(),
                );
                model.diameter = motor_entry.diameter;
                model.length = motor_entry.length;
                model.dry_mass = motor_entry.dry_mass / 1000.0; // g -> kg
                model.propellant_mass = motor_entry.propellant_mass / 1000.0; // g -> kg
                model.burn_time = motor_entry.burn_time;
                model.total_impulse = motor_entry.total_impulse;
                model.delay_time = motor_entry.delay_time;

                // Copy thrust curve points
                for tp in &motor_entry.thrust_curve {
                    model.add_thrust_point(tp.time, tp.thrust);
                }

                Some(model)
            } else {
                // Motor not found in database — create a simple model
                eprintln!("Motor '{}' not found in database, using placeholder", motor_str);
                Some(create_placeholder_motor(motor_str))
            }
        } else {
            // No manufacturer specified — create a simple model
            Some(create_placeholder_motor(motor_str))
        }
    } else {
        None
    };

    // 4. Setup simulation config
    let config = SimulationConfig {
        time_step: args.time_step,
        reference_area: std::f64::consts::PI * 0.0254 * 0.0254, // ~2" diameter default
        reference_diameter: 0.0508,
    };
    let event_config = EventConfig {
        launch_rod_clear_altitude: args.rod_clear,
        max_simulation_time: args.max_time,
        output_interval: args.output_interval,
        ground_altitude: if args.launch_altitude > 0.0 {
            args.launch_altitude
        } else {
            0.0
        },
        ..Default::default()
    };
    let engine = SimulationEngine::new(config, event_config);

    // 5. Run simulation
    let initial_state = FlightState::new();
    let result = engine.simulate(
        initial_state,
        motor,
        &tree,
        &atmosphere,
        &gravity,
        wind.as_ref(),
    );

    // 6. Output results
    if args.json {
        // SimulationResult does not derive Serialize, so build JSON manually
        let json_output = serde_json::json!({
            "flight_time": result.flight_time,
            "max_altitude": result.max_altitude,
            "max_velocity": result.max_velocity,
            "max_acceleration": result.max_acceleration,
            "apogee_time": result.apogee_time,
            "ground_hit_time": result.ground_hit_time,
            "success": result.success,
            "events": result.events.iter().map(|e| serde_json::json!({
                "time": e.time,
                "altitude": e.altitude,
                "velocity": e.velocity,
                "mach": e.mach,
                "acceleration": e.acceleration,
                "description": e.description,
            })).collect::<Vec<_>>(),
            "trajectory_length": result.trajectory.len(),
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        if args.verbose {
            println!("\n=== Simulation Configuration ===");
            println!("File: {}", args.file);
            println!("Time step: {:.6}s", args.time_step);
            println!("Output interval: {:.2}s", args.output_interval);
            println!("Max time: {:.1}s", args.max_time);
            if args.wind_speed > 0.0 {
                println!("Wind: {:.1}m/s from {:.0}°", args.wind_speed, args.wind_direction);
            }
            println!();
        }

        // Print summary
        println!("\n=== Simulation Results ===");
        println!("Flight time: {:.2} s", result.flight_time);
        println!(
            "Max altitude: {:.2} m ({:.1} ft)",
            result.max_altitude,
            result.max_altitude * 3.28084
        );
        println!(
            "Max velocity: {:.2} m/s ({:.1} mph)",
            result.max_velocity,
            result.max_velocity * 2.23694
        );
        println!(
            "Max acceleration: {:.2} m/s² ({:.1} G)",
            result.max_acceleration,
            result.max_acceleration / 9.80665
        );

        // Print events
        println!("\n=== Flight Events ===");
        for event in &result.events {
            println!(
            "{:.3}s: {} (alt: {:.1}m, vel: {:.1}m/s, mach: {:.2})",
            event.time, event.description, event.altitude, event.velocity, event.mach
        );
        }
    }

    // 7. Export CSV if requested
    if let Some(ref output_path) = args.output {
        let points: Vec<ExportPoint> = result
            .trajectory
            .iter()
            .map(|fs| ExportPoint {
                time: fs.time,
                altitude: fs.altitude(),
                velocity: fs.speed(),
                acceleration: 0.0, // would need to compute from delta-v
                mach: fs.mach,
                angle_of_attack: fs.angle_of_attack,
                dynamic_pressure: fs.dynamic_pressure,
                position_x: fs.position.x,
                position_y: fs.position.y,
                position_z: fs.position.z,
            })
            .collect();

        CsvExport::export_trajectory(std::path::Path::new(output_path), &points)?;
        println!("\nTrajectory exported to: {}", output_path);
    }

    Ok(())
}

/// Creates a placeholder motor with a simple thrust profile when the
/// requested motor is not found in the database.
fn create_placeholder_motor(designation: &str) -> MotorModel {
    let mut motor = MotorModel::new("Unknown".into(), designation.into());
    motor.diameter = 24.0;
    motor.length = 100.0;
    motor.dry_mass = 0.020;
    motor.propellant_mass = 0.010;
    motor.burn_time = 1.0;
    motor.total_impulse = 10.0;
    motor.delay_time = 3.0;

    // Simple triangular thrust profile
    motor.add_thrust_point(0.0, 0.0);
    motor.add_thrust_point(0.1, 15.0);
    motor.add_thrust_point(0.5, 12.0);
    motor.add_thrust_point(0.9, 8.0);
    motor.add_thrust_point(1.0, 0.0);

    motor
}