//! Golden-trace fixture for `conformance/08_authority` (audit §5 item 8, Phase 6).
//!
//! Pins a full [`AuthorityTrace`] + [`AuthorityDecision`] (and the two input packs)
//! produced by evaluating a `ShipOrder` request against a credit-`Hold` order with
//! two policies of differing specificity loaded:
//!   - a prohibition (priority 100) that applies and **denies**, and
//!   - a permission (priority 50) that is a structural *candidate* but is excluded
//!     by its `when` condition (`credit_status == Clear`).
//!
//! This exercises multi-pack provenance: `pack_hashes` has two entries and both
//! policies appear in `candidate_policies`, while only the prohibition is
//! `applicable`. Every one of the seven trace hashes is byte-stable because
//! authority hashing flows through `canonical_json_string` (sorted-key
//! serialization) — see `sea-core/src/authority/types.rs`. The pinning test
//! therefore normalizes **only** genuinely volatile fields (wall-clock timestamps,
//! random `decision_id` / `trace_ref`); hashes are pinned byte-for-byte.
//!
//! Regenerate after a deliberate shape change:
//!   cargo test --features cli --test authority_fixture_tests generate_fixtures -- --ignored

#![cfg(feature = "cli")]

use sea_core::authority::pack::compute_pack_hash;
use sea_core::authority::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn conformance_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("conformance")
        .join("08_authority")
}

fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("seaforge-contract-v1.json")
}

// --- builders (fixed IDs for byte-stability; request_id feeds action_request_hash) ---

fn make_request() -> AuthorityRequest {
    AuthorityRequest {
        request_id: "req-authority-001".to_string(),
        actor: ActorContext {
            id: "user-1".to_string(),
            role: Some("WarehouseOperator".to_string()),
            groups: vec![],
            service_account: None,
            agent_identity: None,
        },
        operation: "ShipOrder".to_string(),
        resource: ResourceRef {
            id: None,
            type_: Some("Order".to_string()),
            extra: Default::default(),
        },
        context: json!({}),
        requested_at: chrono::Utc::now(),
        correlation_id: None,
        risk_class: None,
        metadata: Default::default(),
    }
}

fn make_trusted_fact(path: &str, value: Value) -> FactEnvelope {
    FactEnvelope {
        path: path.to_string(),
        value,
        source_class: SourceClass::SystemOfRecord,
        source_id: "credit-service".to_string(),
        observed_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        evidence_ref: Some("ref-1".to_string()),
        signature: None,
        confidence: None,
        lineage: None,
    }
}

fn make_prohibition_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "block_credit_hold_shipping".to_string(),
        modality: PolicyModality::Prohibition,
        priority: 100,
        applies_to: StructuralPredicates {
            predicates: {
                let mut m = std::collections::HashMap::new();
                m.insert("action".to_string(), json!("ShipOrder"));
                m.insert("resource.type".to_string(), json!("Order"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert("customer.credit_status".to_string(), json!("Hold"));
                m
            },
        }),
        requires_fact: vec![FactRequirement {
            fact_path: "customer.credit_status".to_string(),
            allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Derived],
            allowed_source_ids: vec![],
            max_age: None,
            evidence_ref_required: true,
            signature_required: false,
            minimum_confidence: None,
            required_transform: None,
            derived_from_source: None,
        }],
        semantics_version: "0.4".to_string(),
        override_spec: None,
        obligation_spec: None,
        description: Some("Block shipping for credit hold orders".to_string()),
        evidence_ref: None,
    }];
    let hash = compute_pack_hash("shipping-policy", "1.0.0", "0.4", "default", &policies)
        .expect("prohibition pack hash");
    AuthorityPack {
        id: "shipping-policy".to_string(),
        version: "1.0.0".to_string(),
        semantics_version: "0.4".to_string(),
        required_specificity_profile: "default".to_string(),
        policies,
        hash,
        signature: None,
        owner: None,
        created_at: None,
        approved_by: None,
        evidence_ref: None,
    }
}

fn make_permission_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "allow_clear_order_shipping".to_string(),
        modality: PolicyModality::Permission,
        priority: 50,
        applies_to: StructuralPredicates {
            predicates: {
                let mut m = std::collections::HashMap::new();
                m.insert("actor.role".to_string(), json!("WarehouseOperator"));
                m.insert("action".to_string(), json!("ShipOrder"));
                m.insert("resource.type".to_string(), json!("Order"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert("customer.credit_status".to_string(), json!("Clear"));
                m
            },
        }),
        requires_fact: vec![FactRequirement {
            fact_path: "customer.credit_status".to_string(),
            allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Derived],
            allowed_source_ids: vec![],
            max_age: None,
            evidence_ref_required: false,
            signature_required: false,
            minimum_confidence: None,
            required_transform: None,
            derived_from_source: None,
        }],
        semantics_version: "0.4".to_string(),
        override_spec: None,
        obligation_spec: None,
        description: None,
        evidence_ref: None,
    }];
    let hash = compute_pack_hash("shipping-permission", "1.0.0", "0.4", "default", &policies)
        .expect("permission pack hash");
    AuthorityPack {
        id: "shipping-permission".to_string(),
        version: "1.0.0".to_string(),
        semantics_version: "0.4".to_string(),
        required_specificity_profile: "default".to_string(),
        policies,
        hash,
        signature: None,
        owner: None,
        created_at: None,
        approved_by: None,
        evidence_ref: None,
    }
}

fn make_env_config(packs: Vec<Value>) -> AuthorityEnvironmentConfig {
    AuthorityEnvironmentConfig {
        resolver_semantics_version: "0.4".to_string(),
        specificity_profile: SpecificityProfile::default_profile(),
        unknown_handling: UnknownHandlingConfig::defaults(),
        fact_sources: vec![FactSource {
            id: "credit-service".to_string(),
            source_class: SourceClass::SystemOfRecord,
            allowed_paths: vec!["customer.".to_string()],
            evidence_required: true,
            signature_required: false,
            max_response_latency_ms: 5000,
            health_endpoint: None,
            credential_ref: None,
            schema_ref: None,
            owner: None,
            recovery_hint: None,
        }],
        fact_transforms: vec![],
        authority_packs: packs,
        strict_mode: true,
        compatibility_lowering_version: "bounded_compatibility_v1".to_string(),
        resolver_version: "0.1.0".to_string(),
    }
}

/// All six raw (un-normalized) fixture values: the evaluation **inputs**
/// (`config`, `request`, `facts`) plus the input `packs` and the produced
/// `trace`/`decision`. Every binding drives `evaluate` from the first three and
/// must reproduce the last two byte-for-byte (volatile-normalized).
struct RawFixture {
    packs: Value,
    config: Value,
    request: Value,
    facts: Value,
    trace: Value,
    decision: Value,
}

/// Build the raw (un-normalized) fixture values from the single Rust builder set:
/// the evaluation inputs (config, request, facts), the two input packs, and the
/// full trace + decision they produce. Schema-valid but carrying real timestamps/IDs.
fn generate_raw_fixture() -> RawFixture {
    let prohibition = make_prohibition_pack();
    let permission = make_permission_pack();
    let packs_json = json!([
        serde_json::to_value(&prohibition).expect("serialize prohibition"),
        serde_json::to_value(&permission).expect("serialize permission"),
    ]);

    let config = make_env_config(vec![
        serde_json::to_value(&prohibition).unwrap(),
        serde_json::to_value(&permission).unwrap(),
    ]);
    // Serialize config BEFORE it is consumed by AuthorityEnvironment::new.
    let config_json = serde_json::to_value(&config).expect("serialize config");
    let mut env = AuthorityEnvironment::new(config).expect("env created");
    env.validate().expect("env valid");

    let request = make_request();
    let facts = vec![make_trusted_fact("customer.credit_status", json!("Hold"))];
    // Serialize request and facts BEFORE they are consumed by evaluate.
    let request_json = serde_json::to_value(&request).expect("serialize request");
    let facts_json = serde_json::to_value(&facts).expect("serialize facts");

    let (trace, decision) = env.evaluate(&request, &facts).expect("evaluate");

    let trace_json = serde_json::to_value(&trace).expect("serialize trace");
    let decision_json = serde_json::to_value(&decision).expect("serialize decision");

    RawFixture {
        packs: packs_json,
        config: config_json,
        request: request_json,
        facts: facts_json,
        trace: trace_json,
        decision: decision_json,
    }
}

// --- volatile-field normalization (timestamps + random IDs only; hashes pinned) ---

fn is_volatile(key: &str) -> bool {
    matches!(
        key,
        "decision_id" | "trace_ref" | "created_at" | "requested_at" | "observed_at" | "expires_at"
    )
}

fn volatile_sentinel(key: &str) -> String {
    format!("<{key}>")
}

fn normalize_volatile(value: Value) -> Value {
    let v = normalize_volatile_inner(value);
    sort_object_keys(v)
}

fn normalize_volatile_inner(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, val) in map {
                if is_volatile(&key) {
                    out.insert(key.clone(), Value::String(volatile_sentinel(&key)));
                } else {
                    out.insert(key, normalize_volatile_inner(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_volatile_inner).collect()),
        other => other,
    }
}

fn sort_object_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map
                .into_iter()
                .map(|(k, v)| (k, sort_object_keys(v)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(entries.into_iter().collect())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_object_keys).collect()),
        other => other,
    }
}

fn validate_against_schema(instance: &Value, def_name: &str) -> Result<(), String> {
    let schema_text = std::fs::read_to_string(schema_path())
        .map_err(|e| format!("failed to read schema: {e}"))?;
    let schema_json: Value =
        serde_json::from_str(&schema_text).map_err(|e| format!("invalid schema JSON: {e}"))?;
    let sub_schema = schema_json
        .get("$defs")
        .and_then(|d| d.get(def_name))
        .unwrap_or_else(|| panic!("$defs/{} not found in schema", def_name));
    if jsonschema::is_valid(sub_schema, instance) {
        Ok(())
    } else {
        Err(format!("instance does not conform to $defs/{}", def_name))
    }
}

fn load_committed(file: &str) -> Value {
    let path = conformance_dir().join(file);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("{} fixture must exist; run generate_fixtures first", file));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", file))
}

#[test]
fn packs_fixture_is_schema_valid_and_byte_pinned() {
    let committed = load_committed("packs.json");
    let packs = committed.as_array().expect("packs.json is an array");
    assert_eq!(packs.len(), 2, "08_authority pins exactly two input packs");
    for pack in packs {
        validate_against_schema(pack, "pack").expect("each pack must conform to $defs/pack");
    }

    let fixture = generate_raw_fixture();
    assert_eq!(
        normalize_volatile(fixture.packs),
        normalize_volatile(committed),
        "packs.json must match regenerated output. Re-run generate_fixtures if the change is deliberate."
    );
}

#[test]
fn trace_fixture_is_schema_valid_and_byte_pinned() {
    let committed = load_committed("trace.json");
    validate_against_schema(&committed, "trace").expect("trace must conform to $defs/trace");

    let fixture = generate_raw_fixture();
    assert_eq!(
        normalize_volatile(fixture.trace),
        normalize_volatile(committed),
        "trace.json must match regenerated output (hashes pinned byte-for-byte; only timestamps/IDs normalized). \
         Re-run generate_fixtures if the trace shape changed deliberately."
    );
}

#[test]
fn decision_fixture_is_schema_valid_and_byte_pinned() {
    let committed = load_committed("decision.json");
    validate_against_schema(&committed, "decision").expect("decision must conform to $defs/decision");

    let fixture = generate_raw_fixture();
    assert_eq!(
        normalize_volatile(fixture.decision),
        normalize_volatile(committed),
        "decision.json must match regenerated output. Re-run generate_fixtures if deliberate."
    );
}

#[test]
fn config_input_is_byte_pinned() {
    let committed = load_committed("config.json");
    let fixture = generate_raw_fixture();
    assert_eq!(
        normalize_volatile(fixture.config),
        normalize_volatile(committed),
        "config.json must match regenerated output. Re-run generate_fixtures if the change is deliberate."
    );
}

#[test]
fn request_input_is_byte_pinned() {
    let committed = load_committed("request.json");
    let fixture = generate_raw_fixture();
    assert_eq!(
        normalize_volatile(fixture.request),
        normalize_volatile(committed),
        "request.json must match regenerated output. Re-run generate_fixtures if the change is deliberate."
    );
}

#[test]
fn facts_input_is_byte_pinned() {
    let committed = load_committed("facts.json");
    let fixture = generate_raw_fixture();
    assert_eq!(
        normalize_volatile(fixture.facts),
        normalize_volatile(committed),
        "facts.json must match regenerated output. Re-run generate_fixtures if the change is deliberate."
    );
}

/// One-shot helper: writes the seven committed fixture files (raw, schema-valid).
#[test]
#[ignore]
fn generate_fixtures() {
    let dir = conformance_dir();
    std::fs::create_dir_all(&dir).expect("create fixture dir");

    let fixture = generate_raw_fixture();

    let packs_text = serde_json::to_string_pretty(&fixture.packs).unwrap();
    let config_text = serde_json::to_string_pretty(&fixture.config).unwrap();
    let request_text = serde_json::to_string_pretty(&fixture.request).unwrap();
    let facts_text = serde_json::to_string_pretty(&fixture.facts).unwrap();
    let trace_text = serde_json::to_string_pretty(&fixture.trace).unwrap();
    let decision_text = serde_json::to_string_pretty(&fixture.decision).unwrap();

    std::fs::write(dir.join("packs.json"), &packs_text).expect("write packs.json");
    std::fs::write(dir.join("config.json"), &config_text).expect("write config.json");
    std::fs::write(dir.join("request.json"), &request_text).expect("write request.json");
    std::fs::write(dir.join("facts.json"), &facts_text).expect("write facts.json");
    std::fs::write(dir.join("trace.json"), &trace_text).expect("write trace.json");
    std::fs::write(dir.join("decision.json"), &decision_text).expect("write decision.json");

    let manifest = json!({
        "description": "Authority provenance golden (audit §5 item 8, Phase 6): two policies of \
            differing specificity (prohibition p100 + permission p50) are loaded; a ShipOrder \
            request on a credit-Hold order denies via the prohibition while the permission is a \
            candidate excluded by its when-condition. The full AuthorityTrace is pinned with all \
            seven hashes byte-stable (deterministic via canonical_json_string). Volatile fields \
            (timestamps, decision_id, trace_ref) are normalized by the pinning test. The shared \
            evaluation inputs (config, request, facts) are committed so every binding drives \
            identical inputs and byte-matches the same golden.",
        "command": "authority_trace",
        "packs": "packs.json",
        "config": "config.json",
        "request": "request.json",
        "facts": "facts.json",
        "trace": "trace.json",
        "decision": "decision.json",
        "schema": "../../schemas/seaforge-contract-v1.json"
    });
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("write manifest.json");

    eprintln!("08_authority fixtures written to {}", dir.display());
}
