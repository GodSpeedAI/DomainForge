//! DomainForge Semantic Verification Projection (§14.0, profile
//! `domainforge-semantic-verification/v1`).
//!
//! Emits a CEP `work_request` envelope binding each verification-relevant
//! declared declaration to a stable CEP Question, Claim/obligation, expected
//! answer shape, and admissible Evidence requirement. It is deliberately thin:
//! it does not choose a verifier process, evaluate Evidence reliability, or
//! settle.
//!
//! Stable identity: `obligation_id`, `claim_id`, and `question_id` are
//! content-derived from (profile version, semantic_hash, decl_key, obligation
//! kind, canonical Question content), so repeated projection is stable. The
//! outer `envelope_id`/`created_at` vary per emission (I-DET).

use crate::application::envelope::{
    declaration_entity_ref, CanonicalDeclarationId, CanonicalReferenceTarget,
    CanonicalSemanticEnvelopeDocument, CanonicalSemanticPayload,
};
use crate::application::resolve::resolve_application_graph;
use crate::graph::Graph;
use crate::semantic_pack::canonical_json::{canonical_json, compute_sha256};
use serde_json::{json, Value};

/// Profile id (spec §14.0).
pub const SEMANTIC_VERIFICATION_PROFILE: &str = "domainforge-semantic-verification/v1";

/// CEP envelope kind for the verification contract.
pub const ENVELOPE_KIND_WORK_REQUEST: &str = "work_request";

/// Declared obligation kinds projected by Slice 2 (implementation decision,
/// documented in the Slice 2 report).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationKind {
    /// A declared Flow → Gauge Given/When/Then scenario.
    FlowBehavior,
    /// A declared Policy → Lean checked proposition.
    PolicyProof,
    /// A declared Entity availability obligation.
    Availability,
}

impl ObligationKind {
    fn key(self) -> &'static str {
        match self {
            ObligationKind::FlowBehavior => "flow_behavior",
            ObligationKind::PolicyProof => "policy_proof",
            ObligationKind::Availability => "availability",
        }
    }

    /// CEP-0006 §10 evidence type for the obligation.
    fn evidence_type(self) -> &'static str {
        match self {
            ObligationKind::FlowBehavior => "test_result",
            ObligationKind::PolicyProof => "conformance_result",
            ObligationKind::Availability => "configuration_snapshot",
        }
    }

    /// Which DomainForge projection family (or families) carries the obligation.
    fn projection_artifact(self) -> &'static str {
        match self {
            ObligationKind::FlowBehavior => "gauge",
            ObligationKind::PolicyProof => "lean",
            ObligationKind::Availability => "gauge",
        }
    }

    fn limitation(self) -> &'static str {
        match self {
            ObligationKind::FlowBehavior => {
                "Gauge step implementations remain external to the projection"
            }
            ObligationKind::PolicyProof => {
                "Lean checks the supported policy subset; unsupported policies are obligation stubs"
            }
            ObligationKind::Availability => {
                "availability obligations are declared-state only; no runtime observation"
            }
        }
    }
}

/// One projected obligation (spec §14.0 shape).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct VerificationObligation {
    pub obligation_id: String,
    pub decl_key: String,
    pub claim_id: String,
    pub question_id: String,
    pub question_form: String,
    pub expected_answer_shape: String,
    pub evidence_types: Vec<String>,
    pub projection_artifact_refs: Vec<String>,
    pub limitations: Vec<String>,
}

/// Content-derived id: `sha256(canonical_json({profile, semantic_hash,
/// decl_key, kind, canonical_content}))` truncated to a stable hex prefix.
fn content_id(fields: &Value) -> String {
    let digest = compute_sha256(canonical_json(fields).as_bytes());
    digest.strip_prefix("sha256:").unwrap_or(&digest)[..24].to_string()
}

/// Build the CEP `work_request` envelope for a resolved model. Flows come from
/// canonical declarations; the resolved graph supplies policies and entities (§14.0).
#[allow(clippy::too_many_arguments)]
pub fn build_verification_contract(
    doc: &CanonicalSemanticEnvelopeDocument,
    entry_logical_path: &str,
    sources_json: &str,
    source_set_hash: &str,
    envelope_id: &str,
    created_at: &str,
    registry_content_hash: Option<&str>,
    resolved_namespaces: &[(String, String)],
) -> Result<Value, String> {
    let semantic_hash = doc.semantic_closure_hash.clone();
    let graph = resolve_application_graph(entry_logical_path, sources_json).map_err(|diags| {
        diags
            .iter()
            .map(|d| format!("{}: {}", d.code.slug(), d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;

    let obligations = project_obligations(doc, &graph, &semantic_hash);

    // CEP work_request envelope (validated by Gate 0A in the proof).
    let created_by = format!("domainforge {}", env!("CARGO_PKG_VERSION"));

    let mut envelope = serde_json::Map::new();
    envelope.insert("envelope_id".to_string(), json!(envelope_id));
    envelope.insert("cep_version".to_string(), json!("0.1.0"));
    envelope.insert("envelope_version".to_string(), json!("1.0.0"));
    envelope.insert(
        "envelope_kind".to_string(),
        json!(ENVELOPE_KIND_WORK_REQUEST),
    );
    envelope.insert("created_at".to_string(), json!(created_at));
    envelope.insert("created_by".to_string(), json!(created_by));
    envelope.insert(
        "scope".to_string(),
        json!({
            "entry_logical_path": entry_logical_path,
            "semantic_hash": semantic_hash,
            "snapshot_ref": format!("snapshot:{semantic_hash}"),
            "profile": SEMANTIC_VERIFICATION_PROFILE,
        }),
    );
    envelope.insert(
        "boundary_record".to_string(),
        json!({
            "scope": entry_logical_path,
            "included_sections": ["questions", "constraints", "references"],
            "excluded_sections": ["benchmarks", "settlements"],
            "known_omissions": ["runtime state", "implementation test bindings",
                                "postconditions", "CEP Evidence and Settlement results"],
            "unknowns": [],
            "redactions": [],
            "compression_notes": [],
            "out_of_scope_entities": [],
            "limitations": ["declared-state only; does not observe runtime behavior"],
        }),
    );
    envelope.insert(
        "completeness_status".to_string(),
        json!("complete_for_declared_state"),
    );
    envelope.insert(
        "omission_status".to_string(),
        json!("known_omissions_recorded"),
    );
    envelope.insert(
        "provenance_refs".to_string(),
        json!([format!("source-set:{source_set_hash}")]),
    );
    envelope.insert("validation_status".to_string(), json!("valid"));
    envelope.insert("lineage_refs".to_string(), json!([]));
    envelope.insert(
        "provenance".to_string(),
        json!([{
            "provenance_id": format!("provenance:{envelope_id}"),
            "source_system_refs": [format!("source-set:{source_set_hash}")],
            "producer_refs": [created_by],
            "production_method": "domainforge envelope --emit verification-contract",
            "created_at": created_at,
            "verification_status": "unverified",
        }]),
    );
    envelope.insert(
        "references".to_string(),
        json!([{
            "ref_id": format!("snapshot:{semantic_hash}"),
            "ref_type": "semantic_snapshot",
            "target_uri_or_id": entry_logical_path,
            "target_hash": semantic_hash,
            "availability_status": "available",
            "integrity_status": "verifiable",
            "scope": entry_logical_path,
            "limitations": [],
        }]),
    );

    // §14.0 obligation rows → CEP questions + constraints.
    let questions: Vec<Value> = obligations
        .iter()
        .map(|o| {
            json!({
                "question_id": o.question_id,
                "question_form": o.question_form,
                "question_type": "verification",
                "scope": o.decl_key,
                "target_entities": [o.decl_key],
                "expected_answer_shape": o.expected_answer_shape,
            })
        })
        .collect();
    envelope.insert("questions".to_string(), json!(questions));

    let constraints: Vec<Value> = obligations
        .iter()
        .map(|o| {
            json!({
                "constraint_id": format!("evidence:{}", o.obligation_id),
                "constraint_type": "evidence_requirement",
                "description": format!(
                    "obligation {} requires at least one evidence record of type {}",
                    o.obligation_id,
                    o.evidence_types.join(",")
                ),
                "scope": o.decl_key,
                "source": SEMANTIC_VERIFICATION_PROFILE,
                "status": "active",
            })
        })
        .collect();
    envelope.insert("constraints".to_string(), json!(constraints));

    // The full obligation rows ride in the profile extension so the contract
    // import can materialize Claim/obligation lookup rows without re-enveloping.
    envelope.insert(
        "extensions".to_string(),
        json!({
            "domainforge": {
                "profile": SEMANTIC_VERIFICATION_PROFILE,
                "semantic_hash": semantic_hash,
                "snapshot_ref": format!("snapshot:{semantic_hash}"),
                "obligations": obligations,
                "registry_content_hash": registry_content_hash,
                "resolved_namespaces": resolved_namespaces,
            }
        }),
    );

    Ok(Value::Object(envelope))
}

/// Enumerate obligations from canonical declarations and the graph, sorted
/// deterministically by decl_key so output is byte-stable across emissions.
fn project_obligations(
    doc: &CanonicalSemanticEnvelopeDocument,
    graph: &Graph,
    semantic_hash: &str,
) -> Vec<VerificationObligation> {
    let mut obligations: Vec<VerificationObligation> = Vec::new();

    for decl in &doc.envelope.semantic_declarations {
        let CanonicalSemanticPayload::Flow(flow) = &decl.declaration else {
            continue;
        };
        let decl_key = declaration_entity_ref(&decl.id);
        let reference = |target: &CanonicalReferenceTarget| match target {
            CanonicalReferenceTarget::Declaration {
                id: CanonicalDeclarationId::Concept { id },
            } => id.to_string(),
            CanonicalReferenceTarget::Declaration { id } => declaration_entity_ref(id),
            CanonicalReferenceTarget::Namespace {
                exact_name,
                logical_module_id,
            } => format!("{logical_module_id}:{exact_name}"),
        };
        let resource = reference(&flow.resource);
        let from = reference(&flow.from_entity);
        let to = reference(&flow.to_entity);
        let question_content = json!({
            "q": format!("does the declared flow {resource} from {from} to {to} hold at realization?"),
        });
        push_obligation(
            &mut obligations,
            semantic_hash,
            &decl_key,
            ObligationKind::FlowBehavior,
            &question_content,
        );
    }

    for policy in graph.all_policies() {
        let decl_key = format!("concept:{}", policy.id);
        let question_content = json!({
            "q": format!("does declared policy {} hold under its proof polarity?", policy.name),
        });
        push_obligation(
            &mut obligations,
            semantic_hash,
            &decl_key,
            ObligationKind::PolicyProof,
            &question_content,
        );
    }

    for entity in graph.all_entities() {
        let decl_key = format!("concept:{}", entity.id());
        let question_content = json!({
            "q": format!("is declared entity {} available at realization?", entity.name()),
        });
        push_obligation(
            &mut obligations,
            semantic_hash,
            &decl_key,
            ObligationKind::Availability,
            &question_content,
        );
    }

    obligations.sort_by(|a, b| a.decl_key.cmp(&b.decl_key));
    obligations
}

/// Build + push one obligation (content-derived stable ids).
fn push_obligation(
    obligations: &mut Vec<VerificationObligation>,
    semantic_hash: &str,
    decl_key: &str,
    kind: ObligationKind,
    question_content: &Value,
) {
    let obligation_fields = json!({
        "profile": SEMANTIC_VERIFICATION_PROFILE,
        "semantic_hash": semantic_hash,
        "decl_key": decl_key,
        "kind": kind.key(),
    });
    let question_fields = json!({
        "profile": SEMANTIC_VERIFICATION_PROFILE,
        "semantic_hash": semantic_hash,
        "decl_key": decl_key,
        "kind": kind.key(),
        "content": question_content,
    });
    let claim_content = json!({
        "claim": format!("declared {} obligation is satisfied at realization", kind.key()),
    });
    let claim_fields = json!({
        "profile": SEMANTIC_VERIFICATION_PROFILE,
        "semantic_hash": semantic_hash,
        "decl_key": decl_key,
        "kind": kind.key(),
        "content": claim_content,
    });

    let obligation_id = content_id(&obligation_fields);
    let question_id = content_id(&question_fields);
    let claim_id = content_id(&claim_fields);
    let question_form = match kind {
        ObligationKind::FlowBehavior => "does_hold",
        ObligationKind::PolicyProof => "does_hold",
        ObligationKind::Availability => "is_available",
    };

    obligations.push(VerificationObligation {
        decl_key: decl_key.to_string(),
        obligation_id,
        claim_id,
        question_id,
        question_form: question_form.to_string(),
        expected_answer_shape: "boolean_with_failure_list".to_string(),
        evidence_types: vec![kind.evidence_type().to_string()],
        projection_artifact_refs: vec![format!("projection:{}", kind.projection_artifact())],
        limitations: vec![kind.limitation().to_string()],
    });
}
