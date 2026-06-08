use crate::semantic_pack::diagnostics::{DeprecatedPolicy, UnknownConceptPolicy};
use crate::semantic_pack::diff::DiffClassification;
use crate::semantic_pack::schema::{ConceptDefinition, ReviewRecord};
use crate::semantic_pack::{
    build_semantic_pack, compute_pack_content_hash, derive_signer_id, diff_packs, sign_pack,
    validate_graph_with_pack, validate_semantic_pack, verify_pack_signature, ApprovalState,
    ConceptDef, ConceptKind, ConceptStatus, DiagnosticSeverity, PackBuildInput, SemanticDiagnostic,
    SemanticPack, SignatureState, ValidationMode, ValidationOptions,
};
use crate::{parse_to_graph, Graph};
use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
const EXIT_PASSED: i32 = 0;
const EXIT_SEMANTIC_FAILED: i32 = 1;
const EXIT_PARSE_ERROR: i32 = 2;
const EXIT_PACK_UNAVAILABLE: i32 = 3;
const EXIT_SIGNATURE_FAILURE: i32 = 4;
#[allow(dead_code)]
const EXIT_CONFIG_ERROR: i32 = 5;
#[allow(dead_code)]
const EXIT_INTERNAL_ERROR: i32 = 6;

#[derive(Debug, Args)]
pub struct PackArgs {
    #[command(subcommand)]
    pub command: PackCommands,
}

#[derive(Debug, Subcommand)]
pub enum PackCommands {
    #[command(name = "build")]
    Build(BuildArgs),
    #[command(name = "validate")]
    Validate(PackValidateArgs),
    #[command(name = "inspect")]
    Inspect(InspectArgs),
    #[command(name = "diff")]
    Diff(DiffArgs),
    #[command(name = "sign")]
    Sign(SignArgs),
    #[command(name = "verify")]
    Verify(VerifyArgs),
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long, required = true, num_args = 1..)]
    pub source: Vec<String>,
    #[arg(long)]
    pub org: String,
    #[arg(long)]
    pub domain: String,
    #[arg(long)]
    pub version: String,
    #[arg(long)]
    pub meaning_version: String,
    #[arg(long, value_enum, default_value_t = ApprovalChoice::Candidate)]
    pub approval: ApprovalChoice,
    #[arg(long)]
    pub review: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub previous_pack: Option<PathBuf>,
    #[arg(long)]
    pub allow_first_approved_version: bool,
    #[arg(long, value_enum, default_value_t = PackOutputFormat::Human)]
    pub format: PackOutputFormat,
}

#[derive(Debug, Args)]
pub struct PackValidateArgs {
    #[arg(long)]
    pub pack: PathBuf,
    #[arg(long, value_enum, default_value_t = ValidationModeChoice::Warn)]
    pub mode: ValidationModeChoice,
    #[arg(long, value_enum, default_value_t = DeprecatedPolicyChoice::Warn)]
    pub deprecated_policy: DeprecatedPolicyChoice,
    #[arg(long)]
    pub require_signature: bool,
    #[arg(long)]
    pub expected_hash: Option<String>,
    #[arg(long, value_enum, default_value_t = PackOutputFormat::Human)]
    pub format: PackOutputFormat,
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct InspectArgs {
    #[arg(long, value_enum, default_value_t = PackOutputFormat::Human)]
    pub format: PackOutputFormat,
    pub pack: PathBuf,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(long, value_enum, default_value_t = PackOutputFormat::Human)]
    pub format: PackOutputFormat,
    pub old: PathBuf,
    pub new: PathBuf,
}

#[derive(Debug, Args)]
pub struct SignArgs {
    pub pack: PathBuf,
    #[arg(long)]
    pub key: PathBuf,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    pub pack: PathBuf,
    #[arg(long)]
    pub key: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ApprovalChoice {
    Candidate,
    Approved,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ValidationModeChoice {
    Off,
    Warn,
    Strict,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum DeprecatedPolicyChoice {
    Allow,
    Warn,
    ErrorInStrict,
    ErrorAlways,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum PackOutputFormat {
    Human,
    Json,
    Jsonl,
}

pub fn run(args: PackArgs) -> Result<()> {
    match args.command {
        PackCommands::Build(a) => run_build(a),
        PackCommands::Validate(a) => run_validate(a),
        PackCommands::Inspect(a) => run_inspect(a),
        PackCommands::Diff(a) => run_diff(a),
        PackCommands::Sign(a) => run_sign(a),
        PackCommands::Verify(a) => run_verify(a),
    }
}

#[allow(dead_code)]
fn exit_with_code(code: i32, message: &str) -> Result<()> {
    if code != 0 {
        Err(anyhow::anyhow!("{}", message))
    } else {
        println!("{}", message);
        Ok(())
    }
}

fn load_pack_json(path: &Path) -> Result<SemanticPack> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read pack file: {}", path.display()))?;
    serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse pack JSON from: {}", path.display()))
}

fn collect_sea_files(globs: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for pattern in globs {
        let parent = std::path::Path::new(pattern)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let file_pattern = std::path::Path::new(pattern)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("**/*.sea");

        let is_glob = pattern.contains('*') || pattern.contains('?') || pattern.contains('[');

        if is_glob {
            for entry in globwalk::glob(parent.join(file_pattern).to_string_lossy().as_ref())
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            {
                let entry =
                    entry.with_context(|| format!("Error reading glob entry for: {}", pattern))?;
                if entry.path().extension().and_then(|e| e.to_str()) == Some("sea") {
                    files.push(entry.path().to_path_buf());
                }
            }
        } else {
            let path = PathBuf::from(pattern);
            if path.exists() {
                files.push(path);
            } else {
                anyhow::bail!("Source file not found: {}", pattern);
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn parse_sea_files(files: &[PathBuf]) -> Result<Graph> {
    let mut combined = Graph::new();
    for path in files {
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        let graph = parse_to_graph(&source)
            .map_err(|e| anyhow::anyhow!("Parse error in {}: {}", path.display(), e))?;
        combined
            .extend(graph)
            .map_err(|e| anyhow::anyhow!("Merge error from {}: {}", path.display(), e))?;
    }
    Ok(combined)
}

fn graph_to_concepts(graph: &Graph) -> Vec<ConceptDef> {
    let mut concepts = Vec::new();

    for entity in graph.all_entities() {
        concepts.push(make_concept_def(
            entity.id().to_string(),
            entity.name().to_string(),
            ConceptKind::Entity,
        ));
    }
    for resource in graph.all_resources() {
        concepts.push(make_concept_def(
            resource.id().to_string(),
            resource.name().to_string(),
            ConceptKind::Resource,
        ));
    }
    for role in graph.all_roles() {
        concepts.push(make_concept_def(
            role.id().to_string(),
            role.name().to_string(),
            ConceptKind::Role,
        ));
    }
    for flow in graph.all_flows() {
        concepts.push(make_concept_def(
            flow.id().to_string(),
            flow.id().to_string(),
            ConceptKind::Flow,
        ));
    }
    for policy in graph.all_policies() {
        concepts.push(make_concept_def(
            policy.id.to_string(),
            policy.name.to_string(),
            ConceptKind::Policy,
        ));
    }
    for metric in graph.all_metrics() {
        concepts.push(make_concept_def(
            metric.id().to_string(),
            metric.name.to_string(),
            ConceptKind::Metric,
        ));
    }

    concepts
}

fn make_concept_def(id: String, canonical_name: String, kind: ConceptKind) -> ConceptDef {
    ConceptDef {
        id,
        canonical_name,
        kind,
        status: ConceptStatus::Active,
        definition: ConceptDefinition {
            text: String::new(),
            definition_hash: String::new(),
            decision_ref: String::new(),
        },
        owner: String::new(),
        source_refs: vec![],
        examples: vec![],
        counterexamples: vec![],
        allowed_predicates: vec![],
        valid_contexts: vec![],
    }
}

fn load_review_records(path: &Path) -> Result<Vec<ReviewRecord>> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read review file: {}", path.display()))?;
    let mut records = Vec::new();
    for (line_num, line) in data.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record: ReviewRecord = serde_json::from_str(trimmed).with_context(|| {
            format!(
                "Failed to parse review JSONL at line {} in {}",
                line_num + 1,
                path.display()
            )
        })?;
        records.push(record);
    }
    Ok(records)
}

fn compute_graph_hash(graph: &Graph) -> String {
    use sha2::{Digest, Sha256};
    let json = serde_json::to_string(graph).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

fn run_build(args: BuildArgs) -> Result<()> {
    let files = collect_sea_files(&args.source)?;
    if files.is_empty() {
        anyhow::bail!("No .sea files matched the provided source patterns");
    }

    let graph = match parse_sea_files(&files) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            std::process::exit(EXIT_PARSE_ERROR);
        }
    };

    let concepts = graph_to_concepts(&graph);
    let source_graph_hash = compute_graph_hash(&graph);

    let review_records = match args.review {
        Some(ref p) => load_review_records(p)?,
        None => vec![],
    };

    let previous_pack = match args.previous_pack {
        Some(ref p) => Some(load_pack_json(p)?),
        None => None,
    };

    let approval = match args.approval {
        ApprovalChoice::Candidate => ApprovalState::Candidate,
        ApprovalChoice::Approved => ApprovalState::Approved,
    };

    let input = PackBuildInput {
        org_id: args.org,
        domain_id: args.domain,
        pack_version: args.version,
        meaning_version: args.meaning_version,
        approval,
        concepts,
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases: vec![],
        mapping_rules: vec![],
        review_records,
        previous_pack,
        allow_first_approved_version: args.allow_first_approved_version,
        source_graph_hash,
    };

    let output = match build_semantic_pack(input) {
        Ok(o) => o,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics, args.format);
            std::process::exit(EXIT_SEMANTIC_FAILED);
        }
    };

    if !output.pre_pack_diagnostics.is_empty() {
        print_diagnostics(&output.pre_pack_diagnostics, args.format);
    }
    if !output.build_warnings.is_empty() {
        print_diagnostics(&output.build_warnings, args.format);
    }

    let pack_json =
        serde_json::to_string_pretty(&output.pack).context("Failed to serialize pack")?;

    match args.out {
        Some(ref p) => {
            fs::write(p, &pack_json)
                .with_context(|| format!("Failed to write pack to: {}", p.display()))?;
        }
        None => println!("{}", pack_json),
    }

    match args.format {
        PackOutputFormat::Human => {
            eprintln!(
                "Built pack: {} (hash: {})",
                output.pack.pack_id, output.pack_content_hash
            );
            eprintln!(
                "  {} concepts, meaning_fingerprint: {}",
                output.pack.concepts.len(),
                output.meaning_fingerprint
            );
        }
        PackOutputFormat::Json => {
            let meta = serde_json::json!({
                "pack_id": output.pack.pack_id,
                "content_hash": output.pack_content_hash,
                "meaning_fingerprint": output.meaning_fingerprint,
                "concept_count": output.pack.concepts.len(),
            });
            eprintln!("{}", serde_json::to_string_pretty(&meta).unwrap());
        }
        PackOutputFormat::Jsonl => {
            eprintln!(
                "{{\"pack_id\":\"{}\",\"content_hash\":\"{}\",\"concept_count\":{}}}",
                output.pack.pack_id,
                output.pack_content_hash,
                output.pack.concepts.len()
            );
        }
    }

    Ok(())
}

fn run_validate(args: PackValidateArgs) -> Result<()> {
    let pack = load_pack_json(&args.pack)?;

    let content_hash = compute_pack_content_hash(&pack);
    if let Some(ref expected) = args.expected_hash {
        if *expected != content_hash {
            let msg = format!(
                "Pack content hash mismatch: expected '{}', got '{}'",
                expected, content_hash
            );
            match args.format {
                PackOutputFormat::Json => {
                    let out = serde_json::json!({
                        "status": "hash_mismatch",
                        "expected": expected,
                        "actual": content_hash,
                    });
                    println!("{}", serde_json::to_string_pretty(&out).unwrap());
                }
                _ => eprintln!("{}", msg.red()),
            }
            std::process::exit(EXIT_SIGNATURE_FAILURE);
        }
    }

    if args.require_signature && pack.trust.signature_state != SignatureState::Signed {
        let msg = format!(
            "Pack '{}' is unsigned but --require-signature was set",
            pack.pack_id
        );
        match args.format {
            PackOutputFormat::Json => {
                let out = serde_json::json!({
                    "status": "unsigned",
                    "pack_id": pack.pack_id,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            }
            _ => eprintln!("{}", msg.red()),
        }
        std::process::exit(EXIT_PACK_UNAVAILABLE);
    }

    let validation_mode = match args.mode {
        ValidationModeChoice::Off => ValidationMode::Off,
        ValidationModeChoice::Warn => ValidationMode::Warn,
        ValidationModeChoice::Strict => ValidationMode::Strict,
    };
    let deprecated_policy = match args.deprecated_policy {
        DeprecatedPolicyChoice::Allow => DeprecatedPolicy::Allow,
        DeprecatedPolicyChoice::Warn => DeprecatedPolicy::Warn,
        DeprecatedPolicyChoice::ErrorInStrict => DeprecatedPolicy::ErrorInStrict,
        DeprecatedPolicyChoice::ErrorAlways => DeprecatedPolicy::ErrorAlways,
    };

    let options = ValidationOptions {
        mode: validation_mode,
        unknown_concept_policy: UnknownConceptPolicy::Warning,
        deprecated_policy,
        require_signed_pack: args.require_signature,
        allow_unsigned_test_fixtures: false,
    };

    let pack_diagnostics = validate_semantic_pack(&pack)
        .map_err(|e| anyhow::anyhow!("Pack validation error: {:?}", e))?;

    let mut all_diagnostics = pack_diagnostics;

    let mut combined_graph = Graph::new();
    for input_path in &args.inputs {
        let source = fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read: {}", input_path.display()))?;
        let graph = parse_to_graph(&source)
            .map_err(|e| anyhow::anyhow!("Parse error in {}: {}", input_path.display(), e))?;
        combined_graph
            .extend(graph)
            .map_err(|e| anyhow::anyhow!("Merge error: {}", e))?;
    }

    if !combined_graph.is_empty() {
        let source_uri = args
            .inputs
            .first()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let result = validate_graph_with_pack(&pack, &source_uri, &options);
        all_diagnostics.extend(result.diagnostics);
    }

    let has_errors = all_diagnostics
        .iter()
        .any(|d| d.severity == DiagnosticSeverity::Error);

    match args.format {
        PackOutputFormat::Json => {
            let out = serde_json::json!({
                "pack_id": pack.pack_id,
                "content_hash": content_hash,
                "diagnostic_count": all_diagnostics.len(),
                "has_errors": has_errors,
                "diagnostics": all_diagnostics.iter().map(diagnostics_to_json).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        PackOutputFormat::Jsonl => {
            for d in &all_diagnostics {
                let json = serde_json::to_string(&diagnostics_to_json(d)).unwrap();
                println!("{}", json);
            }
        }
        PackOutputFormat::Human => {
            if all_diagnostics.is_empty() {
                println!(
                    "Pack validation passed for '{}' ({})",
                    pack.pack_id, content_hash
                );
                if !args.inputs.is_empty() {
                    println!(
                        "  {} input file(s) validated against pack",
                        args.inputs.len()
                    );
                }
            } else {
                let label = if has_errors {
                    "FAILED".red().to_string()
                } else {
                    "WARNINGS".yellow().to_string()
                };
                println!(
                    "Pack validation {} for '{}' ({}): {} diagnostic(s)",
                    label,
                    pack.pack_id,
                    content_hash,
                    all_diagnostics.len()
                );
                for d in &all_diagnostics {
                    let sev = format_severity(d.severity);
                    println!(
                        "  [{}] {} ({}): {}",
                        sev,
                        d.code.as_str(),
                        d.source_ref.uri,
                        d.message
                    );
                }
            }
        }
    }

    if has_errors {
        std::process::exit(EXIT_SEMANTIC_FAILED);
    }
    Ok(())
}

fn run_inspect(args: InspectArgs) -> Result<()> {
    let pack = load_pack_json(&args.pack)?;
    let content_hash = compute_pack_content_hash(&pack);

    match args.format {
        PackOutputFormat::Json => {
            let out = serde_json::json!({
                "schema_version": pack.schema_version,
                "pack_id": pack.pack_id,
                "org_id": pack.org_id,
                "domain_id": pack.domain_id,
                "pack_version": pack.pack_version,
                "meaning_version": pack.meaning_version,
                "meaning_fingerprint": pack.meaning_fingerprint,
                "content_hash": content_hash,
                "source_graph_hash": pack.source_graph_hash,
                "created_at": pack.created_at,
                "generator": pack.generator,
                "trust": pack.trust,
                "counts": {
                    "concepts": pack.concepts.len(),
                    "relations": pack.relations.len(),
                    "metrics": pack.metrics.len(),
                    "dimensions": pack.dimensions.len(),
                    "units": pack.units.len(),
                    "aliases": pack.aliases.len(),
                    "mapping_rules": pack.mapping_rules.len(),
                },
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        PackOutputFormat::Jsonl => {
            let row = serde_json::json!({
                "pack_id": pack.pack_id,
                "content_hash": content_hash,
                "concepts": pack.concepts.len(),
                "relations": pack.relations.len(),
                "metrics": pack.metrics.len(),
                "dimensions": pack.dimensions.len(),
                "units": pack.units.len(),
                "aliases": pack.aliases.len(),
                "mapping_rules": pack.mapping_rules.len(),
            });
            println!("{}", serde_json::to_string(&row).unwrap());
        }
        PackOutputFormat::Human => {
            println!("{}", "Semantic Pack".bold());
            println!("{:<25} {}", "Pack ID:".dimmed(), pack.pack_id);
            println!("{:<25} {}", "Schema Version:".dimmed(), pack.schema_version);
            println!(
                "{:<25} {}",
                "Org / Domain:".dimmed(),
                format!("{}/{}", pack.org_id, pack.domain_id)
            );
            println!("{:<25} {}", "Pack Version:".dimmed(), pack.pack_version);
            println!(
                "{:<25} {}",
                "Meaning Version:".dimmed(),
                pack.meaning_version
            );
            println!(
                "{:<25} {}",
                "Meaning Fingerprint:".dimmed(),
                pack.meaning_fingerprint
            );
            println!("{:<25} {}", "Content Hash:".dimmed(), content_hash);
            println!(
                "{:<25} {}",
                "Source Graph Hash:".dimmed(),
                pack.source_graph_hash
            );
            println!("{:<25} {}", "Created At:".dimmed(), pack.created_at);
            println!(
                "{:<25} {} {}",
                "Generator:".dimmed(),
                pack.generator.name,
                pack.generator.version
            );
            println!(
                "{:<25} {:?} / {:?}",
                "Trust:".dimmed(),
                pack.trust.approval_state,
                pack.trust.signature_state,
            );
            println!();
            println!("{}", "Counts".bold());
            println!("{:<25} {}", "Concepts:".dimmed(), pack.concepts.len());
            println!("{:<25} {}", "Relations:".dimmed(), pack.relations.len());
            println!("{:<25} {}", "Metrics:".dimmed(), pack.metrics.len());
            println!("{:<25} {}", "Dimensions:".dimmed(), pack.dimensions.len());
            println!("{:<25} {}", "Units:".dimmed(), pack.units.len());
            println!("{:<25} {}", "Aliases:".dimmed(), pack.aliases.len());
            println!(
                "{:<25} {}",
                "Mapping Rules:".dimmed(),
                pack.mapping_rules.len()
            );

            if !pack.concepts.is_empty() {
                println!();
                println!("{}", "Concepts".bold());
                for c in &pack.concepts {
                    let status = format!("{:?}", c.status).to_lowercase();
                    let kind = format!("{:?}", c.kind).to_lowercase();
                    println!(
                        "  {} {} [{}] {}",
                        c.id.cyan(),
                        c.canonical_name,
                        kind.dimmed(),
                        status
                    );
                }
            }
        }
    }

    Ok(())
}

fn run_diff(args: DiffArgs) -> Result<()> {
    let old = load_pack_json(&args.old)?;
    let new = load_pack_json(&args.new)?;

    let diff_result = diff_packs(&old, &new);

    match args.format {
        PackOutputFormat::Json => {
            let out =
                serde_json::to_string_pretty(&diff_result).context("Failed to serialize diff")?;
            println!("{}", out);
        }
        PackOutputFormat::Jsonl => {
            for entry in &diff_result.entries {
                let json = serde_json::to_string(&entry).unwrap();
                println!("{}", json);
            }
        }
        PackOutputFormat::Human => {
            println!(
                "Diff: {} -> {}",
                diff_result.old_pack_id, diff_result.new_pack_id
            );
            println!();
            println!(
                "{} additive, {} definitional, {} deprecating, {} breaking, {} governance, {} signature-only",
                diff_result.summary.additive_count.to_string().green(),
                diff_result.summary.definitional_change_count.to_string().yellow(),
                diff_result.summary.deprecating_count.to_string().magenta(),
                diff_result.summary.breaking_count.to_string().red(),
                diff_result.summary.governance_critical_count.to_string().red().bold(),
                diff_result.summary.signature_only_count,
            );

            if diff_result.entries.is_empty() {
                println!("\nNo differences found.");
            } else {
                println!();
                for entry in &diff_result.entries {
                    let classification = format_classification(entry.classification);
                    println!(
                        "  [{}] {}: {}",
                        classification, entry.subject_id, entry.detail,
                    );
                }
            }
        }
    }

    let has_breaking = diff_result
        .entries
        .iter()
        .any(|e| matches!(e.classification, DiffClassification::Breaking));
    if has_breaking {
        std::process::exit(EXIT_SEMANTIC_FAILED);
    }
    Ok(())
}

fn run_sign(args: SignArgs) -> Result<()> {
    let pack = load_pack_json(&args.pack)?;
    let key_pem = fs::read(&args.key)
        .with_context(|| format!("Failed to read private key: {}", args.key.display()))?;

    let sign_output = match sign_pack(&pack, &key_pem) {
        Ok(o) => o,
        Err(e) => {
            let msg = format!("Signing failed: {:?}", e);
            eprintln!("{}", msg.red());
            std::process::exit(EXIT_SIGNATURE_FAILURE);
        }
    };

    let signer_id = match derive_signer_id(&key_pem) {
        Ok(id) => id,
        Err(e) => {
            let msg = format!("Failed to derive signer identity: {:?}", e);
            eprintln!("{}", msg.red());
            std::process::exit(EXIT_SIGNATURE_FAILURE);
        }
    };

    let mut signed_pack = pack;
    signed_pack.trust.signature_state = SignatureState::Signed;
    signed_pack.trust.signature = Some(sign_output.signature);
    signed_pack.trust.signature_alg = Some(sign_output.signature_alg);
    signed_pack.trust.signed_by = Some(signer_id);

    let pack_json =
        serde_json::to_string_pretty(&signed_pack).context("Failed to serialize signed pack")?;

    match args.out {
        Some(ref p) => {
            fs::write(p, &pack_json)
                .with_context(|| format!("Failed to write signed pack to: {}", p.display()))?;
            eprintln!("Signed pack written to: {}", p.display());
        }
        None => println!("{}", pack_json),
    }

    Ok(())
}

fn run_verify(args: VerifyArgs) -> Result<()> {
    let pack = load_pack_json(&args.pack)?;
    let key_pem = fs::read(&args.key)
        .with_context(|| format!("Failed to read public key: {}", args.key.display()))?;

    match verify_pack_signature(&pack, &key_pem) {
        Ok(()) => {
            println!(
                "Signature verified for pack '{}' ({})",
                pack.pack_id,
                pack.trust.signature_alg.as_deref().unwrap_or("unknown")
            );
            Ok(())
        }
        Err(e) => {
            let msg = format!(
                "Signature verification failed for pack '{}': {:?}",
                pack.pack_id, e
            );
            eprintln!("{}", msg.red());
            std::process::exit(EXIT_SIGNATURE_FAILURE);
        }
    }
}

fn print_diagnostics(diagnostics: &[SemanticDiagnostic], format: PackOutputFormat) {
    match format {
        PackOutputFormat::Json => {
            let out: Vec<_> = diagnostics.iter().map(diagnostics_to_json).collect();
            eprintln!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        PackOutputFormat::Jsonl => {
            for d in diagnostics {
                let json = serde_json::to_string(&diagnostics_to_json(d)).unwrap();
                eprintln!("{}", json);
            }
        }
        PackOutputFormat::Human => {
            for d in diagnostics {
                let sev = format_severity(d.severity);
                eprintln!(
                    "[{}] {} ({}): {}",
                    sev,
                    d.code.as_str(),
                    d.source_ref.uri,
                    d.message
                );
                if !d.recoverability_hint.is_empty() {
                    eprintln!("  hint: {}", d.recoverability_hint.dimmed());
                }
            }
        }
    }
}

fn diagnostics_to_json(d: &SemanticDiagnostic) -> serde_json::Value {
    serde_json::json!({
        "code": d.code.as_str(),
        "severity": format!("{:?}", d.severity).to_lowercase(),
        "semantic_truth": format!("{:?}", d.semantic_truth).to_lowercase(),
        "message": d.message,
        "source_ref": d.source_ref.uri,
        "pack_ref": d.pack_ref.pack_id,
        "recoverability_hint": d.recoverability_hint,
    })
}

fn format_severity(severity: DiagnosticSeverity) -> String {
    match severity {
        DiagnosticSeverity::Error => "ERROR".red().to_string(),
        DiagnosticSeverity::Warning => "WARN".yellow().to_string(),
        DiagnosticSeverity::Info => "INFO".blue().to_string(),
        DiagnosticSeverity::Hint => "HINT".dimmed().to_string(),
    }
}

fn format_classification(c: DiffClassification) -> String {
    match c {
        DiffClassification::Additive => "ADD".green().to_string(),
        DiffClassification::DefinitionalChange => "DEF".yellow().to_string(),
        DiffClassification::Deprecating => "DEP".magenta().to_string(),
        DiffClassification::Breaking => "BRK".red().to_string(),
        DiffClassification::GovernanceCritical => "GOV".red().bold().to_string(),
        DiffClassification::SignatureOnly => "SIG".dimmed().to_string(),
    }
}
