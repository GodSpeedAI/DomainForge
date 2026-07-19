//! Canonical semantic envelope and artifact schema tests (plan Task 12;
//! reference §6/§8).

use domainforge_core::application::{
    resolve_application_contract, resolve_semantic_envelope,
    validate_application_contract_document_json, validate_semantic_envelope_document_json,
    CanonicalSemanticEnvelopeDocument,
};
use serde_json::json;

fn flagship_sources() -> String {
    json!({
        "flagship/command-write.sea": include_str!(
            "../../fixtures/application_generation/flagship/command-write.sea"),
        "flagship/query-read.sea": include_str!(
            "../../fixtures/application_generation/flagship/query-read.sea"),
    })
    .to_string()
}

fn envelope_for(sources: &str) -> CanonicalSemanticEnvelopeDocument {
    resolve_semantic_envelope("flagship/command-write.sea", sources).unwrap()
}

#[test]
fn envelope_hash_ignores_formatting_but_covers_import_meaning() {
    let a = envelope_for(&flagship_sources());
    // Reformat: extra blank lines and a comment change no semantics.
    let reformatted = flagship_sources().replace(
        "operation place_order {",
        "\\n\\n// canonical\\noperation place_order {",
    );
    let b = envelope_for(&reformatted);
    assert_eq!(a.semantic_closure_hash, b.semantic_closure_hash);
    // Changing the policy limit changes semantic meaning.
    let changed = flagship_sources().replace("total <= 10000", "total <= 20000");
    let c = envelope_for(&changed);
    assert_ne!(a.semantic_closure_hash, c.semantic_closure_hash);
}

#[test]
fn contract_document_shares_the_envelope_closure_hash() {
    let contract =
        resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap();
    let envelope = envelope_for(&flagship_sources());
    assert_eq!(
        contract.semantic_closure_hash,
        envelope.semantic_closure_hash
    );
    assert!(contract.semantic_closure_hash.starts_with("sha256:"));
    assert_ne!(
        contract.semantic_closure_hash,
        format!("sha256:{}", "0".repeat(64))
    );
}

#[test]
fn declarations_follow_kind_rank_and_modules_hash_semantics_only() {
    // The query module imports the command module, so its closure has both.
    let doc = resolve_semantic_envelope("flagship/query-read.sea", &flagship_sources()).unwrap();
    let kinds: Vec<String> = doc
        .envelope
        .semantic_declarations
        .iter()
        .map(|d| {
            serde_json::to_value(&d.declaration).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let rank = |k: &str| {
        [
            "dimension",
            "unit",
            "record",
            "enum",
            "operation",
            "entity",
            "resource",
            "flow",
            "pattern",
            "role",
            "relation",
            "instance",
            "policy",
            "concept_change",
            "metric",
        ]
        .iter()
        .position(|x| x == &k)
        .unwrap()
    };
    let ranks: Vec<usize> = kinds.iter().map(|k| rank(k)).collect();
    let mut sorted = ranks.clone();
    sorted.sort();
    assert_eq!(ranks, sorted);
    // Modules carry semantic content hashes and namespaces.
    assert_eq!(doc.envelope.modules.len(), 2);
    for module in &doc.envelope.modules {
        assert!(module.semantic_content_hash.starts_with("sha256:"));
        assert_eq!(module.namespace, "flagship.orders");
    }
    // The import edge from the query module survives into the envelope.
    assert!(doc
        .envelope
        .import_graph
        .iter()
        .any(|e| e.importer == "flagship/query-read.sea"));
}

#[test]
fn flow_occurrences_get_duplicate_indexes() {
    let sources = json!({
        "main.sea": "@namespace \"t\"\nentity \"A\"\nentity \"B\"\nresource \"R\" kg in logistics\nflow \"R\" from \"A\" to \"B\" quantity 1\nflow \"R\" from \"A\" to \"B\" quantity 1\n"
    })
    .to_string();
    let doc = resolve_semantic_envelope("main.sea", &sources).unwrap();
    let mut indexes: Vec<u32> = doc
        .envelope
        .semantic_declarations
        .iter()
        .filter_map(|d| {
            let v = serde_json::to_value(&d.id).unwrap();
            (v["kind"] == "occurrence")
                .then(|| v["data"]["duplicate_index"].as_u64().unwrap() as u32)
        })
        .collect();
    indexes.sort();
    assert_eq!(indexes, [0, 1]);
}

#[test]
fn instance_ids_are_deterministic_concept_ids() {
    use domainforge_core::concept_id::ConceptId;
    let sources = json!({
        "main.sea": "@namespace \"t\"\nentity \"Tank\"\ninstance tank1 of \"Tank\"\n"
    })
    .to_string();
    let doc = resolve_semantic_envelope("main.sea", &sources).unwrap();
    let expected = serde_json::to_value(ConceptId::from_concept("t", "Tank:tank1")).unwrap();
    assert!(doc.envelope.semantic_declarations.iter().any(|d| {
        let v = serde_json::to_value(&d.id).unwrap();
        v["kind"] == "concept" && v["data"]["id"] == expected
    }));
}

#[test]
fn resolved_references_use_canonical_field_paths() {
    let doc = envelope_for(&flagship_sources());
    let paths: Vec<&str> = doc
        .envelope
        .resolved_references
        .iter()
        .map(|r| r.field_path.as_str())
        .collect();
    assert!(paths.contains(&"fields.total.field_type.unit"));
    assert!(paths.contains(&"input"));
    assert!(paths.contains(&"state"));
    assert!(paths.contains(&"actor.role"));
    assert!(paths.contains(&"access.bindings.0.policy"));
}

// ---- APP015 artifact validation ----

#[test]
fn valid_documents_round_trip_through_validation() {
    let contract_json = serde_json::to_string(
        &resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap(),
    )
    .unwrap();
    validate_application_contract_document_json(&contract_json).unwrap();

    let envelope_json = serde_json::to_string(&envelope_for(&flagship_sources())).unwrap();
    validate_semantic_envelope_document_json(&envelope_json).unwrap();
}

fn assert_app015(diags: &[domainforge_core::application::ApplicationDiagnostic], path: &str) {
    assert!(
        diags.iter().any(|d| {
            serde_json::to_value(d.code).unwrap() == "APP015"
                && d.context.field_path.as_deref() == Some(path)
        }),
        "expected APP015 at '{path}', got {diags:?}"
    );
}

#[test]
fn unknown_fields_and_wrong_versions_are_app015() {
    let doc =
        resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap();
    let mut value = serde_json::to_value(&doc).unwrap();

    // Unknown top-level field.
    let mut with_unknown = value.clone();
    with_unknown["surprise"] = json!(1);
    let err = validate_application_contract_document_json(&with_unknown.to_string()).unwrap_err();
    assert_app015(&err, "");

    // Wrong schema version.
    value["schema_version"] = json!("domainforge-application-contract/v9");
    let err = validate_application_contract_document_json(&value.to_string()).unwrap_err();
    assert_app015(&err, "/schema_version");
}

#[test]
fn bad_hashes_are_app015_with_json_pointers() {
    let doc =
        resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap();

    // Tampered self-hash (valid format, wrong bytes).
    let mut tampered = serde_json::to_value(&doc).unwrap();
    tampered["self_hash"] = json!(format!("sha256:{}", "a".repeat(64)));
    let err = validate_application_contract_document_json(&tampered.to_string()).unwrap_err();
    assert_app015(&err, "/self_hash");

    // Invalid hash prefix/hex.
    let mut invalid = serde_json::to_value(&doc).unwrap();
    invalid["inputs"]["source_set_hash"] = json!("md5:abc");
    let err = validate_application_contract_document_json(&invalid.to_string()).unwrap_err();
    assert_app015(&err, "/inputs/source_set_hash");

    // Tampered envelope closure hash.
    let mut env = serde_json::to_value(envelope_for(&flagship_sources())).unwrap();
    env["semantic_closure_hash"] = json!(format!("sha256:{}", "b".repeat(64)));
    let err = validate_semantic_envelope_document_json(&env.to_string()).unwrap_err();
    assert_app015(&err, "/semantic_closure_hash");
}

#[test]
fn documents_satisfy_their_published_json_schemas() {
    let contract_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../schemas/application-contract-v1.schema.json"
    ))
    .unwrap();
    let envelope_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../schemas/canonical-semantic-envelope-v1.schema.json"
    ))
    .unwrap();
    let contract = serde_json::to_value(
        resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap(),
    )
    .unwrap();
    let envelope = serde_json::to_value(envelope_for(&flagship_sources())).unwrap();

    let compiled = jsonschema::JSONSchema::compile(&contract_schema).unwrap();
    assert!(compiled.is_valid(&contract), "contract violates its schema");
    let compiled = jsonschema::JSONSchema::compile(&envelope_schema).unwrap();
    assert!(compiled.is_valid(&envelope), "envelope violates its schema");
    // The schemas are strict at the document level.
    let mut bad = contract.clone();
    bad["extra"] = json!(true);
    let compiled = jsonschema::JSONSchema::compile(&contract_schema).unwrap();
    assert!(!compiled.is_valid(&bad));
}

type Mutation = Box<dyn Fn(&mut serde_json::Value)>;

/// Gate finding 5: every array member and nested object is typed; malformed
/// scalars, unknown fields, missing fields, and wrong types must not validate.
#[test]
fn strict_schemas_reject_malformed_nested_documents() {
    let contract_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../schemas/application-contract-v1.schema.json"
    ))
    .unwrap();
    let envelope_schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../schemas/canonical-semantic-envelope-v1.schema.json"
    ))
    .unwrap();
    let contract = serde_json::to_value(
        resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap(),
    )
    .unwrap();
    let envelope = serde_json::to_value(envelope_for(&flagship_sources())).unwrap();

    // (mutation label, pointer-applied mutation) pairs against the contract.
    let contract_vectors: Vec<(&str, Mutation)> = vec![
        (
            "scalar enum member",
            Box::new(|v| v["contract"]["enums"][0] = json!(42)),
        ),
        (
            "unknown nested field",
            Box::new(|v| v["contract"]["records"][0]["surprise"] = json!(1)),
        ),
        (
            "missing operation field",
            Box::new(|v| {
                v["contract"]["operations"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("intent");
            }),
        ),
        (
            "wrong scalar type",
            Box::new(|v| v["contract"]["enums"][0]["members"][0]["wire"] = json!(5)),
        ),
        (
            "bad field_type kind",
            Box::new(|v| {
                v["contract"]["records"][0]["fields"][0]["field_type"] =
                    json!({"kind": "bogus", "data": {}});
            }),
        ),
        (
            "non-uuid concept id",
            Box::new(|v| v["contract"]["entities"][0]["concept_id"] = json!("Order")),
        ),
        (
            "unclosed effect enum",
            Box::new(|v| v["contract"]["operations"][0]["effect"] = json!("destroys")),
        ),
        (
            "malformed constraint member",
            Box::new(|v| {
                v["contract"]["records"][0]["fields"][0]["constraints"] = json!([{"kind": "min"}]);
            }),
        ),
    ];
    let compiled = jsonschema::JSONSchema::compile(&contract_schema).unwrap();
    assert!(compiled.is_valid(&contract));
    for (label, mutate) in contract_vectors {
        let mut bad = contract.clone();
        mutate(&mut bad);
        assert!(
            !compiled.is_valid(&bad),
            "contract schema accepted: {label}"
        );
    }

    let envelope_vectors: Vec<(&str, Mutation)> = vec![
        (
            "scalar declaration member",
            Box::new(|v| v["envelope"]["semantic_declarations"][0] = json!("junk")),
        ),
        (
            "bad module hash",
            Box::new(|v| {
                v["envelope"]["modules"][0]["semantic_content_hash"] = json!("sha256:XYZ");
            }),
        ),
        (
            "empty import edge",
            Box::new(|v| v["envelope"]["import_graph"] = json!([{}])),
        ),
        (
            "malformed pack ref",
            Box::new(|v| v["envelope"]["semantic_packs"] = json!([{"pack_id": "p"}])),
        ),
        (
            "untyped namespace binding",
            Box::new(|v| v["envelope"]["namespace_bindings"] = json!([17])),
        ),
        (
            "reference without target",
            Box::new(|v| {
                v["envelope"]["resolved_references"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("target");
            }),
        ),
        (
            "malformed nested contract",
            Box::new(|v| {
                v["envelope"]["application_contract"]["operations"] = json!([{"id": "x"}]);
            }),
        ),
        (
            "unknown declaration kind",
            Box::new(|v| {
                v["envelope"]["semantic_declarations"][0]["declaration"]["kind"] = json!("mystery");
            }),
        ),
    ];
    let compiled = jsonschema::JSONSchema::compile(&envelope_schema).unwrap();
    assert!(compiled.is_valid(&envelope));
    for (label, mutate) in envelope_vectors {
        let mut bad = envelope.clone();
        mutate(&mut bad);
        assert!(
            !compiled.is_valid(&bad),
            "envelope schema accepted: {label}"
        );
    }
}
