use clap::Args;
use federated_rocket_fileio::format_detect::{detect_format, load_rocket_file, RocketFileFormat};

#[derive(Args)]
pub struct ConvertArgs {
    /// Input file path
    pub input: String,

    /// Output file path
    pub output: String,
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    let input_path = std::path::Path::new(&args.input);
    let output_path = std::path::Path::new(&args.output);

    // Detect input format
    let input_format = detect_format(input_path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported input format: {}", args.input))?;

    // Detect output format
    let output_format = detect_format(output_path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported output format: {}", args.output))?;

    println!("Converting from {:?} to {:?}", input_format, output_format);

    // Load from input format
    let tree = load_rocket_file(input_path)?;

    // Save to output format
    match output_format {
        RocketFileFormat::OpenRocket => {
            federated_rocket_fileio::ork::OpenRocketFile::save(output_path, &tree)?;
        }
        RocketFileFormat::RockSim => {
            federated_rocket_fileio::rkt::RockSimFile::save(output_path, &tree)?;
        }
        RocketFileFormat::RASAero | RocketFileFormat::RockSimXML => {
            return Err(anyhow::anyhow!(
                "Output format {:?} is not yet supported for writing",
                output_format
            ));
        }
    }

    println!("Converted {} -> {}", args.input, args.output);
    Ok(())
}
