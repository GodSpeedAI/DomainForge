use rust_decimal::Decimal;
use sea_core::graph::Graph;
use sea_core::policy::{Expression, Policy, PolicyKind};
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::unit_from_string;

#[test]
fn test_flow_iteration_order_stable() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("Supplier", "supply_chain");
    let entity2 = Entity::new_with_namespace("Manufacturer", "supply_chain");
    let entity3 = Entity::new_with_namespace("Distributor", "supply_chain");

    let resource = Resource::new_with_namespace("Parts", unit_from_string("kg"), "supply_chain");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let entity3_id = entity3.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity1).unwrap();
    graph.add_entity(entity2).unwrap();
    graph.add_entity(entity3).unwrap();
    graph.add_resource(resource).unwrap();

    let flow1 = Flow::new(
        resource_id.clone(),
        entity1_id.clone(),
        entity2_id.clone(),
        Decimal::new(100, 0),
    );
    let flow2 = Flow::new(
        resource_id.clone(),
        entity2_id.clone(),
        entity3_id.clone(),
        Decimal::new(200, 0),
    );
    let flow3 = Flow::new(resource_id, entity1_id, entity3_id, Decimal::new(150, 0));

    graph.add_flow(flow1.clone()).unwrap();
    graph.add_flow(flow2.clone()).unwrap();
    graph.add_flow(flow3.clone()).unwrap();

    let flows1: Vec<_> = graph.all_flows().iter().map(|f| f.id().clone()).collect();
    let flows2: Vec<_> = graph.all_flows().iter().map(|f| f.id().clone()).collect();

    assert_eq!(flows1, flows2, "Flow iteration order must be stable");
}

#[test]
fn test_entity_iteration_order_stable() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("Entity_A", "test");
    let entity2 = Entity::new_with_namespace("Entity_B", "test");
    let entity3 = Entity::new_with_namespace("Entity_C", "test");

    graph.add_entity(entity1.clone()).unwrap();
    graph.add_entity(entity2.clone()).unwrap();
    graph.add_entity(entity3.clone()).unwrap();

    let entities1: Vec<_> = graph
        .all_entities()
        .iter()
        .map(|e| e.id().clone())
        .collect();
    let entities2: Vec<_> = graph
        .all_entities()
        .iter()
        .map(|e| e.id().clone())
        .collect();

    assert_eq!(
        entities1, entities2,
        "Entity iteration order must be stable"
    );
}

#[test]
fn test_resource_iteration_order_stable() {
    let mut graph = Graph::new();

    let resource1 = Resource::new_with_namespace("Resource_A", unit_from_string("units"), "test");
    let resource2 = Resource::new_with_namespace("Resource_B", unit_from_string("units"), "test");
    let resource3 = Resource::new_with_namespace("Resource_C", unit_from_string("units"), "test");

    graph.add_resource(resource1.clone()).unwrap();
    graph.add_resource(resource2.clone()).unwrap();
    graph.add_resource(resource3.clone()).unwrap();

    let resources1: Vec<_> = graph
        .all_resources()
        .iter()
        .map(|r| r.id().clone())
        .collect();
    let resources2: Vec<_> = graph
        .all_resources()
        .iter()
        .map(|r| r.id().clone())
        .collect();

    assert_eq!(
        resources1, resources2,
        "Resource iteration order must be stable"
    );
}

#[test]
fn test_policy_kind_constraint() {
    let expr = Expression::Literal(serde_json::json!(true));
    let policy = Policy::new("test_policy", expr).with_kind(PolicyKind::Constraint);

    assert_eq!(policy.kind(), &PolicyKind::Constraint);
}

#[test]
fn test_policy_kind_derivation() {
    let expr = Expression::Literal(serde_json::json!(true));
    let policy = Policy::new("test_policy", expr).with_kind(PolicyKind::Derivation);

    assert_eq!(policy.kind(), &PolicyKind::Derivation);
}

#[test]
fn test_policy_kind_obligation() {
    let expr = Expression::Literal(serde_json::json!(true));
    let policy = Policy::new("test_policy", expr).with_kind(PolicyKind::Obligation);

    assert_eq!(policy.kind(), &PolicyKind::Obligation);
}

#[test]
fn test_policy_priority_ordering() {
    let expr1 = Expression::Literal(serde_json::json!(true));
    let expr2 = Expression::Literal(serde_json::json!(true));
    let expr3 = Expression::Literal(serde_json::json!(true));

    let policy1 = Policy::new("policy_a", expr1).with_priority(10);
    let policy2 = Policy::new("policy_b", expr2).with_priority(5);
    let policy3 = Policy::new("policy_c", expr3).with_priority(20);

    let mut policies = [policy1, policy2, policy3];
    // sort the array in-place via a mutable slice
    policies[..].sort_by_key(|p| p.priority);

    assert_eq!(policies[0].name, "policy_b");
    assert_eq!(policies[1].name, "policy_a");
    assert_eq!(policies[2].name, "policy_c");
}

#[test]
fn test_policy_evaluation_is_pure() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("TestEntity", "test");
    graph.add_entity(entity).unwrap();

    let expr = Expression::Literal(serde_json::json!(true));
    let policy = Policy::new("pure_policy", expr);

    let result1 = policy.evaluate(&graph).unwrap();
    let result2 = policy.evaluate(&graph).unwrap();

    assert_eq!(result1.is_satisfied, result2.is_satisfied);
    assert_eq!(result1.violations.len(), result2.violations.len());
}
