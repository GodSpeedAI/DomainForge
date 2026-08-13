//! Declared `Instance` and `Policy` declarations must survive RDF projection.
//!
//! Before this suite, `KnowledgeGraph::from_graph` iterated only entities,
//! roles, resources, patterns, relations, and flows, so every authored instance
//! and policy was silently dropped from `model.ttl`, `model.jsonld`, and
//! `ontology.owl.ttl`.

use domainforge_core::parser::parse_to_graph;
use domainforge_core::projection::rdf::project_rdf_in_memory;
use serde_json::Value;
use std::collections::BTreeMap;

const FIXED_TS: &str = "2026-07-02T00:00:00+00:00";

/// Untyped instances (no `entity` contract) — the shape SEA Forge's interaction
/// model uses.
const UNTYPED: &str = r#"
@namespace "demo"

Entity "Vendor" in demo

Instance vendor_123 of "Vendor" {
    name: "Acme Corp",
    credit_limit: 50000,
    active: true
}

Instance vendor_456 of "Vendor" {
    name: "Globex"
}

Policy vendors_are_named as:
    forall entity_instance in entity_instances: (entity_instance.name != "")
"#;

fn project(source: &str) -> BTreeMap<String, String> {
    let graph = parse_to_graph(source).expect("fixture parses");
    project_rdf_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()), None)
        .expect("projection succeeds")
}

#[test]
fn turtle_carries_every_declared_instance() {
    let ttl = project(UNTYPED).remove("model.ttl").expect("model.ttl");

    assert!(
        ttl.contains("sea:instance_vendor_123 rdf:type sea:EntityInstance"),
        "instance missing from Turtle:\n{ttl}"
    );
    assert!(ttl.contains("sea:instance_vendor_456 rdf:type sea:EntityInstance"));
    assert!(ttl.contains("sea:instance_vendor_123 sea:instanceOf sea:Vendor"));
    assert!(ttl.contains(r#"sea:instance_vendor_123 rdfs:label "vendor_123""#));
}

#[test]
fn turtle_carries_instance_field_values() {
    let ttl = project(UNTYPED).remove("model.ttl").expect("model.ttl");

    assert!(ttl.contains(r#"sea:instance_vendor_123 sea:name "Acme Corp"^^xsd:string"#));
    assert!(ttl.contains(r#"sea:instance_vendor_123 sea:active "true"^^xsd:boolean"#));
    // The parser evaluates field expressions to f64, so an authored `50000`
    // reaches the projector as `50000.0`. That is an IR representation choice,
    // not a projection loss — `"50000.0"^^xsd:decimal` is RDF-equal to `50000`.
    // The projector emits what the IR holds rather than guessing intent.
    assert!(ttl.contains(r#"sea:instance_vendor_123 sea:credit_limit "50000.0"^^xsd:decimal"#));
}

#[test]
fn turtle_carries_declared_policies() {
    let ttl = project(UNTYPED).remove("model.ttl").expect("model.ttl");

    assert!(
        ttl.contains("sea:policy_vendors_are_named rdf:type sea:Policy"),
        "policy missing from Turtle:\n{ttl}"
    );
    assert!(ttl.contains(r#"sea:policy_vendors_are_named rdfs:label "vendors_are_named""#));
    assert!(ttl.contains("sea:policy_vendors_are_named sea:expression"));
    assert!(ttl.contains(r#"sea:policy_vendors_are_named sea:priority "0"^^xsd:integer"#));
}

#[test]
fn ontology_declares_instances_and_policies_as_individuals() {
    let owl = project(UNTYPED)
        .remove("ontology.owl.ttl")
        .expect("ontology.owl.ttl");

    assert!(owl.contains("sea:EntityInstance a owl:Class"));
    assert!(owl.contains("sea:Policy a owl:Class"));
    assert!(owl.contains("sea:instanceOf a owl:ObjectProperty"));
    assert!(
        owl.contains("sea:instance_vendor_123"),
        "instance individual missing from ontology:\n{owl}"
    );
    assert!(owl.contains("sea:policy_vendors_are_named"));
}

#[test]
fn jsonld_carries_instances_with_typed_values() {
    let files = project(UNTYPED);
    let doc: Value = serde_json::from_str(&files["model.jsonld"]).expect("valid JSON");
    let graph = doc["@graph"].as_array().expect("@graph array");

    let node = graph
        .iter()
        .find(|n| n["@id"] == Value::String("sea:instance_vendor_123".to_string()))
        .expect("instance node present in JSON-LD");

    assert_eq!(
        node["sea:name"]["@value"],
        Value::String("Acme Corp".into())
    );
    assert_eq!(
        node["sea:name"]["@type"],
        Value::String("xsd:string".into())
    );
    assert_eq!(
        node["sea:credit_limit"]["@type"],
        Value::String("xsd:decimal".into())
    );
}

/// Instance fields live in a `HashMap`; unsorted iteration would make output
/// order vary run to run and break the `verify-rdf` byte-identity gate.
#[test]
fn instance_output_is_deterministic() {
    let first = project(UNTYPED);
    for _ in 0..8 {
        assert_eq!(
            project(UNTYPED),
            first,
            "RDF projection is not deterministic"
        );
    }
}

/// The name-only IRI scheme is only sound because instance names are unique
/// graph-wide. This pins that invariant: if `insert_entity_instance` ever stops
/// rejecting duplicate names, this test fails and the IRI scheme must be
/// revisited (see `KnowledgeGraph::entity_instance_iri`).
#[test]
fn instance_names_are_unique_graph_wide() {
    const DUPLICATE_NAME: &str = r#"
@namespace "demo"

Entity "Vendor" in demo
Entity "Customer" in demo

Instance acme of "Vendor" { tier: "gold" }
Instance acme of "Customer" { tier: "silver" }
"#;

    let error = parse_to_graph(DUPLICATE_NAME).expect_err("duplicate instance name is rejected");
    assert!(
        format!("{error:?}").contains("already exists"),
        "unexpected rejection reason: {error:?}"
    );
}

/// Distinct instances of different entities each keep their own identity.
#[test]
fn instances_of_different_entities_stay_distinct() {
    const TWO_KINDS: &str = r#"
@namespace "demo"

Entity "Vendor" in demo
Entity "Customer" in demo

Instance acme_vendor of "Vendor" { tier: "gold" }
Instance acme_customer of "Customer" { tier: "silver" }
"#;

    let ttl = project(TWO_KINDS).remove("model.ttl").expect("model.ttl");

    assert!(ttl.contains("sea:instance_acme_vendor sea:instanceOf sea:Vendor"));
    assert!(ttl.contains("sea:instance_acme_customer sea:instanceOf sea:Customer"));
    assert!(ttl.contains(r#"sea:instance_acme_vendor sea:tier "gold"^^xsd:string"#));
    assert!(ttl.contains(r#"sea:instance_acme_customer sea:tier "silver"^^xsd:string"#));
}

/// A model with no instances or policies must project exactly as before, so the
/// new classes never appear speculatively.
#[test]
fn model_without_instances_is_unchanged() {
    const PLAIN: &str = r#"
@namespace "demo"

Entity "Warehouse" in demo
"#;

    let files = project(PLAIN);
    assert!(!files["ontology.owl.ttl"].contains("sea:EntityInstance a owl:Class"));
    assert!(!files["ontology.owl.ttl"].contains("sea:Policy a owl:Class"));
    assert!(!files["model.ttl"].contains("sea:instance_"));
    assert!(!files["model.ttl"].contains("sea:policy_"));
}
