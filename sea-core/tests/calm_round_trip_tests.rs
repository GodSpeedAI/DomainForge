use rust_decimal::Decimal;
use sea_core::calm::{export, import};
use sea_core::primitives::{Entity, Flow, Instance, Resource};
use sea_core::units::unit_from_string;
use sea_core::Graph;

#[test]
fn test_round_trip_simple_graph() {
    let mut original_graph = Graph::new();

    let entity = Entity::new_with_namespace("Warehouse".to_string(), "logistics".to_string());
    original_graph.add_entity(entity).unwrap();

    let calm_json = export(&original_graph).unwrap();
    let imported_graph = import(calm_json).unwrap();

    assert_eq!(original_graph.entity_count(), imported_graph.entity_count());

    let imported_entity = imported_graph.all_entities()[0];
    assert_eq!(imported_entity.name(), "Warehouse");
    assert_eq!(imported_entity.namespace(), "logistics");
}

#[test]
fn test_round_trip_complex_graph() {
    let mut original_graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse A".to_string(), "logistics".to_string());
    let factory = Entity::new_with_namespace("Factory B".to_string(), "manufacturing".to_string());
    let cameras = Resource::new_with_namespace(
        "Cameras".to_string(),
        unit_from_string("units"),
        "products".to_string(),
    );

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let cameras_id = cameras.id().clone();

    original_graph.add_entity(warehouse).unwrap();
    original_graph.add_entity(factory).unwrap();
    original_graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id.clone(),
        warehouse_id.clone(),
        factory_id.clone(),
        Decimal::from(100),
    );
    original_graph.add_flow(flow).unwrap();

    let calm_json = export(&original_graph).unwrap();

    let imported_graph = import(calm_json).unwrap();

    assert_eq!(original_graph.entity_count(), imported_graph.entity_count());
    assert_eq!(
        original_graph.resource_count(),
        imported_graph.resource_count()
    );
    assert_eq!(original_graph.flow_count(), imported_graph.flow_count());

    let entities = imported_graph.all_entities();
    let entity_names: Vec<&str> = entities.iter().map(|e| e.name()).collect();
    assert!(entity_names.contains(&"Warehouse A"));
    assert!(entity_names.contains(&"Factory B"));

    let resources = imported_graph.all_resources();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name(), "Cameras");
    assert_eq!(resources[0].unit().symbol(), "units");

    let flows = imported_graph.all_flows();
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].quantity().to_string(), "100");
}

#[test]
fn test_round_trip_with_instances() {
    let mut original_graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse".to_string(), "default".to_string());
    let cameras = Resource::new_with_namespace("Cameras".to_string(), unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let cameras_id = cameras.id().clone();

    original_graph.add_entity(warehouse).unwrap();
    original_graph.add_resource(cameras).unwrap();

    let instance = Instance::new(cameras_id.clone(), warehouse_id.clone());
    original_graph.add_instance(instance).unwrap();

    let calm_json = export(&original_graph).unwrap();

    let imported_graph = import(calm_json).unwrap();

    assert_eq!(original_graph.entity_count(), imported_graph.entity_count());
    assert_eq!(
        original_graph.resource_count(),
        imported_graph.resource_count()
    );
    assert_eq!(
        original_graph.instance_count(),
        imported_graph.instance_count()
    );
}

#[test]
fn test_round_trip_preserves_metadata() {
    let mut original_graph = Graph::new();

    let entity = Entity::new_with_namespace("TestEntity".to_string(), "default".to_string());
    original_graph.add_entity(entity).unwrap();

    let calm_json = export(&original_graph).unwrap();

    assert_eq!(calm_json["version"], "2.0");
    assert_eq!(calm_json["metadata"]["sea:exported"], true);
    assert_eq!(calm_json["metadata"]["sea:version"], "0.0.1");
    assert!(calm_json["metadata"]["sea:timestamp"].is_string());

    let imported_graph = import(calm_json).unwrap();

    assert_eq!(original_graph.entity_count(), imported_graph.entity_count());
}

#[test]
fn test_export_structure_compliance() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Test".to_string(), "default".to_string());
    graph.add_entity(entity).unwrap();

    let calm_json = export(&graph).unwrap();

    assert!(calm_json["nodes"].is_array());
    assert!(calm_json["relationships"].is_array());
    assert!(calm_json["metadata"].is_object());
    assert!(calm_json["version"].is_string());

    let nodes = calm_json["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 1);

    let node = &nodes[0];
    assert!(node["unique-id"].is_string());
    assert!(node["node-type"].is_string());
    assert!(node["name"].is_string());
    assert!(node["metadata"].is_object());
    assert_eq!(node["metadata"]["sea:primitive"], "Entity");
}

#[test]
fn test_semantic_equivalence_after_round_trip() {
    let mut original_graph = Graph::new();

    let e1 = Entity::new_with_namespace("Entity1".to_string(), "ns1".to_string());
    let e2 = Entity::new_with_namespace("Entity2".to_string(), "ns1".to_string());
    let r1 = Resource::new_with_namespace(
        "Resource1".to_string(),
        unit_from_string("kg"),
        "ns2".to_string(),
    );

    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let r1_id = r1.id().clone();

    original_graph.add_entity(e1).unwrap();
    original_graph.add_entity(e2).unwrap();
    original_graph.add_resource(r1).unwrap();

    let flow = Flow::new(
        r1_id.clone(),
        e1_id.clone(),
        e2_id.clone(),
        Decimal::new(2500, 2),
    );
    original_graph.add_flow(flow).unwrap();

    let calm_json = export(&original_graph).unwrap();
    let imported_graph = import(calm_json).unwrap();

    let original_entities: Vec<(&str, &str)> = original_graph
        .all_entities()
        .iter()
        .map(|e| (e.name(), e.namespace()))
        .collect();

    let imported_entities: Vec<(&str, &str)> = imported_graph
        .all_entities()
        .iter()
        .map(|e| (e.name(), e.namespace()))
        .collect();

    for orig_entity in &original_entities {
        assert!(
            imported_entities.contains(orig_entity),
            "Entity {:?} not found in imported graph",
            orig_entity
        );
    }

    assert_eq!(
        original_graph.entity_count(),
        imported_graph.entity_count(),
        "Entity count mismatch after round trip"
    );
    assert_eq!(
        original_graph.resource_count(),
        imported_graph.resource_count(),
        "Resource count mismatch after round trip"
    );

    assert_eq!(original_graph.flow_count(), imported_graph.flow_count());
}
