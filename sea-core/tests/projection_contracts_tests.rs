use rust_decimal::Decimal;
use sea_core::policy::Severity;
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::unit_from_string;
use sea_core::{ConceptId, Graph, KnowledgeGraph, SbvrModel};
use uuid::Uuid;

fn setup_minimal_graph(quantity: i64) -> (Graph, String, String, String) {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("Warehouse".to_string(), "default".to_string());
    let e2 = Entity::new_with_namespace("Factory".to_string(), "default".to_string());
    let r = Resource::new_with_namespace(
        "Cameras".to_string(),
        unit_from_string("units"),
        "default".to_string(),
    );

    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r_id = r.id().clone();
    let e1_id_str = e1_id.to_string();
    let e2_id_str = e2_id.to_string();
    let r_id_str = r_id.to_string();

    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_resource(r).unwrap();

    let flow = Flow::new(
        r_id.clone(),
        e1_id.clone(),
        e2_id.clone(),
        Decimal::new(quantity, 0),
    );
    graph.add_flow(flow).unwrap();

    (graph, e1_id_str, e2_id_str, r_id_str)
}

#[test]
fn test_sbvr_round_trip_minimal() {
    let (graph, _, _, _) = setup_minimal_graph(100);

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
    let (graph, warehouse_id_str, factory_id_str, resource_id_str) = setup_minimal_graph(100);

    // Export to Turtle
    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let turtle = kg.to_turtle();

    // Parse back
    let kg2 = KnowledgeGraph::from_turtle(&turtle).unwrap();
    let graph2 = kg2.to_graph().unwrap();

    assert_eq!(graph2.entity_count(), 2);
    assert_eq!(graph2.resource_count(), 1);
    assert_eq!(graph2.flow_count(), 1);

    let parse_id =
        |s: &str| ConceptId::from_uuid(Uuid::parse_str(s).expect("Invalid UUID from helper"));
    let warehouse_id = parse_id(&warehouse_id_str);
    let factory_id = parse_id(&factory_id_str);
    let resource_id = parse_id(&resource_id_str);

    let warehouse = graph2
        .get_entity(&warehouse_id)
        .expect("Warehouse entity missing after round trip");
    assert_eq!(warehouse.name(), "Warehouse");
    let factory = graph2
        .get_entity(&factory_id)
        .expect("Factory entity missing after round trip");
    assert_eq!(factory.name(), "Factory");

    let resource = graph2
        .get_resource(&resource_id)
        .expect("Cameras resource missing after round trip");
    assert_eq!(resource.name(), "Cameras");
    assert_eq!(resource.unit_symbol(), "units");

    let flows = graph2.all_flows();
    assert_eq!(flows.len(), 1);
    let flow = flows[0];
    assert_eq!(flow.quantity(), Decimal::new(100, 0));
    assert_eq!(flow.from_id(), &warehouse_id);
    assert_eq!(flow.to_id(), &factory_id);
    assert_eq!(flow.resource_id(), &resource_id);
}

#[test]
fn test_kg_shacl_validation_minimal() {
    let (graph, _, _, _) = setup_minimal_graph(0);

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let violations = kg.validate_shacl().expect("Validation returned error");
    assert!(
        !violations.is_empty(),
        "Expected SHACL violations for quantity <= 0"
    );
    let violation = violations.iter().find(|v| {
        v.severity == Severity::Error
            && v.context.get("predicate").and_then(|val| val.as_str()) == Some("sea:quantity")
            && v.context.get("threshold").and_then(|val| val.as_str()) == Some("0")
    });
    assert!(
        violation.is_some(),
        "Expected a minExclusive violation with predicate sea:quantity"
    );
}
