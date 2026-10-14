//! OPA/Rego projection: a SEA model's `Policy` declarations become Rego v1
//! modules that reproduce the DomainForge evaluator's three-valued result per
//! policy and aggregate those results into an allow / deny / escalate decision.
//!
//! Scope:
//! - Policies are lowered from the resolved graph into a narrow policy IR
//!   ([`ir::RegoPolicyIr`]). A policy using any construct outside that IR
//!   fails the whole projection with [`RegoProjectionError::Unrepresentable`];
//!   nothing is approximated or dropped.
//! - `True` / `False` / `Unknown` and evaluation errors stay distinct end to
//!   end. Rego `undefined` never stands in for `Unknown`.
//! - The aggregate decision is this projection's own contract
//!   (`domainforge.rego.decision/v1`): the evaluator has no aggregate outcome.
//!   The fidelity report records it as `adapter`, not `isomorphic`.
//!
//! Output tree (deterministic; only `provenance.json` carries `created_at`):
//!
//! ```text
//! bundle/.manifest                          OPA bundle manifest (roots, revision)
//! bundle/data.json                          policy metadata + model identity
//! bundle/domainforge/sea/lib.rego           three-valued helpers (static)
//! bundle/domainforge/policy/decision.rego   decision contract (static)
//! bundle/domainforge/policy/policies.rego   one rule per policy (generated)
//! contracts/input.schema.json               typed input contract
//! contracts/decision.schema.json            decision contract
//! fixtures/<name>.json                      input + evaluator-derived expectation
//! tests/decision_test.rego                  `opa test` cases from the fixtures
//! fidelity-report.json                      per-policy fidelity classification
//! provenance.json                           model identity + file hashes
//! ```

pub mod ir;
pub mod reference;
mod render;

use crate::formatter::{format as format_source, FormatConfig};
use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use ir::RegoPolicyIr;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Version of the emitted artifact layout and Rego library.
pub const PROJECTION_VERSION: &str = "domainforge-rego/v1";
pub const FIDELITY_SCHEMA: &str = "domainforge.rego.fidelity/v1";
pub const PROVENANCE_SCHEMA: &str = "domainforge.rego.provenance/v1";

const LIB_REGO: &str = include_str!("rego/lib.rego");
const DECISION_REGO: &str = include_str!("rego/decision.rego");

pub const MANIFEST_FILE: &str = "bundle/.manifest";
pub const DATA_FILE: &str = "bundle/data.json";
pub const LIB_FILE: &str = "bundle/domainforge/sea/lib.rego";
pub const DECISION_FILE: &str = "bundle/domainforge/policy/decision.rego";
pub const POLICIES_FILE: &str = "bundle/domainforge/policy/policies.rego";
pub const INPUT_SCHEMA_FILE: &str = "contracts/input.schema.json";
pub const DECISION_SCHEMA_FILE: &str = "contracts/decision.schema.json";
pub const TEST_FILE: &str = "tests/decision_test.rego";
pub const FIDELITY_FILE: &str = "fidelity-report.json";
pub const PROVENANCE_FILE: &str = "provenance.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegoProjectionError {
    /// A policy uses a construct the projection cannot express faithfully.
    Unrepresentable {
        policy: String,
        construct: String,
        reason: String,
    },
    /// The model could not be turned into projection input.
    Model(String),
    /// An artifact could not be written.
    Write(String),
}

impl std::fmt::Display for RegoProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unrepresentable {
                policy,
                construct,
                reason,
            } => write!(
                f,
                "REGO001 unrepresentable: policy '{policy}' uses {construct}: {reason}"
            ),
            Self::Model(message) => write!(f, "REGO002 model: {message}"),
            Self::Write(message) => write!(f, "REGO003 write: {message}"),
        }
    }
}

impl std::error::Error for RegoProjectionError {}

fn sha256(content: &str) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(content.as_bytes())))
}

fn pretty(value: &Value) -> String {
    let mut out = serde_json::to_string_pretty(value).expect("JSON values serialize");
    out.push('\n');
    out
}

/// The same model hash `cell.lock` records: sha256 of the canonically
/// formatted source, so whitespace-only edits do not change the identity.
pub fn model_hash(source: &str) -> Result<String, RegoProjectionError> {
    let canonical = format_source(source, FormatConfig::default())
        .map_err(|e| RegoProjectionError::Model(format!("failed to canonicalize source: {e}")))?;
    Ok(sha256(&canonical))
}

fn identity(model_hash: &str) -> Value {
    json!({ "hash": model_hash, "projection": PROJECTION_VERSION })
}

fn data_document(ir: &RegoPolicyIr, model_hash: &str) -> Value {
    let mut policies = Map::new();
    for p in &ir.policies {
        policies.insert(
            p.key.clone(),
            json!({
                "id": p.id,
                "name": p.name,
                "namespace": p.namespace,
                "version": p.version,
                "modality": p.modality,
                "kind": p.kind,
                "priority": p.priority,
                "severity": p.severity,
                "rationale": p.rationale,
                "tags": p.tags,
            }),
        );
    }
    json!({
        "domainforge": {
            "model": {
                "identity": identity(model_hash),
                "collections": ir.collections,
                "policies": policies,
            }
        }
    })
}

fn input_schema(ir: &RegoPolicyIr) -> Value {
    let mut collections = Map::new();
    for name in &ir.collections {
        let fields: Map<String, Value> = ir
            .fields
            .iter()
            .filter(|(collection, _)| collection == name)
            .map(|(_, field)| (field.clone(), json!({})))
            .collect();
        collections.insert(
            name.clone(),
            json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "description": "Fields the policies read. A missing field is Unknown, not false.",
                    "properties": fields,
                },
            }),
        );
    }
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "domainforge.rego.input/v1",
        "type": "object",
        "required": ["facts"],
        "properties": {
            "facts": {
                "type": "object",
                "required": ir.collections,
                "properties": collections,
            }
        },
    })
}

fn decision_schema() -> Value {
    let tri = json!({ "enum": ["true", "false", "unknown", "error"] });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": reference::DECISION_CONTRACT,
        "type": "object",
        "additionalProperties": false,
        "required": ["contract", "outcome", "results", "reasons", "input_errors", "model"],
        "properties": {
            "contract": { "const": reference::DECISION_CONTRACT },
            "outcome": { "enum": ["allow", "deny", "escalate"] },
            "results": { "type": "object", "additionalProperties": tri },
            "reasons": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["policy", "code", "modality", "severity", "priority", "blocking"],
                    "properties": {
                        "policy": { "type": "string" },
                        "code": { "enum": ["violated", "unknown", "evaluation_error", "malformed_input"] },
                        "modality": { "type": "string" },
                        "severity": { "enum": ["error", "warning", "info"] },
                        "priority": { "type": "integer" },
                        "blocking": { "type": "boolean" },
                    },
                },
            },
            "input_errors": { "type": "array", "items": { "type": "string" } },
            "model": {
                "type": "object",
                "required": ["hash", "projection"],
                "properties": {
                    "hash": { "type": "string" },
                    "projection": { "type": "string" },
                },
            },
        },
    })
}

/// Fixture inputs derived from the model. Each expectation is computed by the
/// DomainForge evaluator, so `opa test` on the output is itself a differential
/// check for this model.
fn fixtures(
    graph: &Graph,
    ir: &RegoPolicyIr,
    identity: &Value,
) -> Result<Vec<(&'static str, Value)>, RegoProjectionError> {
    let model = reference::facts_from_graph(graph, ir).map_err(RegoProjectionError::Model)?;

    let empty: Map<String, Value> = ir
        .collections
        .iter()
        .map(|name| (name.clone(), json!([])))
        .collect();

    // Incomplete information: every field a policy reads is absent.
    let mut incomplete = model.clone();
    for (collection, field) in &ir.fields {
        if let Some(items) = incomplete["facts"][collection].as_array_mut() {
            for item in items {
                if let Some(object) = item.as_object_mut() {
                    object.remove(field);
                }
            }
        }
    }

    let inputs = [
        ("model", model),
        ("empty", json!({ "facts": empty })),
        ("incomplete", incomplete),
        ("malformed", json!({ "facts": null })),
    ];
    Ok(inputs
        .into_iter()
        .map(|(name, input)| {
            let expected = reference::decide(ir, identity, &input);
            (name, json!({ "input": input, "expected": expected }))
        })
        .collect())
}

fn test_module(fixtures: &[(&'static str, Value)]) -> String {
    let mut out = String::from(
        "# Generated by `domainforge project --format rego`. Do not edit.\n\
         # Expectations come from the DomainForge evaluator (see fixtures/).\n\
         package domainforge.policy_test\n\nimport rego.v1\n\nimport data.domainforge.policy\n",
    );
    for (name, fixture) in fixtures {
        out.push_str(&format!(
            "\ntest_{name} if {{\n\tpolicy.decision == {} with input as {}\n}}\n",
            fixture["expected"], fixture["input"]
        ));
    }
    out
}

fn fidelity_report(ir: &RegoPolicyIr, model_hash: &str) -> Value {
    let policies: Vec<Value> = ir
        .policies
        .iter()
        .map(|p| {
            json!({
                "policy": p.key,
                "classification": "isomorphic",
                "constructs": p.constructs,
            })
        })
        .collect();
    json!({
        "schema": FIDELITY_SCHEMA,
        "projection": PROJECTION_VERSION,
        "model": identity(model_hash),
        "classes": {
            "isomorphic": "the per-policy True/False/Unknown/error result equals the DomainForge evaluator's",
            "structural": "carried as data with the same shape; not evaluated",
            "adapter": "an explicit target adaptation, verified against a reference implementation",
            "unrepresentable": "cannot be projected; the projection fails with REGO001",
        },
        "policies": policies,
        "decision_contract": {
            "classification": "adapter",
            "contract": reference::DECISION_CONTRACT,
            "reason": "The evaluator reports per-policy violations and has no aggregate outcome. \
                       deny: any error-severity policy is violated or fails to evaluate, or the input is malformed; \
                       escalate: otherwise, any error-severity policy is Unknown; allow: otherwise. \
                       Priority orders reasons only; the evaluator does not use it to resolve conflicts.",
        },
        "metadata": {
            "classification": "structural",
            "fields": ["id", "name", "namespace", "version", "kind", "rationale", "tags"],
        },
        "caveats": [
            "Numbers compare exactly in OPA and as f64 in the evaluator; results can differ for integers beyond 2^53.",
            "Equality on array- or object-valued fields compares integer and float forms of the same number as equal in OPA and as different in the evaluator.",
            "Results reflect the evaluator's default three-valued mode; the boolean mode is not projected.",
        ],
        "verification": {
            "reference": "domainforge_core::policy::Policy::evaluate_with_mode (three-valued)",
            "suite": "domainforge-core/tests/rego_projection_tests.rs",
            "evaluator": "opa (see tests/decision_test.rego for this model's fixtures)",
        },
    })
}

/// Emit the Rego projection into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    source: &str,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, RegoProjectionError> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ir = RegoPolicyIr::build(graph)?;
    let model_hash = model_hash(source)?;
    let identity = identity(&model_hash);
    let fixtures = fixtures(graph, &ir, &identity)?;

    let mut files: BTreeMap<String, String> = BTreeMap::new();
    files.insert(
        MANIFEST_FILE.to_string(),
        pretty(&json!({
            "revision": model_hash,
            "roots": ["domainforge"],
            "rego_version": 1,
            "metadata": { "projection": PROJECTION_VERSION },
        })),
    );
    files.insert(
        DATA_FILE.to_string(),
        pretty(&data_document(&ir, &model_hash)),
    );
    files.insert(LIB_FILE.to_string(), LIB_REGO.to_string());
    files.insert(DECISION_FILE.to_string(), DECISION_REGO.to_string());
    files.insert(
        POLICIES_FILE.to_string(),
        render::policies_module(&ir.policies, &model_hash),
    );
    files.insert(INPUT_SCHEMA_FILE.to_string(), pretty(&input_schema(&ir)));
    files.insert(DECISION_SCHEMA_FILE.to_string(), pretty(&decision_schema()));
    for (name, fixture) in &fixtures {
        files.insert(format!("fixtures/{name}.json"), pretty(fixture));
    }
    files.insert(TEST_FILE.to_string(), test_module(&fixtures));
    files.insert(
        FIDELITY_FILE.to_string(),
        pretty(&fidelity_report(&ir, &model_hash)),
    );

    let hashes: Map<String, Value> = files
        .iter()
        .map(|(path, content)| (path.clone(), json!(sha256(content))))
        .collect();
    files.insert(
        PROVENANCE_FILE.to_string(),
        pretty(&json!({
            "schema": PROVENANCE_SCHEMA,
            "projection": PROJECTION_VERSION,
            "generator": format!("domainforge-core {}", env!("CARGO_PKG_VERSION")),
            "model_ref": model_ref,
            "created_at": created_at,
            "model": identity,
            "policy_count": ir.policies.len(),
            "files": hashes,
        })),
    );

    for (path, content) in &files {
        sink.write(path, content)
            .map_err(RegoProjectionError::Write)?;
    }
    Ok(files.into_keys().collect())
}

/// Binding surface: source in, path→content map out, no filesystem.
pub fn project_rego_in_memory(
    graph: &Graph,
    source: &str,
    model_ref: &str,
    created_at: Option<String>,
) -> Result<BTreeMap<String, String>, RegoProjectionError> {
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(graph, source, model_ref, created_at, &mut sink)?;
    Ok(map)
}
