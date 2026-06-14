#![cfg(feature = "cli")]

use sea_core::authority::pack::compute_pack_hash;
use sea_core::authority::*;
use std::path::PathBuf;

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
                m.insert("action".to_string(), serde_json::json!("ShipOrder"));
                m.insert("resource.type".to_string(), serde_json::json!("Order"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "customer.credit_status".to_string(),
                    serde_json::json!("Hold"),
                );
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

fn make_env_config(packs: Vec<serde_json::Value>) -> AuthorityEnvironmentConfig {
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
    use chrono::Utc;
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
        context: serde_json::json!({}),
        requested_at: Utc::now(),
        correlation_id: None,
        risk_class: None,
        metadata: Default::default(),
    }
}

fn make_trusted_fact(path: &str, value: serde_json::Value) -> FactEnvelope {
    use chrono::Utc;
    FactEnvelope {
        path: path.to_string(),
        value,
        source_class: SourceClass::SystemOfRecord,
        source_id: "credit-service".to_string(),
        observed_at: Utc::now(),
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        evidence_ref: Some("ref-1".to_string()),
        signature: None,
        confidence: None,
        lineage: None,
    }
}

fn validate_against_schema(instance: &serde_json::Value, def_name: &str) -> Result<(), String> {
    let schema_text = std::fs::read_to_string(schema_path())
        .map_err(|e| format!("failed to read schema: {e}"))?;
    let schema_json: serde_json::Value =
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

#[test]
fn pack_fixture_conforms_to_schema() {
    let pack = build_prohibition_pack();
    let pack_json = serde_json::to_value(&pack).expect("serialize pack");
    validate_against_schema(&pack_json, "pack").expect("pack must conform to schema");
}

#[test]
fn trace_fixture_conforms_to_schema() {
    let pack = build_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).expect("env");
    env.validate().expect("valid");

    let request = make_request();
    let facts = vec![make_trusted_fact(
        "customer.credit_status",
        serde_json::json!("Hold"),
    )];
    let (trace, _) = env.evaluate(&request, &facts).expect("eval");

    let trace_json = serde_json::to_value(&trace).expect("serialize trace");
    validate_against_schema(&trace_json, "trace").expect("trace must conform to schema");
}

#[test]
fn decision_fixture_conforms_to_schema() {
    let pack = build_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).expect("env");
    env.validate().expect("valid");

    let request = make_request();
    let facts = vec![make_trusted_fact(
        "customer.credit_status",
        serde_json::json!("Hold"),
    )];
    let (_, decision) = env.evaluate(&request, &facts).expect("eval");

    let decision_json = serde_json::to_value(&decision).expect("serialize decision");
    validate_against_schema(&decision_json, "decision").expect("decision must conform to schema");
}

#[test]
fn schema_declares_contract_version_v1() {
    let schema_text = std::fs::read_to_string(schema_path()).expect("read schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema");
    let contract_version = schema
        .get("properties")
        .and_then(|p| p.get("contract_version"))
        .and_then(|c| c.get("const"))
        .and_then(|v| v.as_str());
    assert_eq!(
        contract_version,
        Some("v1"),
        "schema must declare contract_version const 'v1'"
    );
}
