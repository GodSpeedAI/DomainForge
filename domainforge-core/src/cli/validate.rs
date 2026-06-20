use crate::parser::{parse_to_graph_with_options, ParseOptions};
use crate::{Graph, NamespaceRegistry};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub struct ValidateArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub format: OutputFormat,

    #[arg(long)]
    pub no_color: bool,

    #[arg(long)]
    pub show_source: bool,

    #[arg(required = true)]
    pub target: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum OutputFormat {
    Json,
    Human,
    Lsp,
}

pub fn run(args: ValidateArgs) -> Result<()> {
    let use_color = !args.no_color;

    if args.target.is_dir() {
        validate_directory(&args.target, args.format, use_color, args.show_source)
    } else {
        validate_file(&args.target, args.format, use_color, args.show_source)
    }
}

fn validate_file(
    path: &Path,
    format: OutputFormat,
    use_color: bool,
    show_source: bool,
) -> Result<()> {
    let source =
        read_to_string(path).with_context(|| format!("Failed to read file {}", path.display()))?;
    let registry = NamespaceRegistry::discover(path).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(path).map(|ns| ns.to_string()));
    let options = ParseOptions {
        default_namespace,
        namespace_registry: registry.clone(),
        entry_path: Some(path.to_path_buf()),
        ..Default::default()
    };
    let graph = parse_to_graph_with_options(&source, &options)
        .map_err(|e| anyhow::anyhow!("Parse failed for {}: {}", path.display(), e))?;
    report_validation(graph, format, use_color, show_source, Some(&source))
}

fn validate_directory(
    path: &Path,
    format: OutputFormat,
    use_color: bool,
    show_source: bool,
) -> Result<()> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| anyhow::anyhow!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No .sea-registry.toml found for {}. Run inside a workspace with a registry file.",
                path.display()
            )
        })?;

    let files = registry
        .resolve_files()
        .map_err(|e| anyhow::anyhow!("Failed to expand registry: {}", e))?;

    if files.is_empty() {
        return Err(anyhow::anyhow!(
            "Registry at '{}' did not match any .sea files",
            registry.root().display()
        ));
    }

    let mut graph = Graph::new();
    for binding in files {
        let source = read_to_string(&binding.path)
            .with_context(|| format!("Failed to read {}", binding.path.display()))?;
        let options = ParseOptions {
            default_namespace: Some(binding.namespace.clone()),
            namespace_registry: Some(registry.clone()),
            entry_path: Some(binding.path.clone()),
            ..Default::default()
        };
        let file_graph = parse_to_graph_with_options(&source, &options)
            .map_err(|e| anyhow::anyhow!("Parse failed for {}: {}", binding.path.display(), e))?;
        graph
            .extend(file_graph)
            .map_err(|e| anyhow::anyhow!("Failed to merge {}: {}", binding.path.display(), e))?;
    }

    // For directory validation, we don't pass source code for now as errors could be from any file
    // TODO: Map errors back to specific files in directory mode
    report_validation(graph, format, use_color, show_source, None)
}

fn report_validation(
    graph: Graph,
    format: OutputFormat,
    use_color: bool,
    show_source: bool,
    source: Option<&str>,
) -> Result<()> {
    // Note: show_source and source are currently unused because validation violations
    // don't yet include source range information. These parameters are kept for future
    // implementation when source snippets can be displayed.
    let _ = (show_source, source); // Acknowledge parameters for future use

    let result = graph.validate();

    match format {
        OutputFormat::Json => {
            let json_output = serde_json::json!({
                "error_count": result.error_count,
                "violations": result.violations.iter().map(|v| {
                    serde_json::json!({
                        "severity": match v.severity {
                            crate::policy::Severity::Error => "error",
                            crate::policy::Severity::Warning => "warning",
                            crate::policy::Severity::Info => "info",
                        },
                        "policy_name": v.policy_name,
                        "message": v.message,
                        "context": v.context,
                    })
                }).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json_output).context("Failed to serialize output")?
            );
        }
        OutputFormat::Human | OutputFormat::Lsp => {
            if result.error_count > 0 {
                let msg = format!("Validation failed: {} errors", result.error_count);
                if use_color {
                    use colored::Colorize;
                    println!("{}", msg.red());
                } else {
                    println!("{}", msg);
                }

                for v in &result.violations {
                    let severity = match v.severity {
                        crate::policy::Severity::Error => "ERROR",
                        crate::policy::Severity::Warning => "WARN",
                        crate::policy::Severity::Info => "INFO",
                    };
                    let severity_colored = if use_color {
                        use colored::Colorize;
                        match v.severity {
                            crate::policy::Severity::Error => severity.red().to_string(),
                            crate::policy::Severity::Warning => severity.yellow().to_string(),
                            crate::policy::Severity::Info => severity.blue().to_string(),
                        }
                    } else {
                        severity.to_string()
                    };
                    println!("- [{}] {}: {}", severity_colored, v.policy_name, v.message);
                }
            } else {
                let msg = format!(
                    "Validation succeeded: {} violations total",
                    result.violations.len()
                );
                if use_color {
                    use colored::Colorize;
                    println!("{}", msg.green());
                } else {
                    println!("{}", msg);
                }
            }
        }
    }

    if result.error_count > 0 {
        Err(anyhow::anyhow!("Validation errors detected"))
    } else {
        Ok(())
    }
}
