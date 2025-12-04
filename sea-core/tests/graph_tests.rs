use rust_decimal::Decimal;
use sea_core::{
    policy::{Expression, Policy},
    primitives::{Entity, Flow, Resource, ResourceInstance},
    units::unit_from_string,
    Graph,
};
use std::str::FromStr;
use uuid::Uuid;

#[test]
fn test_graph_creation() {
    let graph = Graph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.entity_count(), 0);
}

#[test]
fn test_add_entity_to_graph() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("Warehouse A", "default".to_string());
    let entity_id = entity.id().clone();

    graph.add_entity(entity).unwrap();
    assert_eq!(graph.entity_count(), 1);
    assert!(graph.has_entity(&entity_id));
}

#[test]
fn test_add_duplicate_entity() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("Warehouse A", "default".to_string());

    graph.add_entity(entity.clone()).unwrap();
    assert!(graph.add_entity(entity).is_err());
}

#[test]
fn test_add_resource() {
    let mut graph = Graph::new();
    let resource = Resource::new_with_namespace(
        "Camera Units",
        unit_from_string("units"),
        "default".to_string(),
    );
    let resource_id = resource.id().clone();

    graph.add_resource(resource).unwrap();
    assert_eq!(graph.resource_count(), 1);
    assert!(graph.has_resource(&resource_id));
}

#[test]
fn test_get_entity_by_id() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("Warehouse", "default".to_string());
    let entity_id = entity.id().clone();

    graph.add_entity(entity).unwrap();
    let retrieved = graph.get_entity(&entity_id);

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "Warehouse");
}

#[test]
fn test_remove_entity() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("Factory", "default".to_string());
    let entity_id = entity.id().clone();

    graph.add_entity(entity).unwrap();
    assert_eq!(graph.entity_count(), 1);

    graph.remove_entity(&entity_id).unwrap();
    assert_eq!(graph.entity_count(), 0);
}

#[test]
fn test_flows_from_entity() {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id.clone(),
        warehouse_id.clone(),
        factory_id.clone(),
        Decimal::from_str("100").unwrap(),
    );
    graph.add_flow(flow).unwrap();

    let outflows = graph.flows_from(&warehouse_id);
    assert_eq!(outflows.len(), 1);
}

#[test]
fn test_flows_to_entity() {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id.clone(),
        warehouse_id.clone(),
        factory_id.clone(),
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
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());
    let cameras_id = cameras.id().clone();

    graph.add_resource(cameras).unwrap();

    let warehouse_id = sea_core::ConceptId::from_uuid(Uuid::new_v4());
    let factory_id = sea_core::ConceptId::from_uuid(Uuid::new_v4());

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
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();

    let resource_id = sea_core::ConceptId::from_uuid(Uuid::new_v4());

    let flow = Flow::new(
        resource_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );

    assert!(graph.add_flow(flow).is_err());
}

#[test]
fn test_add_policy() {
    let mut graph = Graph::new();
    let policy = Policy::new("Always True", Expression::literal(true));
    let policy_id = policy.id.clone();

    graph.add_policy(policy).unwrap();

    assert_eq!(graph.policy_count(), 1);
    assert!(graph.has_policy(&policy_id));
    assert!(graph.get_policy(&policy_id).is_some());
}

#[test]
fn test_add_instance() {
    let mut graph = Graph::new();
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_resource(cameras).unwrap();

    let instance = ResourceInstance::new(cameras_id.clone(), warehouse_id.clone());
    let instance_id = instance.id().clone();

    graph.add_instance(instance).unwrap();
    assert_eq!(graph.instance_count(), 1);
    assert!(graph.has_instance(&instance_id));
}

#[test]
fn test_add_instance_without_entity() {
    let mut graph = Graph::new();
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());
    let cameras_id = cameras.id().clone();

    graph.add_resource(cameras).unwrap();

    let entity_id = sea_core::ConceptId::from_uuid(Uuid::new_v4());
    let instance = ResourceInstance::new(cameras_id.clone(), entity_id.clone());

    assert!(graph.add_instance(instance).is_err());
}

#[test]
fn test_add_instance_without_resource() {
    let mut graph = Graph::new();
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let warehouse_id = warehouse.id().clone();

    graph.add_entity(warehouse).unwrap();

    let resource_id = sea_core::ConceptId::from_uuid(Uuid::new_v4());
    let instance = ResourceInstance::new(resource_id.clone(), warehouse_id.clone());

    assert!(graph.add_instance(instance).is_err());
}

#[test]
fn test_find_entity_by_name() {
    let mut graph = Graph::new();
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let warehouse_id = warehouse.id().clone();

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
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());
    let cameras_id = cameras.id().clone();

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
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());
    let cameras_id = cameras.id().clone();

    graph.add_resource(cameras).unwrap();
    assert_eq!(graph.resource_count(), 1);

    graph.remove_resource(&cameras_id).unwrap();
    assert_eq!(graph.resource_count(), 0);
}

#[test]
fn test_remove_flow() {
    let mut graph = Graph::new();
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow = Flow::new(
        cameras_id,
        warehouse_id,
        factory_id,
        Decimal::from_str("100").unwrap(),
    );
    let flow_id = flow.id().clone();

    graph.add_flow(flow).unwrap();
    assert_eq!(graph.flow_count(), 1);

    graph.remove_flow(&flow_id).unwrap();
    assert_eq!(graph.flow_count(), 0);
}

#[test]
fn test_remove_instance() {
    let mut graph = Graph::new();
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let warehouse_id = warehouse.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_resource(cameras).unwrap();

    let instance = ResourceInstance::new(cameras_id.clone(), warehouse_id.clone());
    let instance_id = instance.id().clone();

    graph.add_instance(instance).unwrap();
    assert_eq!(graph.instance_count(), 1);

    graph.remove_instance(&instance_id).unwrap();
    assert_eq!(graph.instance_count(), 0);
}

fn build_supply_chain_graph() -> Graph {
    let mut graph = Graph::new();

    let supplier = Entity::new_with_namespace("Supplier", "default".to_string());
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let cameras =
        Resource::new_with_namespace("Cameras", unit_from_string("units"), "default".to_string());

    let supplier_id = supplier.id().clone();
    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let cameras_id = cameras.id().clone();

    graph.add_entity(supplier).unwrap();
    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(cameras).unwrap();

    let flow1 = Flow::new(
        cameras_id.clone(),
        supplier_id.clone(),
        warehouse_id.clone(),
        Decimal::from_str("200").unwrap(),
    );
    let flow2 = Flow::new(
        cameras_id.clone(),
        warehouse_id.clone(),
        factory_id.clone(),
        Decimal::from_str("150").unwrap(),
    );

    graph.add_flow(flow1).unwrap();
    graph.add_flow(flow2).unwrap();

    graph
}

// Phase 14A: Deterministic Iteration Tests
#[test]
fn test_entity_iteration_order_deterministic() {
    let mut graph = Graph::new();

    // Add multiple entities
    let entities = vec![
        Entity::new_with_namespace("Entity A".to_string(), "default".to_string()),
        Entity::new_with_namespace("Entity B".to_string(), "default".to_string()),
        Entity::new_with_namespace("Entity C".to_string(), "default".to_string()),
        Entity::new_with_namespace("Entity D".to_string(), "default".to_string()),
    ];

    for entity in entities {
        graph.add_entity(entity).unwrap();
    }

    // Collect IDs multiple times - IndexMap guarantees same order
    let ids1: Vec<_> = graph
        .all_entities()
        .iter()
        .map(|e| e.id().clone())
        .collect();
    let ids2: Vec<_> = graph
        .all_entities()
        .iter()
        .map(|e| e.id().clone())
        .collect();
    let ids3: Vec<_> = graph
        .all_entities()
        .iter()
        .map(|e| e.id().clone())
        .collect();

    // IndexMap guarantees same order across iterations
    assert_eq!(ids1, ids2);
    assert_eq!(ids2, ids3);
}

#[test]
fn test_flow_iteration_order_deterministic() {
    let mut graph = Graph::new();

    // Setup entities and resource
    let entity_a = Entity::new_with_namespace("Entity A", "default".to_string());
    let entity_b = Entity::new_with_namespace("Entity B", "default".to_string());
    let entity_c = Entity::new_with_namespace("Entity C", "default".to_string());
    let resource =
        Resource::new_with_namespace("Resource", unit_from_string("units"), "default".to_string());

    let entity_a_id = entity_a.id().clone();
    let entity_b_id = entity_b.id().clone();
    let entity_c_id = entity_c.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity_a).unwrap();
    graph.add_entity(entity_b).unwrap();
    graph.add_entity(entity_c).unwrap();
    graph.add_resource(resource).unwrap();

    // Add multiple flows
    let flow1 = Flow::new(
        resource_id.clone(),
        entity_a_id.clone(),
        entity_b_id.clone(),
        Decimal::from_str("100").unwrap(),
    );
    let flow2 = Flow::new(
        resource_id.clone(),
        entity_b_id.clone(),
        entity_c_id.clone(),
        Decimal::from_str("50").unwrap(),
    );
    let flow3 = Flow::new(
        resource_id.clone(),
        entity_a_id.clone(),
        entity_c_id.clone(),
        Decimal::from_str("25").unwrap(),
    );

    graph.add_flow(flow1).unwrap();
    graph.add_flow(flow2).unwrap();
    graph.add_flow(flow3).unwrap();

    // Collect IDs multiple times
    let ids1: Vec<_> = graph.all_flows().iter().map(|f| f.id().clone()).collect();
    let ids2: Vec<_> = graph.all_flows().iter().map(|f| f.id().clone()).collect();
    let ids3: Vec<_> = graph.all_flows().iter().map(|f| f.id().clone()).collect();

    // Order must be consistent
    assert_eq!(ids1, ids2);
    assert_eq!(ids2, ids3);
}

#[test]
fn test_resource_iteration_order_deterministic() {
    let mut graph = Graph::new();

    let resources = vec![
        Resource::new_with_namespace("Resource A", unit_from_string("kg"), "default".to_string()),
        Resource::new_with_namespace(
            "Resource B",
            unit_from_string("units"),
            "default".to_string(),
        ),
        Resource::new_with_namespace(
            "Resource C",
            unit_from_string("liters"),
            "default".to_string(),
        ),
    ];

    for resource in resources {
        graph.add_resource(resource).unwrap();
    }

    let ids1: Vec<_> = graph
        .all_resources()
        .iter()
        .map(|r| r.id().clone())
        .collect();
    let ids2: Vec<_> = graph
        .all_resources()
        .iter()
        .map(|r| r.id().clone())
        .collect();

    assert_eq!(ids1, ids2);
}

#[test]
fn test_instance_iteration_order_deterministic() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Warehouse", "default".to_string());
    let resource =
        Resource::new_with_namespace("Camera", unit_from_string("units"), "default".to_string());

    let entity_id = entity.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity).unwrap();
    graph.add_resource(resource).unwrap();

    // Add multiple instances
    for _ in 1..=5 {
        let instance = ResourceInstance::new(resource_id.clone(), entity_id.clone());
        graph.add_instance(instance).unwrap();
    }

    let ids1: Vec<_> = graph
        .all_instances()
        .iter()
        .map(|i| i.id().clone())
        .collect();
    let ids2: Vec<_> = graph
        .all_instances()
        .iter()
        .map(|i| i.id().clone())
        .collect();

    assert_eq!(ids1, ids2);
}

#[test]
fn test_extend_merges_graphs() {
    let mut left = Graph::new();
    let mut right = Graph::new();

    let left_entity = Entity::new_with_namespace("Warehouse", "logistics");
    let left_resource =
        Resource::new_with_namespace("Camera", unit_from_string("units"), "logistics");
    left.add_entity(left_entity).unwrap();
    left.add_resource(left_resource).unwrap();

    let right_source = Entity::new_with_namespace("Factory", "logistics");
    let right_target = Entity::new_with_namespace("Customer", "logistics");
    let right_resource = Resource::new_with_namespace("Cash", unit_from_string("units"), "finance");
    let right_source_id = right_source.id().clone();
    let right_target_id = right_target.id().clone();
    let right_resource_id = right_resource.id().clone();
    right.add_entity(right_source).unwrap();
    right.add_entity(right_target).unwrap();
    right.add_resource(right_resource).unwrap();

    let flow = Flow::new(
        right_resource_id,
        right_source_id,
        right_target_id,
        Decimal::from(10),
    );
    right.add_flow(flow).unwrap();

    left.extend(right).unwrap();

    assert_eq!(left.entity_count(), 3);
    assert_eq!(left.resource_count(), 2);
    assert_eq!(left.flow_count(), 1);
}
