use sea_core::{Graph, primitives::{Entity, Resource, Flow, Instance}};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_graph_creation() {
    let graph = Graph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.entity_count(), 0);
}

#[test]
fn test_add_entity_to_graph() {
    let mut graph = Graph::new();
    let entity = Entity::new("Warehouse A");
    let entity_id = *entity.id();

    graph.add_entity(entity).unwrap();
    assert_eq!(graph.entity_count(), 1);
    assert!(graph.has_entity(&entity_id));
}

#[test]
fn test_add_duplicate_entity() {
    let mut graph = Graph::new();
    let entity = Entity::new("Warehouse A");

    graph.add_entity(entity.clone()).unwrap();
    assert!(graph.add_entity(entity).is_err());
}

#[test]
fn test_add_resource() {
    let mut graph = Graph::new();
    let resource = Resource::new("Camera Units", "units");
    let resource_id = *resource.id();

    graph.add_resource(resource).unwrap();
    assert_eq!(graph.resource_count(), 1);
    assert!(graph.has_resource(&resource_id));
}

#[test]
fn test_get_entity_by_id() {
    let mut graph = Graph::new();
    let entity = Entity::new("Warehouse");
    let entity_id = *entity.id();

    graph.add_entity(entity).unwrap();
    let retrieved = graph.get_entity(&entity_id);

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "Warehouse");
}

#[test]
fn test_remove_entity() {
    let mut graph = Graph::new();
    let entity = Entity::new("Factory");
    let entity_id = *entity.id();

    graph.add_entity(entity).unwrap();
    assert_eq!(graph.entity_count(), 1);

    graph.remove_entity(&entity_id).unwrap();
    assert_eq!(graph.entity_count(), 0);
}

#[test]
fn test_flows_from_entity() {
    let mut graph = Graph::new();

    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let cameras = Resource::new("Cameras", "units");

    let warehouse_id = *warehouse.id();
    let factory_id = *factory.id();
    let cameras_id = *cameras.id();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );
    graph.add_flow(flow).unwrap();

    let outflows = graph.flows_from(&warehouse_id);
    assert_eq!(outflows.len(), 1);
}

#[test]
fn test_flows_to_entity() {
    let mut graph = Graph::new();

    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let cameras = Resource::new("Cameras", "units");

    let warehouse_id = *warehouse.id();
    let factory_id = *factory.id();
    let cameras_id = *cameras.id();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );
    graph.add_flow(flow).unwrap();

    let inflows = graph.flows_to(&factory_id);
    assert_eq!(inflows.len(), 1);
}

#[test]
fn test_upstream_entities() {
    let graph = build_supply_chain_graph();

    let warehouse_id = graph.find_entity_by_name("Warehouse").unwrap();
    let upstream = graph.upstream_entities(&warehouse_id);

    assert_eq!(upstream.len(), 1);
    assert_eq!(upstream[0].name(), "Supplier");
}

#[test]
fn test_downstream_entities() {
    let graph = build_supply_chain_graph();

    let supplier_id = graph.find_entity_by_name("Supplier").unwrap();
    let downstream = graph.downstream_entities(&supplier_id);

    assert_eq!(downstream.len(), 1);
    assert_eq!(downstream[0].name(), "Warehouse");
}

#[test]
fn test_multi_stage_supply_chain() {
    let graph = build_supply_chain_graph();

    let supplier_id = graph.find_entity_by_name("Supplier").unwrap();
    let warehouse_id = graph.find_entity_by_name("Warehouse").unwrap();
    let factory_id = graph.find_entity_by_name("Factory").unwrap();

    let supplier_downstream = graph.downstream_entities(&supplier_id);
    assert_eq!(supplier_downstream.len(), 1);

    let warehouse_upstream = graph.upstream_entities(&warehouse_id);
    assert_eq!(warehouse_upstream.len(), 1);

    let warehouse_downstream = graph.downstream_entities(&warehouse_id);
    assert_eq!(warehouse_downstream.len(), 1);

    let factory_upstream = graph.upstream_entities(&factory_id);
    assert_eq!(factory_upstream.len(), 1);
}

#[test]
fn test_add_flow_without_entities() {
    let mut graph = Graph::new();
    let cameras = Resource::new("Cameras", "units");
    let cameras_id = *cameras.id();

    graph.add_resource(cameras).unwrap();

    let warehouse_id = uuid::Uuid::new_v4();
    let factory_id = uuid::Uuid::new_v4();

    let flow = Flow::new(
        cameras_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );

    assert!(graph.add_flow(flow).is_err());
}

#[test]
fn test_add_flow_without_resource() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");

    let warehouse_id = *warehouse.id();
    let factory_id = *factory.id();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();

    let resource_id = uuid::Uuid::new_v4();

    let flow = Flow::new(
        resource_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );

    assert!(graph.add_flow(flow).is_err());
}

#[test]
fn test_add_instance() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let cameras = Resource::new("Cameras", "units");

    let warehouse_id = *warehouse.id();
    let cameras_id = *cameras.id();

    graph.add_entity(warehouse).unwrap();
    graph.add_resource(cameras).unwrap();

    let instance = Instance::new(cameras_id, warehouse_id);
    let instance_id = *instance.id();

    graph.add_instance(instance).unwrap();
    assert_eq!(graph.instance_count(), 1);
    assert!(graph.has_instance(&instance_id));
}

#[test]
fn test_add_instance_without_entity() {
    let mut graph = Graph::new();
    let cameras = Resource::new("Cameras", "units");
    let cameras_id = *cameras.id();

    graph.add_resource(cameras).unwrap();

    let entity_id = uuid::Uuid::new_v4();
    let instance = Instance::new(cameras_id, entity_id);

    assert!(graph.add_instance(instance).is_err());
}

#[test]
fn test_add_instance_without_resource() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let warehouse_id = *warehouse.id();

    graph.add_entity(warehouse).unwrap();

    let resource_id = uuid::Uuid::new_v4();
    let instance = Instance::new(resource_id, warehouse_id);

    assert!(graph.add_instance(instance).is_err());
}

#[test]
fn test_find_entity_by_name() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let warehouse_id = *warehouse.id();

    graph.add_entity(warehouse).unwrap();

    let found_id = graph.find_entity_by_name("Warehouse");
    assert!(found_id.is_some());
    assert_eq!(found_id.unwrap(), warehouse_id);

    let not_found = graph.find_entity_by_name("NonExistent");
    assert!(not_found.is_none());
}

#[test]
fn test_find_resource_by_name() {
    let mut graph = Graph::new();
    let cameras = Resource::new("Cameras", "units");
    let cameras_id = *cameras.id();

    graph.add_resource(cameras).unwrap();

    let found_id = graph.find_resource_by_name("Cameras");
    assert!(found_id.is_some());
    assert_eq!(found_id.unwrap(), cameras_id);

    let not_found = graph.find_resource_by_name("NonExistent");
    assert!(not_found.is_none());
}

#[test]
fn test_remove_resource() {
    let mut graph = Graph::new();
    let cameras = Resource::new("Cameras", "units");
    let cameras_id = *cameras.id();

    graph.add_resource(cameras).unwrap();
    assert_eq!(graph.resource_count(), 1);

    graph.remove_resource(&cameras_id).unwrap();
    assert_eq!(graph.resource_count(), 0);
}

#[test]
fn test_remove_flow() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let cameras = Resource::new("Cameras", "units");

    let warehouse_id = *warehouse.id();
    let factory_id = *factory.id();
    let cameras_id = *cameras.id();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );
    let flow_id = *flow.id();

    graph.add_flow(flow).unwrap();
    assert_eq!(graph.flow_count(), 1);

    graph.remove_flow(&flow_id).unwrap();
    assert_eq!(graph.flow_count(), 0);
}

#[test]
fn test_remove_instance() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let cameras = Resource::new("Cameras", "units");

    let warehouse_id = *warehouse.id();
    let cameras_id = *cameras.id();

    graph.add_entity(warehouse).unwrap();
    graph.add_resource(cameras).unwrap();

    let instance = Instance::new(cameras_id, warehouse_id);
    let instance_id = *instance.id();

    graph.add_instance(instance).unwrap();
    assert_eq!(graph.instance_count(), 1);

    graph.remove_instance(&instance_id).unwrap();
    assert_eq!(graph.instance_count(), 0);
}

fn build_supply_chain_graph() -> Graph {
    let mut graph = Graph::new();

    let supplier = Entity::new("Supplier");
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let cameras = Resource::new("Cameras", "units");

    let supplier_id = *supplier.id();
    let warehouse_id = *warehouse.id();
    let factory_id = *factory.id();
    let cameras_id = *cameras.id();

    graph.add_entity(supplier).unwrap();
    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow1 = Flow::new(
        cameras_id,
        supplier_id,
        warehouse_id,
        Decimal::from_str("200").unwrap(),
    );
    let flow2 = Flow::new(
        cameras_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("150").unwrap(),
    );

    graph.add_flow(flow1).unwrap();
    graph.add_flow(flow2).unwrap();

    graph
}
