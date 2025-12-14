use crate::parser::{parse_to_graph_with_options, ParseOptions};
use crate::projection::protobuf::{CompatibilityMode, SchemaHistory};
use crate::projection::ProtobufEngine;
use crate::NamespaceRegistry;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs::{read_to_string, write};
use std::path::PathBuf;

#[derive(Parser)]
pub struct ProjectArgs {
    #[arg(long, value_enum)]
    pub format: ProjectFormat,

    /// Optional namespace filter (project only entities from this namespace)
    #[arg(long)]
    pub namespace: Option<String>,

    /// Protobuf package name (for protobuf format)
    #[arg(long, default_value = "sea.generated")]
    pub package: String,

    /// Include governance messages (for protobuf format)
    #[arg(long)]
    pub include_governance: bool,

    /// Generate gRPC service definitions from Flow patterns (protobuf only)
    #[arg(long)]
    pub include_services: bool,

    /// Schema compatibility mode: additive, backward, or breaking (protobuf only)
    #[arg(long, value_enum, default_value = "backward")]
    pub compatibility: CliCompatibilityMode,

    /// Directory to store schema history for compatibility checking (protobuf only)
    #[arg(long)]
    pub schema_history: Option<PathBuf>,

    /// Automatically apply fixes (add reserved fields) for backward compatibility
    #[arg(long)]
    pub apply_fixes: bool,

    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ProjectFormat {
    Calm,
    Kg,
    Protobuf,
    Proto, // alias for protobuf
}

#[derive(ValueEnum, Clone, Debug, Copy, Default)]
pub enum CliCompatibilityMode {
    /// Only additions allowed - strictest mode for public APIs
    Additive,
    /// Removals become reserved fields - default for internal APIs
    #[default]
    Backward,
    /// All changes allowed - for breaking releases
    Breaking,
}

impl From<CliCompatibilityMode> for CompatibilityMode {
    fn from(mode: CliCompatibilityMode) -> Self {
        match mode {
            CliCompatibilityMode::Additive => CompatibilityMode::Additive,
            CliCompatibilityMode::Backward => CompatibilityMode::Backward,
            CliCompatibilityMode::Breaking => CompatibilityMode::Breaking,
        }
    }
}

pub fn run(args: ProjectArgs) -> Result<()> {
    let source = read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file {}", args.input.display()))?;

    // Parse input
    let registry =
        NamespaceRegistry::discover(&args.input).context("discovering namespace registry")?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(&args.input).map(|ns| ns.to_string()));
    let options = ParseOptions {
        default_namespace: default_namespace.clone(),
        namespace_registry: registry.clone(),
        entry_path: Some(args.input.clone()),
        ..Default::default()
    };
    let graph = parse_to_graph_with_options(&source, &options)
        .map_err(|e| anyhow::anyhow!("Parse failed for {}: {}", args.input.display(), e))?;

    match args.format {
        ProjectFormat::Calm => {
            let value = crate::calm::export(&graph)
                .map_err(|e| anyhow::anyhow!("Failed to export to CALM: {}", e))?;
            let json =
                serde_json::to_string_pretty(&value).context("Failed to serialize CALM JSON")?;
            write(&args.output, json)
                .with_context(|| format!("Failed to write output to {}", args.output.display()))?;
            println!("Projected to CALM: {}", args.output.display());
        }
        ProjectFormat::Kg => {
            let kg = crate::KnowledgeGraph::from_graph(&graph)
                .map_err(|e| anyhow::anyhow!("Failed to convert to Knowledge Graph: {}", e))?;

            let output_str = if args
                .output
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("xml") || ext.eq_ignore_ascii_case("rdf")
                }) {
                kg.to_rdf_xml()
            } else {
                kg.to_turtle()
            };

            write(&args.output, output_str)
                .with_context(|| format!("Failed to write output to {}", args.output.display()))?;
            println!("Projected to KG: {}", args.output.display());
        }
        ProjectFormat::Protobuf | ProjectFormat::Proto => {
            let namespace_filter = args.namespace.as_deref().unwrap_or("");
            let projection_name = args
                .input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("projection");

            let mut proto_file = ProtobufEngine::project_with_full_options(
                &graph,
                namespace_filter,
                &args.package,
                projection_name,
                args.include_governance,
                args.include_services,
            );

            // Handle compatibility checking if schema history is provided
            if let Some(ref history_dir) = args.schema_history {
                let history = SchemaHistory::new(history_dir);
                let mode: CompatibilityMode = args.compatibility.into();

                let result = history
                    .check_and_update(&mut proto_file, mode, args.apply_fixes)
                    .map_err(|e| anyhow::anyhow!("Compatibility check failed: {}", e))?;

                // Print compatibility report
                if result.has_violations() {
                    eprintln!("\n{}", result.to_report());
                }

                // Fail if not compatible (unless in breaking mode)
                if !result.is_compatible {
                    return Err(anyhow::anyhow!(
                        "Schema evolution is not compatible in {} mode. Use --compatibility breaking to force, or --apply-fixes to auto-fix.",
                        mode
                    ));
                }

                if result.has_violations() {
                    println!("  Compatibility: {} (with {} warnings)", mode, result.violations.len());
                } else {
                    println!("  Compatibility: {} (clean)", mode);
                }
            }

            let proto_string = proto_file.to_proto_string();
            write(&args.output, &proto_string)
                .with_context(|| format!("Failed to write output to {}", args.output.display()))?;
            println!("Projected to Protobuf: {}", args.output.display());
            println!("  Package: {}", args.package);
            println!("  Messages: {}", proto_file.messages.len());
            if !proto_file.services.is_empty() {
                println!("  Services: {} ({} methods)", 
                    proto_file.services.len(),
                    proto_file.services.iter().map(|s| s.methods.len()).sum::<usize>()
                );
            }
        }
    }

    Ok(())
}
