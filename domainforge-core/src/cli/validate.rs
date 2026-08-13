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

    /// Also resolve the Application Contract (records, enums, operations)
    /// and report APP001-APP015 diagnostics. File targets only.
    #[arg(long)]
    pub application: bool,

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
        if args.application {
            return Err(anyhow::anyhow!(
                "--application requires a file target, not a directory"
            ));
        }
        validate_directory(&args.target, args.format, use_color, args.show_source)
    } else {
        validate_file(
            &args.target,
            args.format,
            use_color,
            args.show_source,
            args.application,
        )
    }
}

fn validate_file(
    path: &Path,
    format: OutputFormat,
    use_color: bool,
    show_source: bool,
    check_application: bool,
) -> Result<()> {
    let source =
        read_to_string(path).with_context(|| format!("Failed to read file {}", path.display()))?;
    let registry = NamespaceRegistry::discover(path).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(path).map(|ns| ns.to_string()));
    let options = ParseOptions {
        default_namespace: default_namespace.clone(),
        namespace_registry: registry.clone(),
        entry_path: Some(path.to_path_buf()),
        ..Default::default()
    };
    let graph = crate::application::resolve::resolve_filesystem_graph(
        path,
        &source,
        options.namespace_registry.as_ref(),
        options.default_namespace.as_deref(),
    )
    .map_err(|diagnostics| {
        anyhow::anyhow!(
            "Parse failed for {}: {}",
            path.display(),
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    report_validation(graph, format, use_color, show_source, Some(&source))?;

    if check_application {
        report_application_contract(
            path,
            &source,
            registry.as_ref(),
            default_namespace.as_deref(),
            format,
            use_color,
        )?;
    }
    Ok(())
}

/// Resolve the Application Contract for one file target and report its
/// diagnostics through the same `--format`/`--no-color` conventions
/// `report_validation` uses. `domainforge validate` alone never reaches this
/// path: `parse`/`validate`/`project` only resolve the graph, never the
/// Application Contract, so an operation's own errors were previously
/// invisible to the command an ordinary editor runs.
fn report_application_contract(
    path: &Path,
    source: &str,
    registry: Option<&NamespaceRegistry>,
    default_namespace: Option<&str>,
    format: OutputFormat,
    use_color: bool,
) -> Result<()> {
    let (entry_logical_path, source_map) =
        crate::module::resolver::source_map_from_filesystem(
            path,
            source,
            registry,
            default_namespace,
        )
        .map_err(|diagnostics| {
            anyhow::anyhow!(format_application_diagnostics(&diagnostics))
        })?;

    let sources_json = serde_json::to_string(&source_map.0)
        .context("Failed to serialize resolved source map")?;

    match crate::application::resolve::resolve_application_contract(
        &entry_logical_path,
        &sources_json,
    ) {
        Ok(doc) => {
            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&doc)
                            .context("Failed to serialize application contract")?
                    );
                }
                OutputFormat::Human | OutputFormat::Lsp => {
                    let msg = format!(
                        "Application contract valid: {} operation(s), {} record(s), {} enum(s), {} entit{}",
                        doc.contract.operations.len(),
                        doc.contract.records.len(),
                        doc.contract.enums.len(),
                        doc.contract.entities.len(),
                        if doc.contract.entities.len() == 1 { "y" } else { "ies" },
                    );
                    print_line(&msg, use_color, true);
                }
            }
            Ok(())
        }
        Err(diagnostics) => {
            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "error_count": diagnostics.len(),
                            "diagnostics": diagnostics,
                        }))
                        .context("Failed to serialize application diagnostics")?
                    );
                }
                OutputFormat::Human | OutputFormat::Lsp => {
                    let msg = format!(
                        "Application contract invalid: {} error(s)",
                        diagnostics.len()
                    );
                    print_line(&msg, use_color, false);
                    for d in &diagnostics {
                        print_line(&format!("- [{}] {}: {}", d.slug, d.severity, d.message), use_color, false);
                    }
                }
            }
            Err(anyhow::anyhow!("Application contract validation errors detected"))
        }
    }
}

fn format_application_diagnostics(
    diagnostics: &[crate::application::diagnostic::ApplicationDiagnostic],
) -> String {
    diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

fn print_line(msg: &str, use_color: bool, success: bool) {
    if use_color {
        use colored::Colorize;
        if success {
            println!("{}", msg.green());
        } else {
            println!("{}", msg.red());
        }
    } else {
        println!("{}", msg);
    }
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
