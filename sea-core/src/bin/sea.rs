use clap::Parser;
use sea_core::cli::{format, import, project, test, validate, validate_kg, Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // TODO: Setup logging/tracing based on cli.verbose/quiet

    match cli.command {
        Commands::Validate(args) => validate::run(args),
        Commands::Import(args) => import::run(args),
        Commands::Project(args) => project::run(args),
        Commands::Format(args) => format::run(args),
        Commands::Test(args) => test::run(args),
        Commands::ValidateKg(args) => validate_kg::run(args),
    }
}
