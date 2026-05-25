use clap::Args;
use federated_rocket_motor_db::embedded;

#[derive(Args)]
pub struct MotorsArgs {
    /// List all embedded motors
    #[arg(short, long)]
    pub list: bool,

    /// Search by manufacturer
    #[arg(short, long)]
    pub manufacturer: Option<String>,

    /// Search by designation
    #[arg(short, long)]
    pub designation: Option<String>,

    /// Filter by impulse class (e.g., "C", "D", "H")
    #[arg(short, long)]
    pub impulse_class: Option<String>,

    /// Show motor details
    #[arg(short, long)]
    pub detailed: bool,

    /// JSON output
    #[arg(long)]
    pub json: bool,

    /// Path to motor database file (SQLite)
    #[arg(long)]
    pub db: Option<String>,
}

pub fn run(args: MotorsArgs) -> anyhow::Result<()> {
    // Get motors from embedded database
    let mut motors = embedded::embedded_motors();

    // Optionally load from SQLite database if path provided
    if let Some(ref db_path) = args.db {
        let _ = db_path; // SQLite integration is a future enhancement
        eprintln!("Note: External database loading is not yet implemented");
    }

    // Filter by manufacturer
    if let Some(ref mfr) = args.manufacturer {
        motors = motors
            .into_iter()
            .filter(|m| {
                m.manufacturer
                    .to_lowercase()
                    .contains(&mfr.to_lowercase())
            })
            .collect();
    }

    // Filter by designation
    if let Some(ref desig) = args.designation {
        motors = motors
            .into_iter()
            .filter(|m| {
                m.designation
                    .to_lowercase()
                    .contains(&desig.to_lowercase())
            })
            .collect();
    }

    // Filter by impulse class
    if let Some(ref class_str) = args.impulse_class {
        let class_upper = class_str.to_uppercase();
        motors = motors
            .into_iter()
            .filter(|m| m.impulse_class().display_name() == class_upper)
            .collect();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&motors)?);
    } else if args.list
        || args.manufacturer.is_some()
        || args.designation.is_some()
        || args.impulse_class.is_some()
    {
        println!("\n=== Available Motors ({} found) ===", motors.len());
        for motor in &motors {
            let class = motor.impulse_class().display_name();
            if args.detailed {
                println!(
                    "\n{} {} ({} class)",
                    motor.manufacturer, motor.designation, class
                );
                println!(
                    "  Diameter: {:.1}mm, Length: {:.1}mm",
                    motor.diameter, motor.length
                );
                println!("  Total Impulse: {:.1} N·s", motor.total_impulse);
                println!(
                    "  Burn Time: {:.2}s, Avg Thrust: {:.1}N",
                    motor.burn_time, motor.avg_thrust
                );
                println!(
                    "  Max Thrust: {:.1}N, Delay: {:.0}s",
                    motor.max_thrust, motor.delay_time
                );
                println!(
                    "  Propellant Mass: {:.1}g, Dry Mass: {:.1}g",
                    motor.propellant_mass, motor.dry_mass
                );
            } else {
                println!(
                    "  {} {} ({} class) — {:.1}N·s, {:.1}s burn",
                    motor.manufacturer, motor.designation, class, motor.total_impulse, motor.burn_time
                );
            }
        }
    } else {
        // Print help for motors command
        println!("Use 'federated-rocket motors --list' to show all available motors");
        println!("Use 'federated-rocket motors --manufacturer <name>' to search");
        println!("Use 'federated-rocket motors --designation <name>' to search by designation");
    }

    Ok(())
}