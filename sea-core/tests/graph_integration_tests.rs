use rust_decimal::Decimal;
use sea_core::{
    primitives::{Entity, Flow, Resource, ResourceInstance},
    units::unit_from_string,
    Graph,
};
use std::str::FromStr;

#[test]
fn test_complete_supply_chain_graph() {
    let mut graph = Graph::new();

    let supplier = Entity::new_with_namespace("Steel Supplier", "default".to_string());
    let warehouse = Entity::new_with_namespace("Central Warehouse", "default".to_string());
    let manufacturer = Entity::new_with_namespace("Camera Manufacturer", "default".to_string());
    let retailer = Entity::new_with_namespace("Electronics Retailer", "default".to_string());

    let steel =
        Resource::new_with_namespace("Steel", unit_from_string("kg"), "default".to_string());
    let camera_parts = Resource::new_with_namespace(
        "Camera Parts",
        unit_from_string("units"),
        "default".to_string(),
    );
    let cameras = Resource::new_with_namespace(
        "Finished Cameras",
        unit_from_string("units"),
        "default".to_string(),
    );

    let supplier_id = supplier.id().clone();
    let warehouse_id = warehouse.id().clone();
    let manufacturer_id = manufacturer.id().clone();
    let retailer_id = retailer.id().clone();

    let steel_id = steel.id().clone();
    let camera_parts_id = camera_parts.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(supplier).unwrap();
    graph.add_entity(warehouse).unwrap();
    graph.add_entity(manufacturer).unwrap();
    graph.add_entity(retailer).unwrap();

    graph.add_resource(steel).unwrap();
    graph.add_resource(camera_parts).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow1 = Flow::new(
        steel_id.clone(),
        supplier_id.clone(),
        warehouse_id.clone(),
        Decimal::from_str("500").unwrap(),
    );
    let flow2 = Flow::new(
        camera_parts_id.clone(),
        warehouse_id.clone(),
        manufacturer_id.clone(),
        Decimal::from_str("200").unwrap(),
    );
    let flow3 = Flow::new(
        cameras_id.clone(),
        manufacturer_id.clone(),
        retailer_id.clone(),
        Decimal::from_str("150").unwrap(),
    );

    graph.add_flow(flow1).unwrap();
    graph.add_flow(flow2).unwrap();
    graph.add_flow(flow3).unwrap();

    assert_eq!(graph.entity_count(), 4);
    assert_eq!(graph.resource_count(), 3);
    assert_eq!(graph.flow_count(), 3);

    let warehouse_upstream = graph.upstream_entities(&warehouse_id);
    assert_eq!(warehouse_upstream.len(), 1);
    assert_eq!(warehouse_upstream[0].name(), "Steel Supplier");

    let warehouse_downstream = graph.downstream_entities(&warehouse_id);
    assert_eq!(warehouse_downstream.len(), 1);
    assert_eq!(warehouse_downstream[0].name(), "Camera Manufacturer");

    let manufacturer_upstream = graph.upstream_entities(&manufacturer_id);
    assert_eq!(manufacturer_upstream.len(), 1);

    let manufacturer_downstream = graph.downstream_entities(&manufacturer_id);
    assert_eq!(manufacturer_downstream.len(), 1);
    assert_eq!(manufacturer_downstream[0].name(), "Electronics Retailer");

    let supplier_downstream = graph.downstream_entities(&supplier_id);
    assert_eq!(supplier_downstream.len(), 1);

    let retailer_upstream = graph.upstream_entities(&retailer_id);
    assert_eq!(retailer_upstream.len(), 1);
}

#[test]
fn test_graph_with_instances() {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_resource(cameras).unwrap();

    let instance1 = ResourceInstance::new(cameras_id.clone(), warehouse_id.clone());
    let instance2 = ResourceInstance::new(cameras_id.clone(), warehouse_id.clone());

    graph.add_instance(instance1).unwrap();
    graph.add_instance(instance2).unwrap();

    assert_eq!(graph.instance_count(), 2);
}

#[test]
fn test_graph_validation_prevents_invalid_flows() {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_resource(cameras).unwrap();

    let fake_entity_id = sea_core::ConceptId::from_uuid(uuid::Uuid::new_v4());

    let invalid_flow = Flow::new(
        cameras_id,
        warehouse_id,
        fake_entity_id,
        Decimal::from_str("100").unwrap(),
    );

    assert!(graph.add_flow(invalid_flow).is_err());
    assert_eq!(graph.flow_count(), 0);
}

#[test]
fn test_graph_multi_resource_flows() {
    let mut graph = Graph::new();

    let supplier = Entity::new_with_namespace("Supplier", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());

    let steel =
        Resource::new_with_namespace("Steel", unit_from_string("kg"), "default".to_string());
    let plastic =
        Resource::new_with_namespace("Plastic", unit_from_string("kg"), "default".to_string());
    let electronics = Resource::new_with_namespace(
        "Electronics",
        unit_from_string("units"),
        "default".to_string(),
    );

    let supplier_id = supplier.id().clone();
    let factory_id = factory.id().clone();

    let steel_id = steel.id().clone();
    let plastic_id = plastic.id().clone();
    let electronics_id = electronics.id().clone();

    graph.add_entity(supplier).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(steel).unwrap();
    graph.add_resource(plastic).unwrap();
    graph.add_resource(electronics).unwrap();

    let flow1 = Flow::new(
        steel_id.clone(),
        supplier_id.clone(),
        factory_id.clone(),
        Decimal::from_str("1000").unwrap(),
    );
    let flow2 = Flow::new(
        plastic_id.clone(),
        supplier_id.clone(),
        factory_id.clone(),
        Decimal::from_str("500").unwrap(),
    );
    let flow3 = Flow::new(
        electronics_id.clone(),
        supplier_id.clone(),
        factory_id.clone(),
        Decimal::from_str("200").unwrap(),
    );

    graph.add_flow(flow1).unwrap();
    graph.add_flow(flow2).unwrap();
    graph.add_flow(flow3).unwrap();

    let flows_to_factory = graph.flows_to(&factory_id);
    assert_eq!(flows_to_factory.len(), 3);

    let flows_from_supplier = graph.flows_from(&supplier_id);
    assert_eq!(flows_from_supplier.len(), 3);
}

#[test]
fn test_empty_graph_queries() {
    let graph = Graph::new();

    let fake_id = sea_core::ConceptId::from_uuid(uuid::Uuid::new_v4());

    assert_eq!(graph.flows_from(&fake_id).len(), 0);
    assert_eq!(graph.flows_to(&fake_id).len(), 0);
    assert_eq!(graph.upstream_entities(&fake_id).len(), 0);
    assert_eq!(graph.downstream_entities(&fake_id).len(), 0);
}
