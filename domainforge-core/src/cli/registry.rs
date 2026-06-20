use crate::registry::NamespaceRegistry;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct RegistryArgs {
    #[command(subcommand)]
    pub command: RegistryCommands,
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommands {
    /// Resolve the namespace for a file
    Resolve(ResolveArgs),
}

#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Fail if multiple namespaces match with equal specificity
    #[arg(long)]
    pub fail_on_ambiguity: bool,

    /// The file path to resolve
    pub file: PathBuf,
}

pub fn run(args: RegistryArgs) -> anyhow::Result<()> {
    match args.command {
        RegistryCommands::Resolve(resolve_args) => run_resolve(resolve_args),
    }
}

fn run_resolve(args: ResolveArgs) -> anyhow::Result<()> {
    let path = args.file;
    let registry = NamespaceRegistry::discover(&path)?
        .ok_or_else(|| anyhow::anyhow!("No .sea-registry.toml found"))?;

    let namespace = registry.namespace_for_with_options(&path, args.fail_on_ambiguity)?;
    println!("{}", namespace);
    Ok(())
}
