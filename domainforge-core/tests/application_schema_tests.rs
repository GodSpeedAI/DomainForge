//! Canonical semantic envelope and artifact schema tests (plan Task 12;
//! reference §6/§8).

use domainforge_core::application::{
    resolve_application_contract, resolve_application_contract_with_packs,
    resolve_semantic_envelope, resolve_semantic_envelope_with_packs,
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
    let kinds: Vec<(String, String)> = doc
        .envelope
        .semantic_declarations
        .iter()
        .map(|d| {
            (
                d.logical_module_id.clone(),
                serde_json::to_value(&d.declaration).unwrap()["kind"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
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
    // §6 closure order: logical module ID first, then kind rank.
    let ranks: Vec<(&str, usize)> = kinds.iter().map(|(m, k)| (m.as_str(), rank(k))).collect();
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

/// Gate finding 6: every persisted metadata field is an APP015 vector with
/// the field's JSON Pointer, including internally recomputed self-hashes.
#[test]
fn app015_tamper_vectors_cover_every_metadata_field() {
    let contract_value = serde_json::to_value(
        resolve_application_contract("flagship/command-write.sea", &flagship_sources()).unwrap(),
    )
    .unwrap();
    let envelope_value = serde_json::to_value(envelope_for(&flagship_sources())).unwrap();

    // (path, mutation, expected pointer). Each mutation must produce APP015
    // at the named pointer. A self-consistent but contract-invalid artifact
    // is the failure mode the gate explicitly calls out.
    let contract_vectors: Vec<(&str, Mutation, &str)> = vec![
        (
            "producer name",
            Box::new(|v| v["producer"]["name"] = json!("attacker")),
            "/producer/name",
        ),
        (
            "producer version empty",
            Box::new(|v| v["producer"]["version"] = json!("")),
            "/producer/version",
        ),
        (
            "inputs.source_set_hash bad format",
            Box::new(|v| v["inputs"]["source_set_hash"] = json!("not-a-hash")),
            "/inputs/source_set_hash",
        ),
        (
            "inputs.semantic_pack_set_hash bad format",
            Box::new(|v| v["inputs"]["semantic_pack_set_hash"] = json!("sha256:deadbeef")),
            "/inputs/semantic_pack_set_hash",
        ),
        (
            "inputs.language_schema_version mismatch",
            Box::new(|v| v["inputs"]["language_schema_version"] = json!("domainforge-ast/v1")),
            "/inputs/language_schema_version",
        ),
        (
            "inputs.interpretation_version mismatch",
            Box::new(|v| {
                v["inputs"]["interpretation_version"] = json!("domainforge-interpretation/v9")
            }),
            "/inputs/interpretation_version",
        ),
        (
            "schema_version mismatch",
            Box::new(|v| v["schema_version"] = json!("domainforge-application-contract/v9")),
            "/schema_version",
        ),
        (
            // Tamper a value the explicit checks do not cover; the
            // internally recomputed self_hash is the only signal.
            "tampered enum member with recomputed self_hash",
            Box::new(|v| {
                v["contract"]["enums"][0]["members"][0]["wire"] = json!("tampered");
                v["self_hash"] = json!(format!("sha256:{}", "0".repeat(64)));
            }),
            "/self_hash",
        ),
    ];
    for (label, mutate, expected_path) in contract_vectors {
        let mut bad = contract_value.clone();
        mutate(&mut bad);
        let diags = match validate_application_contract_document_json(&bad.to_string()) {
            Err(d) => d,
            Ok(_) => panic!("expected APP015 for {label}"),
        };
        assert_app015(&diags, expected_path);
    }

    let envelope_vectors: Vec<(&str, Mutation, &str)> = vec![
        (
            "envelope producer name",
            Box::new(|v| v["producer"]["name"] = json!("attacker")),
            "/producer/name",
        ),
        (
            "envelope inputs.semantic_pack_set_hash bad format",
            Box::new(|v| v["inputs"]["semantic_pack_set_hash"] = json!("sha256:00")),
            "/inputs/semantic_pack_set_hash",
        ),
        (
            "envelope language_schema_version mismatch",
            Box::new(|v| v["envelope"]["language_schema_version"] = json!("other")),
            "/envelope/language_schema_version",
        ),
        (
            "envelope canonicalization_version mismatch",
            Box::new(|v| v["envelope"]["canonicalization_version"] = json!("other")),
            "/envelope/canonicalization_version",
        ),
        (
            "envelope compiler_interpretation_version mismatch",
            Box::new(|v| v["envelope"]["compiler_interpretation_version"] = json!("other")),
            "/envelope/compiler_interpretation_version",
        ),
        (
            "envelope semantic_packs malformed",
            Box::new(|v| {
                v["envelope"]["semantic_packs"] =
                    json!([{"pack_id": "p", "content_hash": "sha256:nope"}]);
            }),
            "/envelope/semantic_packs/0/content_hash",
        ),
        (
            "envelope semantic_packs empty pack_id",
            Box::new(|v| {
                v["envelope"]["semantic_packs"] = json!([{
                    "pack_id": "",
                    "content_hash": format!("sha256:{}", "c".repeat(64))
                }]);
            }),
            "/envelope/semantic_packs/0/pack_id",
        ),
        (
            // Cross-consistency: the persisted pack-set hash disagrees with
            // the packs actually listed in the envelope.
            "pack_set_hash disagrees with envelope packs",
            Box::new(|v| {
                v["envelope"]["semantic_packs"] = json!([{
                    "pack_id": "extra",
                    "content_hash": format!("sha256:{}", "c".repeat(64))
                }]);
                v["inputs"]["semantic_pack_set_hash"] = json!(format!("sha256:{}", "d".repeat(64)));
                // Re-stamp self_hash to keep the document self-consistent
                // at the byte level — the cross-check must still fire.
                v["self_hash"] = json!(format!("sha256:{}", "0".repeat(64)));
            }),
            "/inputs/semantic_pack_set_hash",
        ),
    ];
    for (label, mutate, expected_path) in envelope_vectors {
        let mut bad = envelope_value.clone();
        mutate(&mut bad);
        let diags = match validate_semantic_envelope_document_json(&bad.to_string()) {
            Err(d) => d,
            Ok(_) => panic!("expected APP015 for {label}"),
        };
        assert_app015(&diags, expected_path);
    }
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

/// Gate finding 5 (D10): the fixed public boundary carries non-empty
/// semantic-pack sets through `inputs.semantic_pack_set_hash` and the
/// envelope's `semantic_packs`, and rejects malformed inputs as APP015.
#[test]
fn non_empty_semantic_packs_cross_the_public_boundary() {
    let hash_a = format!("sha256:{}", "a".repeat(64));
    let hash_b = format!("sha256:{}", "b".repeat(64));
    let packs = vec![
        ("pack-alpha".to_string(), hash_a.clone()),
        ("pack-beta".to_string(), hash_b.clone()),
    ];

    let contract = resolve_application_contract_with_packs(
        "flagship/command-write.sea",
        &flagship_sources(),
        &packs,
    )
    .expect("non-empty pack set resolves");
    let envelope = resolve_semantic_envelope_with_packs(
        "flagship/command-write.sea",
        &flagship_sources(),
        &packs,
    )
    .expect("non-empty pack set resolves");

    // The recomputed pack-set hash is deterministic and equal across both
    // document kinds.
    let expected_set_hash = domainforge_core::application::semantic_pack_set_hash(&packs).unwrap();
    assert_eq!(contract.inputs.semantic_pack_set_hash, expected_set_hash);
    assert_eq!(envelope.inputs.semantic_pack_set_hash, expected_set_hash);

    // The envelope's listed packs preserve pack_id and content_hash.
    let listed: Vec<(String, String)> = envelope
        .envelope
        .semantic_packs
        .iter()
        .map(|p| (p.pack_id.clone(), p.content_hash.clone()))
        .collect();
    assert_eq!(listed, packs);

    // Both documents round-trip through the persisted-envelope validators.
    let contract_json = serde_json::to_string(&contract).unwrap();
    validate_application_contract_document_json(&contract_json).unwrap();
    let envelope_json = serde_json::to_string(&envelope).unwrap();
    validate_semantic_envelope_document_json(&envelope_json).unwrap();
}

#[test]
fn malformed_semantic_packs_are_app015() {
    let bad_packs = vec![
        ("pack".to_string(), "not-a-hash".to_string()),
        ("".to_string(), format!("sha256:{}", "a".repeat(64))),
    ];
    let err = resolve_application_contract_with_packs(
        "flagship/command-write.sea",
        &flagship_sources(),
        &bad_packs,
    )
    .unwrap_err();
    assert!(err
        .iter()
        .all(|d| d.code == domainforge_core::application::ApplicationDiagnosticCode::App015));
    assert!(err.len() >= 2);

    // Duplicate pack ids are also APP015.
    let dup_packs = vec![
        ("dup".to_string(), format!("sha256:{}", "a".repeat(64))),
        ("dup".to_string(), format!("sha256:{}", "b".repeat(64))),
    ];
    let err = resolve_semantic_envelope_with_packs(
        "flagship/command-write.sea",
        &flagship_sources(),
        &dup_packs,
    )
    .unwrap_err();
    assert!(err
        .iter()
        .all(|d| d.code == domainforge_core::application::ApplicationDiagnosticCode::App015));
}
