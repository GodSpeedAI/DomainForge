use sea_core::{Graph, primitives::{Entity, Resource, Flow}, units::unit_from_string};
use rust_decimal::Decimal;

#[test]
fn test_flow_creation_with_resource_unit() {
    let mut graph = Graph::new();

    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let kg = unit_from_string("kg");
    let gold = Resource::new("Gold", kg);

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
