use rust_decimal::Decimal;
use sea_core::{
    primitives::{Entity, Flow, Resource},
    units::unit_from_string,
    Graph,
};

#[test]
fn test_flow_creation_with_resource_unit() {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let kg = unit_from_string("kg");
    let gold = Resource::new_with_namespace("Gold", kg, "default".to_string());

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    // For MVP, Flow quantity is assumed to be in the resource's unit
    let flow = Flow::new(
        gold.id().clone(),
        warehouse.id().clone(),
        factory.id().clone(),
        Decimal::from(100),
    );

    let result = graph.add_flow(flow);
    assert!(result.is_ok());
}
