use crate::application::canonical::{semantic_pack_set_hash, source_set_hash};
use crate::application::diagnostic::ApplicationDiagnostic;
use crate::application::envelope::{
    build_cep_envelope, invalid_declared_checkpoint_hash, resolve_semantic_envelope_with_packs,
    CepEnvelopeParams,
};
use crate::application::resolve::resolve_application_graph;
use crate::module::resolver::source_map_from_filesystem;
use crate::registry::NamespaceRegistry;
use crate::semantic_pack::canonical_json::{canonical_json, compute_sha256};
use anyhow::{Context, Result};
use chrono::SecondsFormat;
use clap::{Parser, ValueEnum};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use uuid::Uuid;

#[derive(Parser)]
pub struct EnvelopeArgs {
    /// Entry .sea file whose resolved closure is emitted
    pub entry: PathBuf,

    /// What to emit
    #[arg(long, value_enum, default_value_t = EmitMode::Both)]
    pub emit: EmitMode,

    /// Semantic pack inputs as <pack_id>=<content_hash>, repeatable
    #[arg(long = "pack", value_parser = parse_pack)]
    pub packs: Vec<(String, String)>,

    /// Namespace to use when the entry has no resolved registry binding
    #[arg(long)]
    pub default_namespace: Option<String>,

    /// CEP scope object (JSON). Replaces the default derived scope.
    #[arg(long)]
    pub scope: Option<String>,

    /// Inline representations up to this many bytes; larger ones use content_ref
    #[arg(long, default_value_t = 65_536)]
    pub inline_threshold_bytes: u64,

    /// Write output to FILE instead of stdout
    #[arg(long, short = 'o')]
    pub out: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum EmitMode {
    /// Canonical D only (byte-identical across runs)
    Representation,
    /// CEP-0008 semantic snapshot envelope only
    Cep,
    /// Both, as {"representation": D, "cep_envelope": E}
    Both,
    /// CEP `work_request` verification contract (§14.0)
    VerificationContract,
}

fn parse_pack(raw: &str) -> Result<(String, String), String> {
    let (pack_id, content_hash) = raw
        .split_once('=')
        .ok_or_else(|| format!("--pack expects <pack_id>=<content_hash>, got '{raw}'"))?;
    if pack_id.is_empty() || content_hash.is_empty() {
        return Err(format!(
            "--pack expects <pack_id>=<content_hash>, got '{raw}'"
        ));
    }
    Ok((pack_id.to_string(), content_hash.to_string()))
}

pub fn run(args: EnvelopeArgs) -> Result<()> {
    let source = match fs::read_to_string(&args.entry) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: failed to read {}: {error}", args.entry.display());
            exit(2);
        }
    };
    let registry = NamespaceRegistry::discover(&args.entry).ok().flatten();
    let default_namespace = args.default_namespace.clone().or_else(|| {
        registry
            .as_ref()
            .and_then(|reg| reg.namespace_for(&args.entry).map(str::to_string))
    });
    let registry_content_hash = registry_content_hash(registry.as_ref());
    let user_scope = match args.scope.as_deref() {
        None => None,
        Some(text) => match serde_json::from_str::<Value>(text) {
            Ok(value @ Value::Object(_)) => Some(value),
            Ok(_) => {
                eprintln!("error: --scope must be a JSON object");
                exit(2);
            }
            Err(error) => {
                eprintln!("error: --scope is not valid JSON: {error}");
                exit(2);
            }
        },
    };
    let created_at = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
    let envelope_id = Uuid::now_v7().to_string();
    let inline_threshold_bytes = args.inline_threshold_bytes;

    let (entry_logical_path, sources) = match source_map_from_filesystem(
        &args.entry,
        &source,
        registry.as_ref(),
        default_namespace.as_deref(),
    ) {
        Ok(resolved) => resolved,
        Err(diagnostics) => {
            let messages: Vec<String> = diagnostics.iter().map(render_diagnostic).collect();
            let logical = args.entry.display().to_string();
            emit_failed(
                &args,
                FailureContext {
                    entry_logical_path: &logical,
                    sources: &[(logical.clone(), source)],
                    messages: &messages,
                    registry_content_hash: registry_content_hash.as_deref(),
                    user_scope: user_scope.as_ref(),
                    envelope_id: &envelope_id,
                    created_at: &created_at,
                    inline_threshold_bytes,
                },
            );
        }
    };

    let sources_json =
        serde_json::to_string(&sources.0).context("failed to serialize resolved source map")?;

    match resolve_semantic_envelope_with_packs(&entry_logical_path, &sources_json, &args.packs) {
        Ok(doc) => {
            let validation = resolve_application_graph(&entry_logical_path, &sources_json)
                .map(|graph| graph.validate());
            let model_valid = validation
                .as_ref()
                .map(|result| result.error_count == 0)
                .unwrap_or(false);
            let diagnostics: Vec<String> = match &validation {
                Ok(result) => result
                    .violations
                    .iter()
                    .map(|violation| format!("{}: {}", violation.policy_name, violation.message))
                    .collect(),
                Err(application_diagnostics) => application_diagnostics
                    .iter()
                    .map(render_diagnostic)
                    .collect(),
            };
            let resolved_namespaces: Vec<(String, String)> = doc
                .envelope
                .namespace_bindings
                .iter()
                .map(|binding| (binding.logical_id.clone(), binding.namespace.clone()))
                .collect();
            let scope = match &user_scope {
                Some(value) => value.clone(),
                None => json!({
                    "entry_logical_path": entry_logical_path,
                    "semantic_closure_hash": doc.semantic_closure_hash,
                }),
            };
            let envelope = build_cep_envelope(&CepEnvelopeParams {
                doc: Some(&doc),
                model_valid,
                source_set_hash: &doc.inputs.source_set_hash,
                invalid_declared_checkpoint_hash: None,
                diagnostics: &diagnostics,
                scope,
                entry_logical_path: &entry_logical_path,
                inline_threshold_bytes,
                envelope_id: &envelope_id,
                created_at: &created_at,
                registry_content_hash: registry_content_hash.as_deref(),
                resolved_namespaces: &resolved_namespaces,
            });
            let representation = serde_json::to_string_pretty(&doc)
                .context("failed to serialize canonical envelope document")?;
            if args.emit == EmitMode::VerificationContract {
                if !model_valid {
                    eprintln!(
                        "error: verification contract requires a valid declared model; {} error(s)",
                        diagnostics.len()
                    );
                    exit(1);
                }
                let contract =
                    crate::application::verification_contract::build_verification_contract(
                        &doc,
                        &entry_logical_path,
                        &sources_json,
                        &doc.inputs.source_set_hash,
                        &envelope_id,
                        &created_at,
                        registry_content_hash.as_deref(),
                        &resolved_namespaces,
                    )
                    .map_err(|e| anyhow::anyhow!("failed to build verification contract: {e}"))?;
                emit(&args, &EmitPayload::Envelope(contract))?;
                return Ok(());
            }
            let payload = match args.emit {
                EmitMode::Representation => EmitPayload::Representation(representation),
                EmitMode::Cep | EmitMode::VerificationContract => EmitPayload::Envelope(envelope),
                EmitMode::Both => EmitPayload::Both(representation, envelope),
            };
            emit(&args, &payload)?;
            if !model_valid {
                eprintln!(
                    "error: model validation failed: {} error(s); envelope emitted",
                    diagnostics.len()
                );
                exit(1);
            }
            Ok(())
        }
        Err(diagnostics) => {
            let messages: Vec<String> = diagnostics.iter().map(render_diagnostic).collect();
            let sources: Vec<(String, String)> = sources
                .0
                .iter()
                .map(|(id, text)| (id.clone(), text.clone()))
                .collect();
            emit_failed(
                &args,
                FailureContext {
                    entry_logical_path: &entry_logical_path,
                    sources: &sources,
                    messages: &messages,
                    registry_content_hash: registry_content_hash.as_deref(),
                    user_scope: user_scope.as_ref(),
                    envelope_id: &envelope_id,
                    created_at: &created_at,
                    inline_threshold_bytes,
                },
            );
        }
    }
}

enum EmitPayload {
    Representation(String),
    Envelope(Value),
    Both(String, Value),
}

fn emit(args: &EnvelopeArgs, payload: &EmitPayload) -> Result<()> {
    let text = match (args.emit, payload) {
        (EmitMode::Representation, EmitPayload::Representation(document)) => document.clone(),
        (
            EmitMode::Cep | EmitMode::Both | EmitMode::VerificationContract,
            EmitPayload::Envelope(envelope),
        ) => serde_json::to_string_pretty(envelope)?,
        (EmitMode::Cep, EmitPayload::Both(_, envelope)) => serde_json::to_string_pretty(envelope)?,
        (EmitMode::Both, EmitPayload::Both(representation, envelope)) => {
            let representation: Value = serde_json::from_str(representation)?;
            serde_json::to_string_pretty(&json!({
                "representation": representation,
                "cep_envelope": envelope,
            }))?
        }
        (EmitMode::Representation, EmitPayload::Envelope(_) | EmitPayload::Both(_, _)) => {
            eprintln!(
                "error: cannot emit representation: canonical D is unavailable (model validation failed)"
            );
            exit(1);
        }
        (
            EmitMode::Cep | EmitMode::Both | EmitMode::VerificationContract,
            EmitPayload::Representation(_),
        ) => {
            unreachable!()
        }
        (EmitMode::VerificationContract, EmitPayload::Both(_, _)) => unreachable!(),
    };
    match &args.out {
        Some(path) => {
            fs::write(path, text).with_context(|| format!("failed to write {}", path.display()))?
        }
        None => {
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
        }
    }
    Ok(())
}

/// Identity and diagnostic context for a failed-model emission (§14.4).
struct FailureContext<'a> {
    entry_logical_path: &'a str,
    sources: &'a [(String, String)],
    messages: &'a [String],
    registry_content_hash: Option<&'a str>,
    user_scope: Option<&'a Value>,
    envelope_id: &'a str,
    created_at: &'a str,
    inline_threshold_bytes: u64,
}

fn emit_failed(args: &EnvelopeArgs, context: FailureContext<'_>) -> ! {
    let source_set_hash = source_set_hash(
        context
            .sources
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str())),
    )
    .unwrap_or_else(|_| format!("sha256:{}", "0".repeat(64)));
    let pack_set_hash = semantic_pack_set_hash(&args.packs)
        .unwrap_or_else(|_| format!("sha256:{}", "0".repeat(64)));
    let checkpoint = invalid_declared_checkpoint_hash(
        &source_set_hash,
        &pack_set_hash,
        context.entry_logical_path,
        context.registry_content_hash,
        &[],
    );
    let scope = match context.user_scope {
        Some(value) => value.clone(),
        None => json!({
            "entry_logical_path": context.entry_logical_path,
            "invalid_declared_checkpoint_hash": checkpoint,
        }),
    };
    let envelope = build_cep_envelope(&CepEnvelopeParams {
        doc: None,
        model_valid: false,
        source_set_hash: &source_set_hash,
        invalid_declared_checkpoint_hash: Some(&checkpoint),
        diagnostics: context.messages,
        scope,
        entry_logical_path: context.entry_logical_path,
        inline_threshold_bytes: context.inline_threshold_bytes,
        envelope_id: context.envelope_id,
        created_at: context.created_at,
        registry_content_hash: context.registry_content_hash,
        resolved_namespaces: &[],
    });
    if let Err(error) = emit(args, &EmitPayload::Envelope(envelope)) {
        eprintln!("error: failed to emit envelope: {error}");
        exit(2);
    }
    eprintln!(
        "error: model validation failed: {} diagnostic(s); envelope emitted with missing-D omission",
        context.messages.len()
    );
    exit(1);
}

fn render_diagnostic(diagnostic: &ApplicationDiagnostic) -> String {
    format!("{}: {}", diagnostic.code.slug(), diagnostic.message)
}

fn registry_content_hash(registry: Option<&NamespaceRegistry>) -> Option<String> {
    let registry = registry?;
    let mut files: Vec<(String, String)> = match registry.resolve_files() {
        Ok(files) => files
            .iter()
            .map(|binding| {
                (
                    binding.path.to_string_lossy().to_string(),
                    binding.namespace.clone(),
                )
            })
            .collect(),
        Err(_) => return None,
    };
    files.sort();
    let value = json!({
        "schema_version": "domainforge-registry/v1",
        "bindings": files,
    });
    Some(compute_sha256(canonical_json(&value).as_bytes()))
}
