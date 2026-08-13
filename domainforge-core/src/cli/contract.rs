use crate::application::resolve::resolve_application_contract;
use crate::module::resolver::source_map_from_filesystem;
use crate::NamespaceRegistry;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;

/// Resolve and print one file's Application Contract document
/// (`domainforge-application-contract/v1`). Replaces the hand-built
/// `application-contract-harness.rs` pattern: no other subcommand reaches
/// `resolve_application_contract`.
#[derive(Parser)]
pub struct ContractArgs {
    #[arg(required = true)]
    pub target: PathBuf,
}

pub fn run(args: ContractArgs) -> Result<()> {
    let source = read_to_string(&args.target)
        .with_context(|| format!("Failed to read file {}", args.target.display()))?;
    let registry =
        NamespaceRegistry::discover(&args.target).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(&args.target).map(|ns| ns.to_string()));

    let (entry_logical_path, source_map) = source_map_from_filesystem(
        &args.target,
        &source,
        registry.as_ref(),
        default_namespace.as_deref(),
    )
    .map_err(|diagnostics| {
        anyhow::anyhow!(diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; "))
    })?;

    let sources_json =
        serde_json::to_string(&source_map.0).context("Failed to serialize resolved source map")?;

    match resolve_application_contract(&entry_logical_path, &sources_json) {
        Ok(doc) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&doc)
                    .context("Failed to serialize application contract")?
            );
            Ok(())
        }
        Err(diagnostics) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "error_count": diagnostics.len(),
                    "diagnostics": diagnostics,
                }))
                .context("Failed to serialize application diagnostics")?
            );
            Err(anyhow::anyhow!("Application contract resolution failed"))
        }
    }
}
