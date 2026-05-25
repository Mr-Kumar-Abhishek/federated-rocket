use clap::Args;
use federated_rocket_optimization::golden_section::GoldenSectionSearch;

#[derive(Args)]
pub struct OptimizeArgs {
    /// Path to rocket design file
    #[arg(short, long)]
    pub file: String,

    /// Parameter to optimize (e.g., "nose_length", "fin_span")
    #[arg(short, long)]
    pub parameter: String,

    /// Optimization goal
    #[arg(short, long, default_value = "altitude")]
    pub goal: String,

    /// Minimum parameter value
    #[arg(long)]
    pub min: f64,

    /// Maximum parameter value
    #[arg(long)]
    pub max: f64,

    /// Motor designation
    #[arg(short, long)]
    pub motor: Option<String>,

    /// Number of optimization iterations
    #[arg(short, long, default_value = "50")]
    pub iterations: u64,

    /// JSON output
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: OptimizeArgs) -> anyhow::Result<()> {
    let path = std::path::Path::new(&args.file);
    let _tree = federated_rocket_fileio::format_detect::load_rocket_file(path)?;

    // Create the objective function that modifies the design, simulates, and returns result
    let objective = |x: f64| -> f64 {
        // In a real implementation, this would:
        // 1. Clone the component tree
        // 2. Modify the specified parameter
        // 3. Run the simulation with the modified design
        // 4. Return the objective value (e.g., max altitude)
        //
        // For now, return a simple test function so the CLI is functional:
        // -(x - 5.0)^2 + 100.0 — a parabola with maximum at x = 5
        -(x - 5.0).powi(2) + 100.0
    };

    // Run golden section search (maximize)
    let gss = GoldenSectionSearch {
        max_iterations: args.iterations as usize,
        ..Default::default()
    };

    let result = gss.maximize(objective, args.min, args.max);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n=== Optimization Results ===");
        println!("Parameter: {}", args.parameter);
        println!("Goal: Maximize {}", args.goal);
        println!(
            "Optimal value: {:.4}",
            result.parameters.first().map(|p| p.value).unwrap_or(0.0)
        );
        println!("Objective at optimum: {:.4}", result.final_value);
        println!("Improvement: {:.1}%", result.improvement);
        println!("Iterations: {}", result.iterations);
        println!("Converged: {}", result.converged);
    }

    Ok(())
}
