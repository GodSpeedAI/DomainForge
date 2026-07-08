use crate::parser::{parse_to_graph_with_options, ParseOptions};
use crate::projection::protobuf::{CompatibilityMode, SchemaHistory};
use crate::projection::ProtobufEngine;
use crate::NamespaceRegistry;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

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

    /// Run 'buf lint' on the generated output
    #[arg(long)]
    pub buf_lint: bool,

    /// Run 'buf breaking' check against previous version
    #[arg(long)]
    pub buf_breaking: bool,

    /// Run 'buf generate' to build code artifacts
    #[arg(long)]
    pub buf_generate: bool,

    /// Generate multiple files (one per namespace) instead of a single file
    #[arg(long)]
    pub multi_file: bool,

    /// JSON recipe file (required for ai-learning; optional for the other
    /// learning-loop / AI-recipe formats: ai-llm, ai-graph-ml, cep-eval, baml,
    /// dspy, zenml)
    #[arg(long)]
    pub recipe: Option<PathBuf>,

    /// Split/sampling seed for the AI-recipe formats (ai-llm, ai-graph-ml,
    /// cep-eval, ai-learning, baml, dspy, zenml; overrides the recipe seed)
    #[arg(long)]
    pub seed: Option<u64>,

    /// Comma-separated family subset (ai-llm / cep-eval only)
    #[arg(long)]
    pub families: Option<String>,

    /// AuthorityEnvironmentConfig JSON for resolver-grounded labels (AI-recipe
    /// formats: ai-*; required for baml, dspy, zenml)
    #[arg(long)]
    pub authority_config: Option<PathBuf>,

    /// Fixed RFC3339 created_at timestamp for byte-identical output (all
    /// directory-output projections: ai-*, rdf, bpmn, cmmn, archimate,
    /// otel-semconv, baml, dspy, zenml)
    #[arg(long)]
    pub created_at: Option<String>,

    /// Base IRI the `sea:` prefix expands to (rdf format only; defaults to the
    /// canonical SEA vocabulary namespace)
    #[arg(long)]
    pub base_iri: Option<String>,

    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum ProjectFormat {
    /// Architecture operator: CALM architecture JSON (single-file output)
    Calm,
    /// Semantic-graph operator (legacy): knowledge-graph Turtle/RDF-XML (single-file output; prefer `rdf`)
    Kg,
    /// Interface operator: Protobuf schema + optional gRPC services (single-file output)
    Protobuf,
    /// Interface operator: alias for `protobuf`
    Proto,
    /// Learning-loop operator: LLM JSONL dataset (directory output)
    AiLlm,
    /// Learning-loop operator: graph-ML dataset (directory output)
    AiGraphMl,
    /// Learning-loop operator: CEP evaluation dataset (directory output)
    CepEval,
    /// Learning-loop operator: all enabled AI datasets via recipe (directory output)
    AiLearning,
    /// Formal-verification operator: Lean 4 Lake package (`lake build`-checkable defs/proofs; directory output)
    Lean,
    /// Semantic graph operator: RDF/OWL dataset (Turtle + JSON-LD + OWL, directory output)
    Rdf,
    /// Ordered-process operator: BPMN 2.0 XML (non-executable subset, directory output)
    Bpmn,
    /// Adaptive-work/case operator: CMMN 1.1 XML (directory output)
    Cmmn,
    /// Enterprise-architecture operator: ArchiMate 3.0 Model Exchange File (directory output)
    Archimate,
    /// Runtime-observation operator: OpenTelemetry SemConv registry + attribute constants (directory output)
    OtelSemconv,
    /// AI-representation operator: BAML capability (typed classes/enums, policy-decision function, resolver-seeded tests; directory output)
    Baml,
    /// AI-optimization operator: DSPy program (authority-decision signature, decision-label metric, optimizer config with regression gate; directory output)
    Dspy,
    /// Learning-loop operator: ZenML pipeline package (@step load/train/eval/register wired into a @pipeline DAG, model version keyed to the model hash; directory output)
    Zenml,
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
        ProjectFormat::AiLlm
        | ProjectFormat::AiGraphMl
        | ProjectFormat::CepEval
        | ProjectFormat::AiLearning => {
            run_ai_learning(&args, &graph)?;
        }
        ProjectFormat::Lean => {
            run_lean(&args, &graph)?;
        }
        ProjectFormat::Rdf => {
            run_rdf(&args, &graph)?;
        }
        ProjectFormat::Bpmn => {
            run_bpmn(&args, &graph)?;
        }
        ProjectFormat::Cmmn => {
            run_cmmn(&args, &graph)?;
        }
        ProjectFormat::Archimate => {
            run_archimate(&args, &graph)?;
        }
        ProjectFormat::OtelSemconv => {
            run_otel_semconv(&args, &graph)?;
        }
        ProjectFormat::Baml => {
            run_baml(&args, &graph)?;
        }
        ProjectFormat::Dspy => {
            run_dspy(&args, &graph)?;
        }
        ProjectFormat::Zenml => {
            run_zenml(&args, &graph)?;
        }
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

            if args.multi_file {
                if args.schema_history.is_some() {
                    eprintln!(
                        "Warning: --schema-history is currently ignored in --multi-file mode."
                    );
                }

                if !args.output.exists() {
                    std::fs::create_dir_all(&args.output).with_context(|| {
                        format!(
                            "Failed to create output directory {}",
                            args.output.display()
                        )
                    })?;
                } else if !args.output.is_dir() {
                    return Err(anyhow::anyhow!(
                        "Output path must be a directory for --multi-file projection"
                    ));
                }

                let files = ProtobufEngine::project_multi_file(
                    &graph,
                    &args.package,
                    args.include_governance,
                    args.include_services,
                );

                for (rel_path, proto) in &files {
                    let abs_path = args.output.join(rel_path);
                    crate::projection::protobuf::validate_output_path(&args.output, &abs_path)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    if let Some(parent) = abs_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    let content = proto.to_proto_string();
                    write(&abs_path, content)
                        .with_context(|| format!("Failed to write {}", abs_path.display()))?;
                }

                println!(
                    "Projected to Protobuf (Multi-file): {}",
                    args.output.display()
                );
                println!("  Files: {}", files.len());
                println!("  Base Package: {}", args.package);
            } else {
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
                        println!(
                            "  Compatibility: {} (with {} warnings)",
                            mode,
                            result.violations.len()
                        );
                    } else {
                        println!("  Compatibility: {} (clean)", mode);
                    }
                }

                let proto_string = proto_file.to_proto_string();
                write(&args.output, &proto_string).with_context(|| {
                    format!("Failed to write output to {}", args.output.display())
                })?;
                println!("Projected to Protobuf: {}", args.output.display());
                println!("  Package: {}", args.package);
                println!("  Messages: {}", proto_file.messages.len());
                if !proto_file.services.is_empty() {
                    println!(
                        "  Services: {} ({} methods)",
                        proto_file.services.len(),
                        proto_file
                            .services
                            .iter()
                            .map(|s| s.methods.len())
                            .sum::<usize>()
                    );
                }
            }

            // Buf integration
            if args.buf_lint || args.buf_breaking || args.buf_generate {
                use crate::projection::buf::BufIntegration;
                let buf = BufIntegration::new();
                let output_dir = if args.multi_file {
                    &args.output
                } else {
                    args.output.parent().unwrap_or(Path::new("."))
                };

                if args.buf_lint {
                    print!("  Running buf lint... ");
                    match buf.lint(output_dir) {
                        Ok(result) => {
                            if result.success {
                                println!("clean");
                            } else {
                                println!("issues found:");
                                println!("{}", result.raw_output);
                            }
                        }
                        Err(e) => println!("failed: {}", e),
                    }
                }

                if args.buf_breaking {
                    if let Some(history) = args.schema_history {
                        print!("  Running buf breaking check... ");
                        println!("(checking against {})", history.display());
                    }
                }
            }
        }
    }

    Ok(())
}

fn run_lean(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    if args.recipe.is_some() {
        return Err(anyhow::anyhow!(
            "--recipe is not used by --format lean (the projection is model-driven)"
        ));
    }

    // Directory output, like the ai-* projections.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the lean projection"
        ));
    }

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::lean::emit(
        graph,
        &args.input.display().to_string(),
        args.created_at.clone(),
        &mut sink,
    )
    .map_err(|e| anyhow::anyhow!("lean projection failed: {e}"))?;
    println!(
        "Projected Lean 4 package to {} ({} files); check with `lake build`",
        args.output.display(),
        files.len()
    );
    Ok(())
}

fn run_rdf(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    use crate::projection::rdf::{self, RdfOptions};

    if args.recipe.is_some() {
        return Err(anyhow::anyhow!(
            "--recipe is not used by --format rdf (the projection is model-driven); \
             see docs/rdf-projections.md"
        ));
    }
    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Directory output, identical to the lean projection.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the rdf projection; \
             see docs/rdf-projections.md"
        ));
    }

    let opts = RdfOptions {
        base_iri: args.base_iri.clone(),
    };
    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = rdf::emit(
        graph,
        &args.input.display().to_string(),
        args.created_at.clone(),
        &opts,
        &mut sink,
    )
    .map_err(|e| anyhow::anyhow!("rdf projection failed: {e}; see docs/rdf-projections.md"))?;
    println!(
        "Projected RDF dataset to {} ({} files); validate with `domainforge validate-kg {}`",
        args.output.display(),
        files.len(),
        args.output.join("model.ttl").display()
    );
    Ok(())
}

fn run_bpmn(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    if args.recipe.is_some() {
        return Err(anyhow::anyhow!(
            "--recipe is not used by --format bpmn (the projection is model-driven); \
             see docs/bpmn-projections.md"
        ));
    }
    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Directory output, identical to the lean and rdf projections.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the bpmn projection; \
             see docs/bpmn-projections.md"
        ));
    }

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::bpmn::emit(
        graph,
        &args.input.display().to_string(),
        args.created_at.clone(),
        &mut sink,
    )
    .map_err(|e| anyhow::anyhow!("bpmn projection failed: {e}; see docs/bpmn-projections.md"))?;
    println!(
        "Projected BPMN 2.0 process to {} ({} files); validate with \
         `xmllint --schema schemas/bpmn/BPMN20.xsd {}`",
        args.output.display(),
        files.len(),
        args.output.join("model.bpmn").display()
    );
    Ok(())
}

fn run_cmmn(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    if args.recipe.is_some() {
        return Err(anyhow::anyhow!(
            "--recipe is not used by --format cmmn (the projection is model-driven); \
             see docs/cmmn-projections.md"
        ));
    }
    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Directory output, identical to the lean, rdf, and bpmn projections.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the cmmn projection; \
             see docs/cmmn-projections.md"
        ));
    }

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::cmmn::emit(
        graph,
        &args.input.display().to_string(),
        args.created_at.clone(),
        &mut sink,
    )
    .map_err(|e| anyhow::anyhow!("cmmn projection failed: {e}; see docs/cmmn-projections.md"))?;
    println!(
        "Projected CMMN 1.1 case to {} ({} files); validate with \
         `xmllint --schema schemas/cmmn/CMMN11.xsd {}`",
        args.output.display(),
        files.len(),
        args.output.join("model.cmmn").display()
    );
    Ok(())
}

fn run_archimate(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    if args.recipe.is_some() {
        return Err(anyhow::anyhow!(
            "--recipe is not used by --format archimate (the projection is model-driven); \
             see docs/archimate-projections.md"
        ));
    }
    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Directory output, identical to the lean, rdf, bpmn, and cmmn projections.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the archimate projection; \
             see docs/archimate-projections.md"
        ));
    }

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::archimate::emit(
        graph,
        &args.input.display().to_string(),
        args.created_at.clone(),
        &mut sink,
    )
    .map_err(|e| {
        anyhow::anyhow!("archimate projection failed: {e}; see docs/archimate-projections.md")
    })?;
    println!(
        "Projected ArchiMate model to {} ({} files); validate with \
         `xmllint --schema schemas/archimate/archimate3_Diagram.xsd {}`",
        args.output.display(),
        files.len(),
        args.output.join("model.xml").display()
    );
    Ok(())
}

fn run_otel_semconv(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    if args.recipe.is_some() {
        return Err(anyhow::anyhow!(
            "--recipe is not used by --format otel-semconv (the projection is model-driven); \
             see docs/otel-projections.md"
        ));
    }
    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Directory output, identical to the lean, rdf, bpmn, cmmn, and archimate projections.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the otel-semconv projection; \
             see docs/otel-projections.md"
        ));
    }

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::otel::emit(
        graph,
        &args.input.display().to_string(),
        args.created_at.clone(),
        &mut sink,
    )
    .map_err(|e| {
        anyhow::anyhow!("otel-semconv projection failed: {e}; see docs/otel-projections.md")
    })?;
    println!(
        "Projected OTel SemConv registry to {} ({} files); the registry is \
         {}",
        args.output.display(),
        files.len(),
        args.output.join("registry/telemetry.yaml").display()
    );
    Ok(())
}

fn run_baml(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    use crate::projection::ai_learning::{self, recipe::Recipe};

    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Recipe supplies the `baml` section and (optionally) the authority config
    // path; it is defaulted when absent, exactly like the other ai-* formats.
    let (recipe, recipe_ref, recipe_dir) = match &args.recipe {
        Some(path) => {
            let text = read_to_string(path)
                .with_context(|| format!("Failed to read recipe {}", path.display()))?;
            let recipe = Recipe::from_json(&text).map_err(|e| anyhow::anyhow!(e))?;
            let dir = path.parent().map(Path::to_path_buf);
            (recipe, path.display().to_string(), dir)
        }
        None => (Recipe::default(), "<default>".to_string(), None),
    };

    // Authority environment: CLI flag wins, else the recipe's authority_config
    // resolved relative to the recipe file (identical to run_ai_learning).
    let authority_path: Option<PathBuf> = match (&args.authority_config, &recipe.authority_config) {
        (Some(cli), _) => Some(cli.clone()),
        (None, Some(from_recipe)) => {
            let rel = PathBuf::from(from_recipe);
            Some(match &recipe_dir {
                Some(dir) if rel.is_relative() => dir.join(rel),
                _ => rel,
            })
        }
        (None, None) => None,
    };
    let (authority, authority_ref) = match authority_path {
        Some(path) => {
            let text = read_to_string(&path)
                .with_context(|| format!("Failed to read authority config {}", path.display()))?;
            let config: crate::authority::AuthorityEnvironmentConfig = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("Failed to parse authority config: {e}"))?;
            let mut env = crate::authority::AuthorityEnvironment::new(config)
                .map_err(|e| anyhow::anyhow!("Failed to create authority environment: {e}"))?;
            env.validate()
                .map_err(|e| anyhow::anyhow!("Authority environment validation failed: {e}"))?;
            (Some(env), Some(path.display().to_string()))
        }
        None => {
            return Err(anyhow::anyhow!(
                "--format baml needs an authority environment (the function and its tests are \
                 resolver-grounded): pass --authority-config <FILE> or set 'authority_config' \
                 in the recipe; see docs/baml-projections.md"
            ))
        }
    };

    // Directory output, identical to the other projection families.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the baml projection; \
             see docs/baml-projections.md"
        ));
    }

    let ctx = ai_learning::AiLearningContext::build(
        graph,
        args.input.display().to_string(),
        recipe,
        recipe_ref,
        args.seed,
        args.created_at.clone(),
        authority,
        authority_ref,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::baml::emit(&ctx, &mut sink).map_err(|e| {
        anyhow::anyhow!("baml projection failed: {e}; see docs/baml-projections.md")
    })?;
    println!(
        "Projected BAML capability to {} ({} files); fill in a client in \
         baml_src/clients.baml, then run `baml-cli generate`",
        args.output.display(),
        files.len(),
    );
    Ok(())
}

fn run_dspy(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    use crate::projection::ai_learning::{self, recipe::Recipe};

    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Recipe supplies the `dspy` section and (optionally) the authority config
    // path; it is defaulted when absent, exactly like the baml format.
    let (recipe, recipe_ref, recipe_dir) = match &args.recipe {
        Some(path) => {
            let text = read_to_string(path)
                .with_context(|| format!("Failed to read recipe {}", path.display()))?;
            let recipe = Recipe::from_json(&text).map_err(|e| anyhow::anyhow!(e))?;
            let dir = path.parent().map(Path::to_path_buf);
            (recipe, path.display().to_string(), dir)
        }
        None => (Recipe::default(), "<default>".to_string(), None),
    };

    // Authority environment: CLI flag wins, else the recipe's authority_config
    // resolved relative to the recipe file (identical to run_baml).
    let authority_path: Option<PathBuf> = match (&args.authority_config, &recipe.authority_config) {
        (Some(cli), _) => Some(cli.clone()),
        (None, Some(from_recipe)) => {
            let rel = PathBuf::from(from_recipe);
            Some(match &recipe_dir {
                Some(dir) if rel.is_relative() => dir.join(rel),
                _ => rel,
            })
        }
        (None, None) => None,
    };
    let (authority, authority_ref) = match authority_path {
        Some(path) => {
            let text = read_to_string(&path)
                .with_context(|| format!("Failed to read authority config {}", path.display()))?;
            let config: crate::authority::AuthorityEnvironmentConfig = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("Failed to parse authority config: {e}"))?;
            let mut env = crate::authority::AuthorityEnvironment::new(config)
                .map_err(|e| anyhow::anyhow!("Failed to create authority environment: {e}"))?;
            env.validate()
                .map_err(|e| anyhow::anyhow!("Authority environment validation failed: {e}"))?;
            (Some(env), Some(path.display().to_string()))
        }
        None => {
            return Err(anyhow::anyhow!(
                "--format dspy needs an authority environment (the signature and its examples are \
                 resolver-grounded): pass --authority-config <FILE> or set 'authority_config' \
                 in the recipe; see docs/dspy-projections.md"
            ))
        }
    };

    // Directory output, identical to the other projection families.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the dspy projection; \
             see docs/dspy-projections.md"
        ));
    }

    let ctx = ai_learning::AiLearningContext::build(
        graph,
        args.input.display().to_string(),
        recipe,
        recipe_ref,
        args.seed,
        args.created_at.clone(),
        authority,
        authority_ref,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::dspy::emit(&ctx, &mut sink).map_err(|e| {
        anyhow::anyhow!("dspy projection failed: {e}; see docs/dspy-projections.md")
    })?;
    println!(
        "Projected DSPy program to {} ({} files); generate the ai-learning dataset it \
         references, then run `python optimize.py`",
        args.output.display(),
        files.len(),
    );
    Ok(())
}

fn run_zenml(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    use crate::projection::ai_learning::{self, recipe::Recipe};

    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Recipe supplies the `zenml` section and (optionally) the authority config
    // path; it is defaulted when absent, exactly like the dspy format.
    let (recipe, recipe_ref, recipe_dir) = match &args.recipe {
        Some(path) => {
            let text = read_to_string(path)
                .with_context(|| format!("Failed to read recipe {}", path.display()))?;
            let recipe = Recipe::from_json(&text).map_err(|e| anyhow::anyhow!(e))?;
            let dir = path.parent().map(Path::to_path_buf);
            (recipe, path.display().to_string(), dir)
        }
        None => (Recipe::default(), "<default>".to_string(), None),
    };

    // Authority environment: CLI flag wins, else the recipe's authority_config
    // resolved relative to the recipe file (identical to run_dspy).
    let authority_path: Option<PathBuf> = match (&args.authority_config, &recipe.authority_config) {
        (Some(cli), _) => Some(cli.clone()),
        (None, Some(from_recipe)) => {
            let rel = PathBuf::from(from_recipe);
            Some(match &recipe_dir {
                Some(dir) if rel.is_relative() => dir.join(rel),
                _ => rel,
            })
        }
        (None, None) => None,
    };
    let (authority, authority_ref) = match authority_path {
        Some(path) => {
            let text = read_to_string(&path)
                .with_context(|| format!("Failed to read authority config {}", path.display()))?;
            let config: crate::authority::AuthorityEnvironmentConfig = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("Failed to parse authority config: {e}"))?;
            let mut env = crate::authority::AuthorityEnvironment::new(config)
                .map_err(|e| anyhow::anyhow!("Failed to create authority environment: {e}"))?;
            env.validate()
                .map_err(|e| anyhow::anyhow!("Authority environment validation failed: {e}"))?;
            (Some(env), Some(path.display().to_string()))
        }
        None => {
            return Err(anyhow::anyhow!(
                "--format zenml needs an authority environment (the pipeline and its examples are \
                 resolver-grounded): pass --authority-config <FILE> or set 'authority_config' \
                 in the recipe; see docs/zenml-projections.md"
            ))
        }
    };

    // Directory output, identical to the other projection families.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for the zenml projection; \
             see docs/zenml-projections.md"
        ));
    }

    let ctx = ai_learning::AiLearningContext::build(
        graph,
        args.input.display().to_string(),
        recipe,
        recipe_ref,
        args.seed,
        args.created_at.clone(),
        authority,
        authority_ref,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    let mut sink = crate::projection::sink::ArtifactSink::Dir(&args.output);
    let files = crate::projection::zenml::emit(&ctx, &mut sink).map_err(|e| {
        anyhow::anyhow!("zenml projection failed: {e}; see docs/zenml-projections.md")
    })?;
    println!(
        "Projected ZenML pipeline to {} ({} files); generate the ai-learning dataset it \
         references, then run `python run.py --dry-run`",
        args.output.display(),
        files.len(),
    );
    Ok(())
}

fn run_ai_learning(args: &ProjectArgs, graph: &crate::graph::Graph) -> Result<()> {
    use crate::projection::ai_learning::{self, recipe::Recipe};

    if let Some(ref ts) = args.created_at {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map_err(|e| anyhow::anyhow!("--created-at must be RFC3339: {e}"))?;
    }

    // Load recipe (required for ai-learning; defaulted for single formats).
    let (mut recipe, recipe_ref, recipe_dir) = match &args.recipe {
        Some(path) => {
            let text = read_to_string(path)
                .with_context(|| format!("Failed to read recipe {}", path.display()))?;
            let recipe = Recipe::from_json(&text).map_err(|e| anyhow::anyhow!(e))?;
            let dir = path.parent().map(Path::to_path_buf);
            (recipe, path.display().to_string(), dir)
        }
        None => {
            if matches!(args.format, ProjectFormat::AiLearning) {
                return Err(anyhow::anyhow!(
                    "--format ai-learning requires --recipe <FILE> (JSON recipe)"
                ));
            }
            (Recipe::default(), "<default>".to_string(), None)
        }
    };

    // --families narrows the family list for the format being emitted.
    if let Some(ref families) = args.families {
        let list: Vec<String> = families
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        match args.format {
            ProjectFormat::AiLlm => recipe.projections.llm_dataset.families = list,
            ProjectFormat::CepEval => recipe.projections.cep_eval_dataset.families = list,
            _ => {
                return Err(anyhow::anyhow!(
                    "--families is only meaningful for --format ai-llm or cep-eval"
                ))
            }
        }
        recipe.validate().map_err(|e| anyhow::anyhow!(e))?;
    }

    // Authority environment: CLI flag wins, else the recipe's authority_config
    // resolved relative to the recipe file.
    let authority_path: Option<PathBuf> = match (&args.authority_config, &recipe.authority_config) {
        (Some(cli), _) => Some(cli.clone()),
        (None, Some(from_recipe)) => {
            let rel = PathBuf::from(from_recipe);
            Some(match &recipe_dir {
                Some(dir) if rel.is_relative() => dir.join(rel),
                _ => rel,
            })
        }
        (None, None) => None,
    };
    let (authority, authority_ref) = match authority_path {
        Some(path) => {
            let text = read_to_string(&path)
                .with_context(|| format!("Failed to read authority config {}", path.display()))?;
            let config: crate::authority::AuthorityEnvironmentConfig = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("Failed to parse authority config: {e}"))?;
            let mut env = crate::authority::AuthorityEnvironment::new(config)
                .map_err(|e| anyhow::anyhow!("Failed to create authority environment: {e}"))?;
            env.validate()
                .map_err(|e| anyhow::anyhow!("Authority environment validation failed: {e}"))?;
            (Some(env), Some(path.display().to_string()))
        }
        None => (None, None),
    };

    // Directory output, like protobuf --multi-file.
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).with_context(|| {
            format!(
                "Failed to create output directory {}",
                args.output.display()
            )
        })?;
    } else if !args.output.is_dir() {
        return Err(anyhow::anyhow!(
            "Output path must be a directory for ai-* projections"
        ));
    }

    let ctx = ai_learning::AiLearningContext::build(
        graph,
        args.input.display().to_string(),
        recipe,
        recipe_ref,
        args.seed,
        args.created_at.clone(),
        authority,
        authority_ref,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    type EmitFn = fn(
        &ai_learning::AiLearningContext,
        &mut ai_learning::ArtifactSink,
    ) -> Result<Vec<String>, String>;
    let mut emitted: Vec<(&str, usize)> = Vec::new();
    let mut run = |kind: &'static str, subdir: &str, emit: EmitFn| -> Result<()> {
        let dir = if subdir.is_empty() {
            args.output.clone()
        } else {
            args.output.join(subdir)
        };
        std::fs::create_dir_all(&dir)?;
        let mut sink = ai_learning::ArtifactSink::Dir(&dir);
        let artifacts =
            emit(&ctx, &mut sink).map_err(|e| anyhow::anyhow!("{kind} projection failed: {e}"))?;
        emitted.push((kind, artifacts.len()));
        Ok(())
    };

    match args.format {
        ProjectFormat::AiLlm => run("llm_dataset", "", ai_learning::llm::emit)?,
        ProjectFormat::AiGraphMl => run("graph_ml_dataset", "", ai_learning::graph_ml::emit)?,
        ProjectFormat::CepEval => run("cep_eval_dataset", "", ai_learning::cep_eval::emit)?,
        ProjectFormat::AiLearning => {
            if ctx.recipe.projections.llm_dataset.enabled {
                run("llm_dataset", "llm_dataset", ai_learning::llm::emit)?;
            }
            if ctx.recipe.projections.graph_ml_dataset.enabled {
                run(
                    "graph_ml_dataset",
                    "graph_dataset",
                    ai_learning::graph_ml::emit,
                )?;
            }
            if ctx.recipe.projections.cep_eval_dataset.enabled {
                run("cep_eval_dataset", "cep_eval", ai_learning::cep_eval::emit)?;
            }
        }
        _ => unreachable!("run_ai_learning called for non-ai format"),
    }

    println!("Projected AI learning artifacts: {}", args.output.display());
    for (kind, count) in emitted {
        println!("  {kind}: {count} artifacts");
    }
    Ok(())
}
