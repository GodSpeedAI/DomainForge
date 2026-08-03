use domainforge_core::application::{resolve_application_graph, ApplicationDiagnostic};
use serde_json::json;

fn resolve(source: &str) -> Result<domainforge_core::Graph, Vec<ApplicationDiagnostic>> {
    resolve_application_graph("main.sea", &json!({ "main.sea": source }).to_string())
}

fn typed_model(instances: &str) -> String {
    format!(
        r#"@namespace "interaction"
pattern "Slug" matches "^[a-z0-9-]+$"
enum Maturity {{ specified = "specified", exercised = "exercised" }}
entity "Journey" {{
    key journey_id: string (min_length 1, max_length 8, pattern Slug)
    maturity: Maturity
    attempts: int (min 1, exclusive_max 10)
    score: decimal (exclusive_min 0, max 1)
    enabled: bool
    observed_at: timestamp
    trace_id: uuid
    notes: string optional
    tags: list<string> optional (min_items 1, max_items 2)
}}
entity "Binding" {{
    key binding_id: string
    journey: ref<Journey>
}}
{instances}
"#
    )
}

fn journey(name: &str, id: &str, maturity: &str, attempts: &str) -> String {
    format!(
        r#"instance {name} of "Journey" {{
    journey_id: "{id}",
    maturity: "{maturity}",
    attempts: {attempts},
    score: 0.75,
    enabled: true,
    observed_at: "2026-08-03T12:00:00Z",
    trace_id: "018f47a6-7b2d-7b62-8a00-000000000001"
}}
"#
    )
}

fn error_text(result: Result<domainforge_core::Graph, Vec<ApplicationDiagnostic>>) -> String {
    let diagnostics = result.expect_err("typed instance data should be rejected");
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn valid_typed_instances_preserve_contract_and_allow_optional_omission() {
    let source = typed_model(&format!(
        "{}\ninstance binding of \"Binding\" {{ binding_id: \"primary\", journey: \"cj01\" }}",
        journey("journey", "cj01", "exercised", "2")
    ));

    let graph = resolve(&source).expect("valid typed instances resolve");
    let journey_entity = graph
        .all_entities()
        .into_iter()
        .find(|entity| entity.name() == "Journey")
        .expect("Journey entity exists");
    let graph_json = serde_json::to_value(&graph).expect("graph serializes");

    assert_eq!(graph.entity_instance_count(), 2);
    assert_eq!(
        graph_json["entity_contracts"][journey_entity.id().to_string()]["key_field"],
        "journey_id"
    );
    assert_eq!(
        graph_json["entity_contracts"][journey_entity.id().to_string()]["fields"]
            .as_array()
            .expect("fields remain in graph")
            .len(),
        9
    );
}

#[test]
fn forward_typed_reference_resolves_against_complete_instance_set() {
    let source = typed_model(&format!(
        "instance binding of \"Binding\" {{ binding_id: \"primary\", journey: \"cj01\" }}\n{}",
        journey("journey", "cj01", "exercised", "2")
    ));

    assert_eq!(resolve(&source).unwrap().entity_instance_count(), 2);
}

#[test]
fn ordinary_parser_preserves_typed_entity_contracts() {
    let source = typed_model(&journey("journey", "cj01", "specified", "2"));
    let graph = domainforge_core::parse_to_graph(&source)
        .expect("ordinary parser resolves typed entity contracts");
    let entity_id = graph
        .find_entity_by_name_and_namespace("Journey", "interaction")
        .expect("Journey entity exists");

    let contract = graph
        .entity_contract(&entity_id)
        .expect("typed entity contract remains in Graph");
    assert_eq!(contract.key_field, "journey_id");
    assert_eq!(contract.fields.len(), 9);
}

#[test]
fn missing_required_field_is_rejected() {
    let source = typed_model(
        r#"instance journey of "Journey" {
    journey_id: "cj01",
    maturity: "exercised",
    attempts: 2,
    score: 0.75,
    observed_at: "2026-08-03T12:00:00Z",
    trace_id: "018f47a6-7b2d-7b62-8a00-000000000001"
}"#,
    );

    let error = error_text(resolve(&source));
    assert!(error.contains("journey") && error.contains("enabled") && error.contains("required"));
}

#[test]
fn unknown_typed_entity_field_is_rejected() {
    let instance = journey("journey", "cj01", "specified", "2").replacen(
        "maturity: \"specified\",",
        "maturity: \"specified\",\n    surprise: true,",
        1,
    );

    let error = error_text(resolve(&typed_model(&instance)));
    assert!(error.contains("journey") && error.contains("surprise") && error.contains("unknown"));
}

#[test]
fn invalid_enum_member_is_rejected() {
    let source = typed_model(&journey("journey", "cj01", "unknown", "2"));

    let error = error_text(resolve(&source));
    assert!(error.contains("journey") && error.contains("maturity") && error.contains("unknown"));
}

#[test]
fn duplicate_entity_key_is_rejected() {
    let source = typed_model(&format!(
        "{}\n{}",
        journey("journey_one", "cj01", "specified", "1"),
        journey("journey_two", "cj01", "exercised", "2")
    ));

    let error = error_text(resolve(&source));
    assert!(error.contains("journey_id") && error.contains("cj01") && error.contains("unique"));
}

#[test]
fn dangling_typed_reference_is_rejected() {
    let source = typed_model(
        r#"instance binding of "Binding" {
    binding_id: "primary",
    journey: "missing"
}"#,
    );

    let error = error_text(resolve(&source));
    assert!(error.contains("binding") && error.contains("journey") && error.contains("missing"));
}

#[test]
fn scalar_type_and_supported_constraints_are_enforced() {
    for (field, replacement, fragment) in [
        ("attempts", "attempts: \"two\"", "int"),
        ("attempts", "attempts: 0", "min"),
        ("attempts", "attempts: 10", "exclusive_max"),
        ("score", "score: 0", "exclusive_min"),
        ("score", "score: 1.1", "max"),
        ("score", "score: \"0.75\"", "decimal"),
        ("journey_id", "journey_id: \"TOO-LONG-ID\"", "max_length"),
        ("journey_id", "journey_id: \"BAD!\"", "pattern"),
        ("enabled", "enabled: \"yes\"", "bool"),
        ("observed_at", "observed_at: \"not-a-time\"", "timestamp"),
        ("trace_id", "trace_id: \"not-a-uuid\"", "uuid"),
    ] {
        let mut instance = journey("journey", "cj01", "specified", "2");
        let original = match field {
            "attempts" => "attempts: 2",
            "score" => "score: 0.75",
            "journey_id" => "journey_id: \"cj01\"",
            "enabled" => "enabled: true",
            "observed_at" => "observed_at: \"2026-08-03T12:00:00Z\"",
            "trace_id" => "trace_id: \"018f47a6-7b2d-7b62-8a00-000000000001\"",
            _ => unreachable!(),
        };
        instance = instance.replacen(original, replacement, 1);

        let error = error_text(resolve(&typed_model(&instance)));
        assert!(
            error.contains(field) && error.contains(fragment),
            "expected {field}/{fragment}, got {error}"
        );
    }
}

#[test]
fn list_element_and_item_count_constraints_are_enforced() {
    let source = typed_model(&journey("journey", "cj01", "specified", "2"));
    let base = resolve(&source).expect("base typed graph resolves");

    for (tags, fragment) in [
        (json!([]), "min_items"),
        (json!(["one", "two", "three"]), "max_items"),
        (json!([1]), "string"),
    ] {
        let mut graph_json = serde_json::to_value(&base).expect("graph serializes");
        let instance = graph_json["entity_instances"]
            .as_object_mut()
            .and_then(|instances| instances.values_mut().next())
            .expect("journey instance is serialized");
        instance["fields"]["tags"] = tags;
        let graph: domainforge_core::Graph =
            serde_json::from_value(graph_json).expect("graph deserializes");
        let errors = graph
            .validate_entity_instances()
            .expect_err("invalid list must fail validation")
            .join("\n");
        assert!(
            errors.contains("tags") && errors.contains(fragment),
            "{errors}"
        );
    }
}

#[test]
fn bodyless_untyped_entities_keep_accepting_free_form_instances() {
    let graph = resolve(
        r#"@namespace "legacy"
entity "Vendor"
instance vendor of "Vendor" { arbitrary: "still accepted" }
"#,
    )
    .expect("legacy schemaless instance remains valid");
    let json = serde_json::to_value(&graph).expect("legacy graph serializes");

    assert_eq!(graph.entity_instance_count(), 1);
    assert!(json.get("entity_contracts").is_none());
    assert!(json.get("enum_contracts").is_none());
}
