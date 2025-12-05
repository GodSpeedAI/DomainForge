#[cfg(feature = "cli")]
use assert_cmd::Command;
#[cfg(feature = "cli")]
use predicates::prelude::*;
#[cfg(feature = "shacl")]
use sea_core::ImportError;
#[cfg(feature = "cli")]
use std::fs::write;
#[cfg(feature = "cli")]
use std::path::PathBuf;
#[cfg(feature = "cli")]
use tempfile::tempdir;

#[cfg(feature = "cli")]
fn get_sea_binary() -> String {
    std::env::var("CARGO_BIN_EXE_sea").unwrap_or_else(|_| {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../target");
        if cfg!(debug_assertions) {
            path.push("debug");
        } else {
            path.push("release");
        }
        let binary_name = if cfg!(windows) { "sea.exe" } else { "sea" };
        path.push(binary_name);
        path.to_string_lossy().to_string()
    })
}

#[cfg(feature = "cli")]
#[test]
fn test_cli_import_sbvr_minimal() {
    // Create a minimal graph and export to SBVR via API, then import with CLI
    let mut graph = sea_core::Graph::new();
    let e1 = sea_core::primitives::Entity::new_with_namespace("Warehouse", "default");
    let e2 = sea_core::primitives::Entity::new_with_namespace("Factory", "default");
    let r = sea_core::primitives::Resource::new_with_namespace(
        "Cameras",
        sea_core::units::unit_from_string("units"),
        "default",
    );
    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();
    let flow =
        sea_core::primitives::Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let sbvr = graph.export_sbvr().unwrap();
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.sbvr");
    write(&file, sbvr).unwrap();

    let bin = get_sea_binary();
    let mut cmd = Command::new(&bin);
    cmd.arg("import")
        .arg("--format")
        .arg("sbvr")
        .arg(file.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Imported SBVR to Graph"));
}

#[cfg(feature = "cli")]
#[test]
fn test_cli_import_kg_turtle_minimal() {
    let mut graph = sea_core::Graph::new();
    let e1 = sea_core::primitives::Entity::new_with_namespace("Warehouse", "default");
    let e2 = sea_core::primitives::Entity::new_with_namespace("Factory", "default");
    let r = sea_core::primitives::Resource::new_with_namespace(
        "Cameras",
        sea_core::units::unit_from_string("units"),
        "default",
    );
    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();
    let flow =
        sea_core::primitives::Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let kg = sea_core::kg::KnowledgeGraph::from_graph(&graph).unwrap();
    let turtle = kg.to_turtle();
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ttl");
    write(&file, turtle).unwrap();

    let bin = get_sea_binary();
    let mut cmd = Command::new(&bin);
    cmd.arg("import")
        .arg("--format")
        .arg("kg")
        .arg(file.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Imported KG (Turtle) to Graph"));
}

#[cfg(not(feature = "cli"))]
#[test]
fn cli_import_tests_skipped_without_feature() {
    assert!(true);
}

#[cfg(feature = "shacl")]
#[test]
fn test_cli_import_kg_rdfxml_minimal() {
    let mut graph = sea_core::Graph::new();
    let e1 = sea_core::primitives::Entity::new_with_namespace("Warehouse", "default");
    let e2 = sea_core::primitives::Entity::new_with_namespace("Factory", "default");
    let r = sea_core::primitives::Resource::new_with_namespace(
        "Cameras",
        sea_core::units::unit_from_string("units"),
        "default",
    );
    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();
    let flow =
        sea_core::primitives::Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let kg = sea_core::kg::KnowledgeGraph::from_graph(&graph).unwrap();
    let rdf_xml = kg.to_rdf_xml();
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.rdf");
    write(&file, rdf_xml).unwrap();

    let bin = get_sea_binary();
    let mut cmd = Command::new(&bin);
    cmd.arg("import")
        .arg("--format")
        .arg("kg")
        .arg(file.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Imported KG (RDF/XML) to Graph"));
}

#[cfg(feature = "shacl")]
#[test]
fn test_cli_import_kg_rdfxml_validation_fails() {
    // Create graph with a flow that violates minExclusive (quantity == 0)
    let mut graph = sea_core::Graph::new();
    let e1 = sea_core::primitives::Entity::new_with_namespace("Warehouse", "default");
    let e2 = sea_core::primitives::Entity::new_with_namespace("Factory", "default");
    let r = sea_core::primitives::Resource::new_with_namespace(
        "Cameras",
        sea_core::units::unit_from_string("units"),
        "default",
    );
    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();
    let flow =
        sea_core::primitives::Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::new(0, 0));
    graph.add_flow(flow).unwrap();

    let kg = sea_core::kg::KnowledgeGraph::from_graph(&graph).unwrap();
    let rdf_xml = kg.to_rdf_xml();
    let dir = tempdir().unwrap();
    let file = dir.path().join("test_bad.rdf");
    write(&file, rdf_xml).unwrap();

    let bin = get_sea_binary();
    let mut cmd = Command::new(&bin);
    cmd.arg("import")
        .arg("--format")
        .arg("kg")
        .arg(file.to_str().unwrap());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("SHACL validation failed"));
}

#[cfg(feature = "shacl")]
#[test]
fn test_import_kg_rdfxml_direct() {
    let mut graph = sea_core::Graph::new();
    let e1 = sea_core::primitives::Entity::new_with_namespace("Warehouse", "default");
    let e2 = sea_core::primitives::Entity::new_with_namespace("Factory", "default");
    let r = sea_core::primitives::Resource::new_with_namespace(
        "Cameras",
        sea_core::units::unit_from_string("units"),
        "default",
    );
    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();
    let flow =
        sea_core::primitives::Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let kg = sea_core::kg::KnowledgeGraph::from_graph(&graph).unwrap();
    let rdf_xml = kg.to_rdf_xml();

    // Import using the direct function
    let imported = sea_core::import_kg_rdfxml(&rdf_xml);
    assert!(imported.is_ok(), "RDF/XML import failed: {:?}", imported);
    let imported_graph = imported.unwrap();
    assert_eq!(imported_graph.entity_count(), 2);
    assert_eq!(imported_graph.resource_count(), 1);
    assert_eq!(imported_graph.flow_count(), 1);
}

#[cfg(feature = "shacl")]
#[test]
fn test_import_kg_rdfxml_direct_validation_fails() {
    // Create graph with a flow that violates minExclusive (quantity == 0)
    let mut graph = sea_core::Graph::new();
    let e1 = sea_core::primitives::Entity::new_with_namespace("Warehouse", "default");
    let e2 = sea_core::primitives::Entity::new_with_namespace("Factory", "default");
    let r = sea_core::primitives::Resource::new_with_namespace(
        "Cameras",
        sea_core::units::unit_from_string("units"),
        "default",
    );
    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();
    let flow =
        sea_core::primitives::Flow::new(r_id, e1_id, e2_id, rust_decimal::Decimal::new(0, 0));
    graph.add_flow(flow).unwrap();

    let kg = sea_core::kg::KnowledgeGraph::from_graph(&graph).unwrap();
    let rdf_xml = kg.to_rdf_xml();

    let imported = sea_core::import_kg_rdfxml(&rdf_xml);
    assert!(
        imported.is_err(),
        "Expected SHACL validation error, got {:?}",
        imported
    );
    match imported.err().unwrap() {
        ImportError::ShaclValidation(msg) => {
            assert!(
                msg.contains("SHACL validation failed"),
                "Expected SHACL validation failure, got: {}",
                msg
            );
        }
        other => panic!("Expected SHACL validation error, got {:?}", other),
    }
}
