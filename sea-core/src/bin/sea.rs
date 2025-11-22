#[cfg(feature = "shacl")]
use sea_core::kg_import::ImportError;
use sea_core::parser::{parse_to_graph_with_options, ParseOptions};
use sea_core::{import_kg_turtle, Graph, NamespaceRegistry};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use serde_json::to_string_pretty;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "sea", version, about = "SEA DSL CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate(ValidateArgs),
    Registry(RegistryArgs),
    Import(ImportArgs),
}

#[derive(Parser)]
struct ValidateArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    #[arg(long)]
    no_color: bool,

    #[arg(long)]
    no_source: bool,

    #[arg(required = true)]
    target: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
enum OutputFormat {
    Json,
    Human,
    Lsp,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
enum ImportFormat {
    Sbvr,
    Kg,
}

#[derive(Parser)]
struct RegistryArgs {
    #[command(subcommand)]
    command: RegistryCommands,
}

#[derive(Subcommand)]
enum RegistryCommands {
    List {
        #[arg(long)]
        fail_on_ambiguity: bool,
        path: Option<PathBuf>,
    },
    Resolve {
        #[arg(long)]
        fail_on_ambiguity: bool,
        file: PathBuf,
    },
}

#[derive(Parser)]
struct ImportArgs {
    #[arg(long, value_enum)]
    format: ImportFormat, // "sbvr" or "kg"
    
    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate(args) => {
            let use_color = !args.no_color;
            let show_source = !args.no_source;
            if let Err(err) = run_validate(&args.target, args.format, use_color, show_source) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Registry(args) => {
            match args.command {
                RegistryCommands::List { fail_on_ambiguity, path } => {
                    let path_ref = path.as_deref().unwrap_or_else(|| Path::new("."));
                    if let Err(err) = registry_list(path_ref, fail_on_ambiguity) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
                RegistryCommands::Resolve { fail_on_ambiguity, file } => {
                    if let Err(err) = registry_resolve(&file, fail_on_ambiguity) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Import(args) => {
            let source = match read_to_string(&args.file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", args.file.display(), e);
                    std::process::exit(1);
                }
            };

            match args.format {
                ImportFormat::Sbvr => match sea_core::SbvrModel::from_xmi(&source) {
                    Ok(model) => match model.to_graph() {
                        Ok(graph) => {
                            println!(
                                "Imported SBVR to Graph: entities={} resources={} flows={}",
                                graph.entity_count(),
                                graph.resource_count(),
                                graph.flow_count()
                            );
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("Failed to convert SBVR to Graph: {}", e);
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to parse SBVR XMI: {}", e);
                        std::process::exit(1);
                    }
                },
                ImportFormat::Kg => {
                    // If the source looks like XML (starts with <), prefer RDF/XML import path
                    let src_trim = source.trim_start();
                    if src_trim.starts_with('<') {
                        #[cfg(feature = "shacl")]
                        {
                            match sea_core::import_kg_rdfxml(&source) {
                                Ok(graph) => {
                                    println!("Imported KG (RDF/XML) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                    std::process::exit(0);
                                }
                                Err(err_rdf) => match err_rdf {
                                    ImportError::ShaclValidation(msg) => {
                                        eprintln!("{}", msg);
                                        std::process::exit(1);
                                    }
                                    other => {
                                        let err_rdf_msg = other.to_string();
                                        match import_kg_turtle(&source) {
                                            Ok(graph) => {
                                                println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                                std::process::exit(0);
                                            }
                                            Err(err_turtle) => {
                                                eprintln!("Failed to import KG as RDF/XML: {}; as Turtle: {}", err_rdf_msg, err_turtle);
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                },
                            }
                        }
                        #[cfg(not(feature = "shacl"))]
                        {
                            // Without shacl feature we can't parse RDF/XML reliably, so attempt Turtle
                            match import_kg_turtle(&source) {
                                Ok(graph) => {
                                    println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                    std::process::exit(0);
                                }
                                Err(err) => {
                                    eprintln!("Failed to import KG: {}", err);
                                    std::process::exit(1);
                                }
                            }
                        }
                    } else {
                        // Treat as Turtle by default
                        match import_kg_turtle(&source) {
                            Ok(graph) => {
                                println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                std::process::exit(0);
                            }
                            Err(err_turtle) => {
                                // If shacl feature enabled, attempt RDF/XML import as fallback
                                #[cfg(feature = "shacl")]
                                {
                                    match sea_core::import_kg_rdfxml(&source) {
                                        Ok(graph) => {
                                            println!("Imported KG (RDF/XML) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                            std::process::exit(0);
                                        }
                                        Err(err_rdf) => match err_rdf {
                                            ImportError::ShaclValidation(msg) => {
                                                eprintln!("{}", msg);
                                                std::process::exit(1);
                                            }
                                            other => {
                                                let err_rdf_msg = other.to_string();
                                                eprintln!("Failed to import KG as Turtle: {}; and as RDF/XML: {}", err_turtle, err_rdf_msg);
                                                std::process::exit(1);
                                            }
                                        },
                                    }
                                }
                                #[cfg(not(feature = "shacl"))]
                                {
                                    eprintln!("Failed to import KG (Turtle): {}. To try RDF/XML, enable feature 'shacl' in the build.", err_turtle);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn run_validate(target: &Path, format: OutputFormat, use_color: bool, show_source: bool) -> Result<(), String> {
    if target.is_dir() {
        validate_directory(target, format, use_color, show_source)
    } else {
        validate_file(target, format, use_color, show_source)
    }
}

fn validate_file(path: &Path, format: OutputFormat, use_color: bool, show_source: bool) -> Result<(), String> {
    let source = read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
    let registry = NamespaceRegistry::discover(path).map_err(|e| e.to_string())?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(path).map(|ns| ns.to_string()));
    let options = ParseOptions { default_namespace };
    let graph = parse_to_graph_with_options(&source, &options)
        .map_err(|e| format!("Parse failed for {}: {}", path.display(), e))?;
    report_validation(graph, Some(&source), format, use_color, show_source)
}

fn validate_directory(path: &Path, format: OutputFormat, use_color: bool, show_source: bool) -> Result<(), String> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| format!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| {
            format!(
                "No .sea-registry.toml found for {}. Run inside a workspace with a registry file.",
                path.display()
            )
        })?;

    let files = registry
        .resolve_files()
        .map_err(|e| format!("Failed to expand registry: {}", e))?;

    if files.is_empty() {
        return Err(format!(
            "Registry at '{}' did not match any .sea files",
            registry.root().display()
        ));
    }

    let mut graph = Graph::new();
    for binding in files {
        let source = read_to_string(&binding.path)
            .map_err(|e| format!("Failed to read {}: {}", binding.path.display(), e))?;
        let options = ParseOptions {
            default_namespace: Some(binding.namespace.clone()),
        };
        let file_graph = parse_to_graph_with_options(&source, &options)
            .map_err(|e| format!("Parse failed for {}: {}", binding.path.display(), e))?;
        graph
            .extend(file_graph)
            .map_err(|e| format!("Failed to merge {}: {}", binding.path.display(), e))?;
    }

    report_validation(graph, None, format, use_color, show_source)
}

fn report_validation(
    graph: Graph,
    _source: Option<&str>,
    format: OutputFormat,
    use_color: bool,
    _show_source: bool,
) -> Result<(), String> {
    let result = graph.validate();
    
    // For now, we keep the existing policy violation reporting
    // In the future, we could convert policy violations to ValidationErrors
    // and use the formatters
    match format {
        OutputFormat::Json => {
            // JSON output for policy violations
            let json_output = serde_json::json!({
                "error_count": result.error_count,
                "violations": result.violations.iter().map(|v| {
                    serde_json::json!({
                        "severity": match v.severity {
                            sea_core::policy::Severity::Error => "error",
                            sea_core::policy::Severity::Warning => "warning",
                            sea_core::policy::Severity::Info => "info",
                        },
                        "policy_name": v.policy_name,
                        "message": v.message,
                    })
                }).collect::<Vec<_>>(),
            });
            match to_string_pretty(&json_output) {
                Ok(s) => println!("{}", s),
                Err(e) => return Err(format!("Failed to serialize output: {}", e)),
            }
        }
        OutputFormat::Human | OutputFormat::Lsp => {
            // Human-readable output (LSP not applicable for policy violations)
            if result.error_count > 0 {
                let msg = format!("Validation failed: {} errors", result.error_count);
                if use_color {
                    println!("\x1b[31m{}\x1b[0m", msg);
                } else {
                    println!("{}", msg);
                }

                for v in &result.violations {
                    let severity = match v.severity {
                        sea_core::policy::Severity::Error => "ERROR",
                        sea_core::policy::Severity::Warning => "WARN",
                        sea_core::policy::Severity::Info => "INFO",
                    };
                    let severity_colored = if use_color {
                        match v.severity {
                            sea_core::policy::Severity::Error => format!("\x1b[31m{}\x1b[0m", severity),
                            sea_core::policy::Severity::Warning => format!("\x1b[33m{}\x1b[0m", severity),
                            sea_core::policy::Severity::Info => format!("\x1b[34m{}\x1b[0m", severity),
                        }
                    } else {
                        severity.to_string()
                    };
                    println!(
                        "- [{}] {}: {}",
                        severity_colored,
                        v.policy_name,
                        v.message
                    );
                }
            } else {
                let msg = format!(
                    "Validation succeeded: {} violations total",
                    result.violations.len()
                );
                if use_color {
                    println!("\x1b[32m{}\x1b[0m", msg);
                } else {
                    println!("{}", msg);
                }
            }
        }
    }
    
    if result.error_count > 0 {
        Err("Validation errors detected".into())
    } else {
        Ok(())
    }
}

fn registry_list(path: &Path, fail_on_ambiguity: bool) -> Result<(), String> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| format!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| format!("No .sea-registry.toml found for {}", path.display()))?;

    let files = registry
        .resolve_files_with_options(fail_on_ambiguity)
        .map_err(|e| format!("Failed to expand registry: {}", e))?;
    for binding in files {
        println!("{} => {}", binding.path.display(), binding.namespace);
    }
    Ok(())
}

fn registry_resolve(path: &Path, fail_on_ambiguity: bool) -> Result<(), String> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| format!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| format!("No .sea-registry.toml found for {}", path.display()))?;

    let ns = match registry.namespace_for_with_options(path, fail_on_ambiguity) {
        Ok(s) => s.to_string(),
        Err(e) => return Err(e.to_string()),
    };
    println!("{} => {}", path.display(), ns);
    Ok(())
}
