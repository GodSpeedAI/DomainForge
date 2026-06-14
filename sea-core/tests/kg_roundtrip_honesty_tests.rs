#![cfg(feature = "cli")]

use sea_core::kg::KnowledgeGraph;
use sea_core::parser::parse_to_graph;

fn conformance_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sea-core has parent workspace dir")
        .join("conformance")
        .join("10_kg_roundtrip")
}

#[test]
fn kg_export_then_import_preserves_supported_constructs() {
    let input_path = conformance_dir().join("input.sea");
    let source = std::fs::read_to_string(&input_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", input_path.display()));

    let graph = parse_to_graph(&source).expect("parsed input.sea");

    let entity_count_before = graph.entity_count();
    let resource_count_before = graph.resource_count();
    let flow_count_before = graph.flow_count();

    let kg = KnowledgeGraph::from_graph(&graph).expect("exported to KG");
    let turtle = kg.to_turtle();

    let turtle_no_policy = strip_unsupported_constructs(&turtle);
    let reimported =
        KnowledgeGraph::from_turtle(&turtle_no_policy).expect("parsed surviving turtle");
    let result = reimported
        .to_graph()
        .expect("reconstructed surviving graph");

    assert_eq!(
        result.entity_count(),
        entity_count_before,
        "entity count must survive round-trip"
    );
    assert_eq!(
        result.resource_count(),
        resource_count_before,
        "resource count must survive round-trip"
    );
    assert_eq!(
        result.flow_count(),
        flow_count_before,
        "flow count must survive round-trip"
    );
}

#[test]
fn kg_import_errors_on_unsupported_construct() {
    let turtle_with_role = r#"
@prefix sea: <http://domainforge.ai/sea#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

sea:Payer rdf:type sea:Role .
sea:Payer sea:namespace "default" .
"#;
    let kg = KnowledgeGraph::from_turtle(turtle_with_role).expect("turtle parses to KG");
    let result = kg.to_graph();
    assert!(
        result.is_err(),
        "import must error loudly on sea:Role, not silently drop it"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("sea:Role"),
        "error must name the unsupported construct; got: {err_msg}"
    );
    assert!(
        err_msg.contains("kg_projection_loss"),
        "error must reference the loss manifest; got: {err_msg}"
    );
}

#[test]
fn kg_import_errors_on_multiple_unsupported_constructs() {
    let turtle = r#"
@prefix sea: <http://domainforge.ai/sea#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

sea:Warehouse rdf:type sea:Entity .
sea:Widget rdf:type sea:Resource .
sea:Widget sea:unit "units" .
sea:Reviewer rdf:type sea:Role .
sea:Delivery rdf:type sea:Relation .
"#;
    let kg = KnowledgeGraph::from_turtle(turtle).expect("turtle parses to KG");
    let result = kg.to_graph();
    assert!(
        result.is_err(),
        "import must error when unsupported constructs are present alongside supported ones"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("sea:Role") && err_msg.contains("sea:Relation"),
        "error must name ALL unsupported constructs; got: {err_msg}"
    );
}

fn strip_unsupported_constructs(turtle: &str) -> String {
    turtle
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.contains("rdf:type sea:Role")
                || trimmed.contains("rdf:type sea:Relation")
                || trimmed.contains("rdf:type sea:Pattern")
                || trimmed.contains("rdf:type sea:Policy")
                || trimmed.contains("rdf:type sea:Unit"))
        })
        .collect::<Vec<&str>>()
        .join("\n")
}
