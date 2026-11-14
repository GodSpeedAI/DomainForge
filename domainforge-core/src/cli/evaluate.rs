use crate::application::envelope::{resolve_semantic_envelope_with_packs, DomainModelIdentity};
use crate::application::resolve::{resolve_application_contract, resolve_application_graph};
use crate::application::world::{WorldName, WorldRef};
use crate::cli::envelope::registry_content_hash;
use crate::evaluate::{evaluate_model, load_instances, Facts, OUTPUT_SCHEMA};
use crate::module::resolver::source_map_from_filesystem;
use crate::registry::NamespaceRegistry;
use crate::semantic_pack::canonical_json::compute_sha256;
use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::exit;

/// Evaluate every policy of a model against a facts file and report a
/// three-valued result per policy and per bound instance.
///
/// Exit status: 0 whenever evaluation ran, even if policies are false or
/// unknown (the result is in the JSON); 2 on input or usage errors (unreadable
/// or invalid model or facts, unknown entity/field/operation in the facts).
#[derive(Parser)]
pub struct EvaluateArgs {
    /// Entry .sea model file
    #[arg(long)]
    pub model: PathBuf,

    /// Facts file (`domainforge.evaluate-facts.v1` JSON)
    #[arg(long)]
    pub facts: PathBuf,

    #[arg(long, value_enum, default_value_t = EvaluateFormat::Json)]
    pub format: EvaluateFormat,

    /// Path to namespace registry file (defaults to discovery from the model)
    #[arg(long)]
    pub registry: Option<PathBuf>,

    /// Namespace to use when the model has no resolved registry binding
    #[arg(long)]
    pub default_namespace: Option<String>,

    /// Also report the immutable `world_ref` of the model under this world
    /// name, derived exactly as `envelope --world-name` derives it
    #[arg(long = "world-name")]
    pub world_name: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum EvaluateFormat {
    Json,
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {message}");
    exit(2);
}

pub fn run(args: EvaluateArgs) -> Result<()> {
    let EvaluateFormat::Json = args.format;
    let model_bytes = fs::read(&args.model)
        .unwrap_or_else(|e| fail(format!("failed to read {}: {e}", args.model.display())));
    let facts_bytes = fs::read(&args.facts)
        .unwrap_or_else(|e| fail(format!("failed to read {}: {e}", args.facts.display())));
    let source = String::from_utf8(model_bytes.clone())
        .unwrap_or_else(|_| fail("model file is not valid UTF-8"));
    let facts_text =
        std::str::from_utf8(&facts_bytes).unwrap_or_else(|_| fail("facts file is not valid UTF-8"));
    let facts = Facts::from_json(facts_text).unwrap_or_else(|e| fail(e));

    let registry =
        match &args.registry {
            Some(path) => Some(NamespaceRegistry::from_file(path).unwrap_or_else(|e| {
                fail(format!("failed to load registry {}: {e}", path.display()))
            })),
            None => NamespaceRegistry::discover(&args.model).ok().flatten(),
        };
    let default_namespace = args.default_namespace.clone().or_else(|| {
        registry
            .as_ref()
            .and_then(|reg| reg.namespace_for(&args.model).map(str::to_string))
    });
    let render = |diagnostics: &[crate::application::ApplicationDiagnostic]| {
        diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    };
    let (logical, sources) = source_map_from_filesystem(
        &args.model,
        &source,
        registry.as_ref(),
        default_namespace.as_deref(),
    )
    .unwrap_or_else(|d| fail(format!("model did not resolve: {}", render(&d))));
    let sources_json = serde_json::to_string(&sources.0)?;

    let mut graph = resolve_application_graph(&logical, &sources_json)
        .unwrap_or_else(|d| fail(format!("model did not resolve: {}", render(&d))));
    let contract = resolve_application_contract(&logical, &sources_json).unwrap_or_else(|d| {
        fail(format!(
            "application contract did not resolve: {}",
            render(&d)
        ))
    });
    let model_instance_count = graph.entity_instance_count();
    load_instances(&mut graph, &facts).unwrap_or_else(|e| fail(e));
    let policies = evaluate_model(&graph, &contract, &facts).unwrap_or_else(|e| fail(e));

    let world_ref: Value = match args.world_name.as_deref() {
        None => Value::Null,
        Some(name) => {
            let name: WorldName = name
                .parse()
                .unwrap_or_else(|e| fail(format!("--world-name: {e}")));
            let doc = resolve_semantic_envelope_with_packs(&logical, &sources_json, &[])
                .unwrap_or_else(|d| fail(format!("envelope did not resolve: {}", render(&d))));
            let identity = DomainModelIdentity::from_document(
                &doc,
                registry_content_hash(registry.as_ref()).as_deref(),
            );
            let world = WorldRef::from_identity(name, &identity)
                .unwrap_or_else(|e| fail(format!("world_ref: {e}")));
            json!(world.to_string())
        }
    };

    let report = json!({
        "schema": OUTPUT_SCHEMA,
        "compiler": {"name": "domainforge-core", "version": env!("CARGO_PKG_VERSION")},
        "model_sha256": compute_sha256(&model_bytes),
        "source_set_hash": contract.inputs.source_set_hash,
        "facts_sha256": compute_sha256(&facts_bytes),
        "model_instance_count": model_instance_count,
        "facts_instance_count": facts.instances.len(),
        "world_ref": world_ref,
        "policies": policies,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
