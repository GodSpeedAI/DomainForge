use clap::{Parser, Subcommand, ValueEnum};

pub mod format;
pub mod import;
pub mod normalize;
pub mod project;
pub mod registry;
pub mod test;
pub mod validate;
pub mod validate_kg;

#[derive(Parser)]
#[command(name = "sea", version, about = "SEA DSL CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    #[arg(long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Validate SEA files
    Validate(validate::ValidateArgs),
    /// Import from other formats
    Import(import::ImportArgs),
    /// Project/Export to other formats
    Project(project::ProjectArgs),
    /// Format SEA files
    #[command(name = "format", alias = "fmt")]
    Format(format::FormatArgs),
    /// Run tests
    Test(test::TestArgs),
    /// Validate Knowledge Graph files
    #[command(name = "validate-kg")]
    ValidateKg(validate_kg::ValidateKgArgs),
    /// Normalize a policy expression
    Normalize(normalize::NormalizeArgs),
    /// Registry management commands
    Registry(registry::RegistryArgs),
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}
