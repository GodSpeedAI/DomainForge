//! Public wire-shape tests for the application contract types (reference §8)
//! and the APP diagnostic model (reference §5).

use domainforge_core::application::{
    ApplicationDiagnostic, ApplicationSymbolId, FieldContract, FieldType, ScalarType,
};

#[test]
fn public_enums_use_adjacent_tags_and_reject_unknown_fields() {
    let ty = FieldType::Scalar {
        scalar: ScalarType::String,
    };
    assert_eq!(
        serde_json::to_value(&ty).unwrap(),
        serde_json::json!({ "kind": "scalar", "data": { "scalar": "string" } })
    );
    let bad = r#"{"kind":"scalar","data":{"scalar":"string","extra":1}}"#;
    assert!(serde_json::from_str::<FieldType>(bad).is_err());
}

#[test]
fn symbol_id_is_a_transparent_string() {
    let id = ApplicationSymbolId("flagship.orders.record.PlaceOrderInput".to_string());
    assert_eq!(
        serde_json::to_value(&id).unwrap(),
        serde_json::json!("flagship.orders.record.PlaceOrderInput")
    );
}

#[test]
fn absent_options_are_omitted_and_required_booleans_are_explicit() {
    let field = FieldContract {
        name: "order_id".to_string(),
        field_type: FieldType::Scalar {
            scalar: ScalarType::Uuid,
        },
        optional: false,
        constraints: vec![],
        default: None,
    };
    let value = serde_json::to_value(&field).unwrap();
    let obj = value.as_object().unwrap();
    assert!(
        !obj.contains_key("default"),
        "absent Option must be omitted"
    );
    assert_eq!(obj["optional"], serde_json::json!(false));
    assert_eq!(obj["constraints"], serde_json::json!([]));
    // Unknown fields on public structs are rejected.
    let bad = r#"{"name":"x","field_type":{"kind":"scalar","data":{"scalar":"string"}},"optional":false,"constraints":[],"surprise":1}"#;
    assert!(serde_json::from_str::<FieldContract>(bad).is_err());
}

#[test]
fn app014_diagnostic_carries_reason_and_source_coordinates() {
    let mut diag = ApplicationDiagnostic::closure_error(
        "not_exported",
        "'Order' exists in 'b.sea' but is not exported",
    )
    .at("a.sea", 3, 1);
    diag.context.expected = Some("an exported declaration named 'Order'".to_string());
    diag.context.actual = Some("unexported declaration 'Order'".to_string());
    diag.context.remediation = Some("add `export` to the declaration in b.sea".to_string());

    let value = serde_json::to_value(&diag).unwrap();
    assert_eq!(value["code"], "APP014");
    assert_eq!(value["slug"], "closure_resolution_error");
    let ctx = value["context"].as_object().unwrap();
    for key in [
        "reason",
        "expected",
        "actual",
        "remediation",
        "logical_module_id",
        "line",
        "column",
    ] {
        assert!(ctx.contains_key(key), "context must contain {key}");
    }
    assert_eq!(ctx["reason"], "not_exported");
    // Inapplicable values are omitted, not serialized as empty strings.
    assert!(!ctx.contains_key("document_kind"));
}

#[test]
fn hashes_carry_the_sha256_prefix() {
    use domainforge_core::application::{
        ApplicationContract, ApplicationContractDocument, ApplicationContractInputs,
        ProducerIdentity,
    };
    let doc = ApplicationContractDocument {
        schema_version: "domainforge-application-contract/v1".to_string(),
        producer: ProducerIdentity {
            name: "domainforge-core".to_string(),
            version: "0.13.0".to_string(),
        },
        inputs: ApplicationContractInputs {
            source_set_hash: format!("sha256:{}", "0".repeat(64)),
            semantic_pack_set_hash: format!("sha256:{}", "0".repeat(64)),
            language_schema_version: "3".to_string(),
            interpretation_version: "1".to_string(),
        },
        self_hash: format!("sha256:{}", "0".repeat(64)),
        semantic_closure_hash: format!("sha256:{}", "0".repeat(64)),
        contract: ApplicationContract::default(),
    };
    let value = serde_json::to_value(&doc).unwrap();
    assert!(value["inputs"]["source_set_hash"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(value["self_hash"].as_str().unwrap().starts_with("sha256:"));
    let round: ApplicationContractDocument =
        serde_json::from_value(value).expect("document round-trips");
    assert_eq!(round, doc);
}
