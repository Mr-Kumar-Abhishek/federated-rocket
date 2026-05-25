mod commands;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "federated-rocket")]
#[command(about = "Model rocket simulation and design tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Simulate a rocket flight
    Simulate(commands::simulate::SimulateArgs),
    /// Display information about a rocket design file
    Info(commands::info::InfoArgs),
    /// List or search available motors
    Motors(commands::motors::MotorsArgs),
    /// Optimize a rocket design parameter
    Optimize(commands::optimize::OptimizeArgs),
    /// Convert between rocket file formats
    Convert(commands::convert::ConvertArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Simulate(args) => commands::simulate::run(args),
        Command::Info(args) => commands::info::run(args),
        Command::Motors(args) => commands::motors::run(args),
        Command::Optimize(args) => commands::optimize::run(args),
        Command::Convert(args) => commands::convert::run(args),
    }
}