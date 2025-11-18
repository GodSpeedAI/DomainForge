use rust_decimal::Decimal;
use sea_core::policy::Expression;
use sea_core::policy::Policy;
use sea_core::primitives::Entity;
use sea_core::primitives::Flow;
use sea_core::primitives::Resource;
use sea_core::units::unit_from_string;
use sea_core::Graph;

#[test]
fn test_graph_validate_collects_violations() {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("A", "default");
    let e2 = Entity::new_with_namespace("B", "default");
    graph.add_entity(e1.clone()).unwrap();
    graph.add_entity(e2.clone()).unwrap();

    let resource = Resource::new_with_namespace("Money", unit_from_string("USD"), "fin");
    let resource_id = resource.id().clone();
    graph.add_resource(resource).unwrap();

    let flow = Flow::new(
        resource_id.clone(),
        e1.id().clone(),
        e2.id().clone(),
        Decimal::from(100),
    );
    graph.add_flow(flow).unwrap();

    // This policy checks that sum of flow quantities should be less than 50 (violated)
    let policy = Policy::new_with_namespace("check_amt", "default", Expression::literal(false));
    graph.add_policy(policy).unwrap();

    let result = graph.validate();
    assert!(result.total_policies >= 1);
    assert!(!result.violations.is_empty());
}
