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
        .join("12_seaforge_fixture")
}

fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("seaforge-contract-v1.json")
}

fn build_prohibition_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "block_credit_hold".to_string(),
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
            allowed_source_classes: vec![SourceClass::SystemOfRecord],
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
        description: None,
        evidence_ref: None,
    }];
    let hash = compute_pack_hash("sp", "1.0.0", "0.4", "default", &policies).expect("hash");
    AuthorityPack {
        id: "sp".to_string(),
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

fn make_request() -> AuthorityRequest {
    AuthorityRequest {
        request_id: "req-contract-001".to_string(),
        actor: ActorContext {
            id: "user-1".to_string(),
            role: Some("Operator".to_string()),
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

/// Recursively replace volatile field values with stable sentinels so the
/// fixture is byte-stable across runs. Fields normalized:
/// - Timestamps: created_at, requested_at, observed_at, expires_at (wall-clock)
/// - IDs: decision_id, trace_ref (random UUIDv4)
///
/// Hash fields are intentionally **not** normalized: authority hashing now flows
/// through `canonical_json_string` (sorted-key serialization in
/// `authority/types.rs`), so `hash`, `ir_hash`, `pack_hashes`,
/// `resolver_semantics_hash`, `specificity_profile_hash`,
/// `unknown_handling_config_hash`, and `action_request_hash` are deterministic
/// and are pinned byte-for-byte — a drift in any hash now correctly fails.
fn normalize_volatile(mut value: Value) -> Value {
    match &mut value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if is_volatile(key) {
                    *val = Value::String(volatile_sentinel(key));
                } else {
                    normalize_volatile_ref(val);
                }
            }
        }
        Value::Array(items) => {
            for item in items.iter_mut() {
                normalize_volatile_ref(item);
            }
        }
        _ => {}
    }
    sort_object_keys(value)
}

fn is_volatile(key: &str) -> bool {
    matches!(
        key,
        "decision_id" | "trace_ref" | "created_at" | "requested_at" | "observed_at" | "expires_at"
    )
}

fn volatile_sentinel(key: &str) -> String {
    format!("<{key}>")
}

fn sort_object_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map
                .into_iter()
                .map(|(k, v)| (k, sort_object_keys(v)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let result: serde_json::Map<String, Value> = entries.into_iter().collect();
            Value::Object(result)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_object_keys).collect()),
        other => other,
    }
}

fn normalize_volatile_ref(val: &mut Value) {
    if let Value::Object(map) = val {
        for (key, v) in map.iter_mut() {
            if is_volatile(key) {
                *v = Value::String(volatile_sentinel(key));
            } else {
                normalize_volatile_ref(v);
            }
        }
    } else if let Value::Array(items) = val {
        for item in items.iter_mut() {
            normalize_volatile_ref(item);
        }
    }
}

/// Generate the raw (un-normalized) fixture values from the builders.
/// These have real timestamps, hashes, and IDs — schema-valid but not byte-stable.
fn generate_raw_fixture() -> (Value, Value, Value) {
    let pack = build_prohibition_pack();
    let pack_json = serde_json::to_value(&pack).expect("serialize pack");

    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).expect("env");
    env.validate().expect("valid");

    let request = make_request();
    let facts = vec![make_trusted_fact("customer.credit_status", json!("Hold"))];
    let (trace, decision) = env.evaluate(&request, &facts).expect("eval");

    let trace_json = serde_json::to_value(&trace).expect("serialize trace");
    let decision_json = serde_json::to_value(&decision).expect("serialize decision");

    (pack_json, trace_json, decision_json)
}

#[test]
fn pack_fixture_is_schema_valid_and_byte_pinned() {
    let dir = conformance_dir();
    let committed_raw = std::fs::read_to_string(dir.join("pack.json"))
        .expect("pack.json fixture must exist; run generate_fixtures first");
    let committed_raw: Value = serde_json::from_str(&committed_raw).expect("parse pack.json");

    validate_against_schema(&committed_raw, "pack").expect("pack must conform to schema");

    let (pack, _trace, _decision) = generate_raw_fixture();
    let committed = normalize_volatile(committed_raw);
    let generated = normalize_volatile(pack);
    assert_eq!(
        generated, committed,
        "pack.json fixture must match regenerated output (volatile fields normalized). \
         If the pack shape changed deliberately, re-run generate_fixtures to update."
    );
}

#[test]
fn trace_fixture_is_schema_valid_and_byte_pinned() {
    let dir = conformance_dir();
    let committed_raw = std::fs::read_to_string(dir.join("trace.json"))
        .expect("trace.json fixture must exist; run generate_fixtures first");
    let committed_raw: Value = serde_json::from_str(&committed_raw).expect("parse trace.json");

    validate_against_schema(&committed_raw, "trace").expect("trace must conform to schema");

    let (_pack, trace, _decision) = generate_raw_fixture();
    let committed = normalize_volatile(committed_raw);
    let generated = normalize_volatile(trace);
    assert_eq!(
        generated, committed,
        "trace.json fixture must match regenerated output (volatile fields normalized). \
         If the trace shape changed deliberately, re-run generate_fixtures to update."
    );
}

#[test]
fn decision_fixture_is_schema_valid_and_byte_pinned() {
    let dir = conformance_dir();
    let committed_raw = std::fs::read_to_string(dir.join("decision.json"))
        .expect("decision.json fixture must exist; run generate_fixtures first");
    let committed_raw: Value = serde_json::from_str(&committed_raw).expect("parse decision.json");

    validate_against_schema(&committed_raw, "decision").expect("decision must conform to schema");

    let (_pack, _trace, decision) = generate_raw_fixture();
    let committed = normalize_volatile(committed_raw);
    let generated = normalize_volatile(decision);
    assert_eq!(
        generated, committed,
        "decision.json fixture must match regenerated output (volatile fields normalized). \
         If the decision shape changed deliberately, re-run generate_fixtures to update."
    );
}

/// One-shot helper: generates and writes the three fixture files (raw, schema-valid).
/// Run with: cargo test --features cli --test seaforge_fixture_tests generate_fixtures -- --ignored
#[test]
#[ignore]
fn generate_fixtures() {
    let dir = conformance_dir();
    std::fs::create_dir_all(&dir).expect("create fixture dir");

    let (pack, trace, decision) = generate_raw_fixture();

    let pack_text = serde_json::to_string_pretty(&pack).unwrap();
    let trace_text = serde_json::to_string_pretty(&trace).unwrap();
    let decision_text = serde_json::to_string_pretty(&decision).unwrap();

    std::fs::write(dir.join("pack.json"), &pack_text).expect("write pack.json");
    std::fs::write(dir.join("trace.json"), &trace_text).expect("write trace.json");
    std::fs::write(dir.join("decision.json"), &decision_text).expect("write decision.json");

    let manifest = json!({
        "description": "SEA-Forge contract v1 fixture: real AuthorityPack + AuthorityTrace + AuthorityDecision that validate against schemas/seaforge-contract-v1.json. Volatile fields (timestamps, hashes, IDs) are normalized to sentinels by the pinning test for byte-stable comparison.",
        "command": "seaforge_contract",
        "pack": "pack.json",
        "trace": "trace.json",
        "decision": "decision.json",
        "schema": "../../schemas/seaforge-contract-v1.json"
    });
    let manifest_text = serde_json::to_string_pretty(&manifest).unwrap();
    std::fs::write(dir.join("manifest.json"), &manifest_text).expect("write manifest.json");

    eprintln!("Fixtures written to {}", dir.display());
    eprintln!("pack.json: {} bytes", pack_text.len());
    eprintln!("trace.json: {} bytes", trace_text.len());
    eprintln!("decision.json: {} bytes", decision_text.len());
}
