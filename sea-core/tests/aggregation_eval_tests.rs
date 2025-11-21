use rust_decimal::Decimal;
use sea_core::policy::{AggregateFunction, BinaryOp, Expression, Policy};
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::{Dimension, Unit};
use sea_core::Graph;

fn build_test_graph() -> Graph {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    let gold = Resource::new_with_namespace("Gold", kg, "default".to_string());

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    for i in 1..=3 {
        let flow = Flow::new(
            gold.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(i * 100),
        );
        graph.add_flow(flow).unwrap();
    }

    graph
}

#[test]
fn test_count_flows() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "Count Flows",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Count,
                Expression::variable("flows"),
                None::<&str>,
                None,
            ),
            Expression::literal(3),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn aggregation_in_boolean_context_errors() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "Bare Aggregation",
        Expression::aggregation(
            AggregateFunction::Count,
            Expression::variable("flows"),
            None::<&str>,
            None,
        ),
    );

    let err = policy.evaluate(&graph).unwrap_err();
    assert!(
        err.contains("Aggregation in boolean context requires explicit comparison"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_sum_quantities() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "Sum Quantities",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Sum,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(600.0),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_avg_quantity() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "Average Quantity",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Avg,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(200.0),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_min_quantity() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "Min Quantity",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Min,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(100.0),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_max_quantity() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "Max Quantity",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Max,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(300.0),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}

#[test]
fn test_count_mixed_resources() {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());

    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    let units = Unit::new(
        "units",
        "units",
        Dimension::Count,
        Decimal::from(1),
        "units",
    );
    let camera = Resource::new_with_namespace("Camera", units, "default".to_string());
    let gold = Resource::new_with_namespace("Gold", kg, "default".to_string());

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(camera.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    // Create 2 camera flows
    for _ in 0..2 {
        graph
            .add_flow(Flow::new(
                camera.id().clone(),
                warehouse.id().clone(),
                factory.id().clone(),
                Decimal::from(100),
            ))
            .unwrap();
    }

    // Create 1 gold flow
    graph
        .add_flow(Flow::new(
            gold.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(50),
        ))
        .unwrap();

    // Total should be 3 flows
    let policy_all = Policy::new(
        "All Flow Count",
        Expression::binary(
            BinaryOp::Equal,
            Expression::aggregation(
                AggregateFunction::Count,
                Expression::variable("flows"),
                None::<&str>,
                None,
            ),
            Expression::literal(3),
        ),
    );

    let result = policy_all.evaluate(&graph).unwrap();
    assert!(result.is_satisfied, "Should count all 3 flows");
}

#[test]
fn test_count_greater_than() {
    let graph = build_test_graph();

    let policy = Policy::new(
        "At Least 2 Flows",
        Expression::binary(
            BinaryOp::GreaterThan,
            Expression::aggregation(
                AggregateFunction::Count,
                Expression::variable("flows"),
                None::<&str>,
                None,
            ),
            Expression::literal(2),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
}
