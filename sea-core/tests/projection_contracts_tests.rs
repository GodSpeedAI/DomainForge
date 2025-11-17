use sea_core::{Graph, SbvrModel, KnowledgeGraph};
use sea_core::primitives::{Entity, Resource, Flow};
use sea_core::units::unit_from_string;
use rust_decimal::Decimal;

#[test]
fn test_sbvr_round_trip_minimal() {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("Warehouse".to_string(), "default".to_string());
    let e2 = Entity::new_with_namespace("Factory".to_string(), "default".to_string());
    let r = Resource::new_with_namespace("Cameras".to_string(), unit_from_string("units"), "default".to_string());

    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();

    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();

    let flow = Flow::new(r_id, e1_id, e2_id, Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    // Export to SBVR XMI
    let xml = graph.export_sbvr().unwrap();

    // Parse back into SbvrModel
    let model = SbvrModel::from_xmi(&xml).expect("Failed to parse SBVR XMI");
    let graph2 = model.to_graph().expect("Failed to convert SBVR to Graph");

    // Validate counts
    assert_eq!(graph2.entity_count(), 2);
    assert_eq!(graph2.resource_count(), 1);
    assert_eq!(graph2.flow_count(), 1);
}

#[test]
fn test_kg_round_trip_minimal() {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("Warehouse".to_string(), "default".to_string());
    let e2 = Entity::new_with_namespace("Factory".to_string(), "default".to_string());
    let r = Resource::new_with_namespace("Cameras".to_string(), unit_from_string("units"), "default".to_string());

    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();

    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();

    let flow = Flow::new(r_id, e1_id, e2_id, Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    // Export to Turtle
    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let turtle = kg.to_turtle();

    // Parse back
    let kg2 = KnowledgeGraph::from_turtle(&turtle).unwrap();
    let graph2 = kg2.to_graph().unwrap();

    assert_eq!(graph2.entity_count(), 2);
    assert_eq!(graph2.resource_count(), 1);
    assert_eq!(graph2.flow_count(), 1);
}

#[test]
fn test_kg_shacl_validation_minimal() {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("Warehouse".to_string(), "default".to_string());
    let e2 = Entity::new_with_namespace("Factory".to_string(), "default".to_string());
    let r = Resource::new_with_namespace("Cameras".to_string(), unit_from_string("units"), "default".to_string());

    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();

    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();

    // Flow with quantity 0 should violate minExclusive > 0 rule
    let flow = Flow::new(r_id, e1_id, e2_id, Decimal::new(0, 0));
    graph.add_flow(flow).unwrap();

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let violations = kg.validate_shacl().expect("Validation returned error");
    assert!(!violations.is_empty(), "Expected SHACL violations for quantity <= 0");
    let found = violations.iter().any(|v| v.message.contains("must be >"));
    assert!(found, "Expected message to include minExclusive violation");
}
