//! Tests for unit mismatch validation during graph validation
use rust_decimal::Decimal;
use sea_core::policy::Policy;
use sea_core::policy::{AggregateFunction, BinaryOp, Expression};
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::unit_from_string;
use sea_core::Graph;

#[test]
fn test_validation_catches_unit_mismatch_in_policy() {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("A", "default");
    let e2 = Entity::new_with_namespace("B", "default");
    graph.add_entity(e1.clone()).unwrap();
    graph.add_entity(e2.clone()).unwrap();

    // Money (Currency) resource
    let money_resource = Resource::new_with_namespace("Money", unit_from_string("USD"), "default");
    let money_id = money_resource.id().clone();
    graph.add_resource(money_resource).unwrap();

    // Weight (Mass) resource
    let weight_resource = Resource::new_with_namespace("Weight", unit_from_string("kg"), "default");
    let weight_id = weight_resource.id().clone();
    graph.add_resource(weight_resource).unwrap();

    // Add flows
    let money_flow = Flow::new(
        money_id.clone(),
        e1.id().clone(),
        e2.id().clone(),
        Decimal::from(100),
    );
    graph.add_flow(money_flow).unwrap();

    let weight_flow = Flow::new(
        weight_id.clone(),
        e1.id().clone(),
        e2.id().clone(),
        Decimal::from(200),
    );
    graph.add_flow(weight_flow).unwrap();

    // Policy that attempts to compare currency sum <= mass literal (mismatched dimensions)
    // Using the simpler aggregation_simple form to avoid comprehension parsing complexities
    // Construct the aggregation comprehension AST directly to avoid parsing issues with raw expressions in test env
    let predicate = Expression::binary(
        BinaryOp::Equal,
        Expression::member_access("f", "resource"),
        Expression::literal("Money"),
    );
    let left_expr = Expression::AggregationComprehension {
        function: AggregateFunction::Sum,
        variable: "f".to_string(),
        collection: Box::new(Expression::Variable("flows".to_string())),
        window: None,
        predicate: Box::new(predicate),
        projection: Box::new(Expression::member_access("f", "quantity")),
        target_unit: Some("USD".to_string()),
    };
    let right_expr = Expression::QuantityLiteral {
        value: Decimal::from(100),
        unit: "kg".to_string(),
    };
    let expr = Expression::binary(BinaryOp::LessThanOrEqual, left_expr, right_expr);
    // Construct expression programmatically; avoid using raw parsing in this test
    let policy = Policy::new("mismatch_test", expr);
    graph.add_policy(policy).unwrap();

    let result = graph.validate();
    // Ensure at least one violation is present and the message refers to unit mismatch
    assert!(
        !result.violations.is_empty(),
        "Expected a violation for unit mismatch"
    );
    let msg = result.violations[0].message.to_lowercase();
    assert!(
        msg.contains("unit") || msg.contains("mismatch"),
        "Expected unit mismatch message, got: {}",
        result.violations[0].message
    );
}
