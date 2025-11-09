use rust_decimal::Decimal;
use sea_core::parser::parse;
use sea_core::policy::{AggregateFunction, BinaryOp, Expression, Policy};
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::{unit_from_string, Dimension, Unit};
use sea_core::Graph;

#[test]
fn test_end_to_end_count_aggregation() {
    let source = r#"
        Policy flow_count as: count(flows) > 2
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);

    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)).unwrap();
    let gold = Resource::new("Gold", kg);

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

    let policy = Policy::new(
        "flow_count",
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
    assert!(result.is_satisfied, "Should have more than 2 flows");
}

#[test]
fn test_end_to_end_sum_aggregation() {
    let source = r#"
        Policy total_quantity as: sum(flows.quantity) > 500
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);

    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)).unwrap();
    let gold = Resource::new("Gold", kg);

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

    let policy = Policy::new(
        "total_quantity",
        Expression::binary(
            BinaryOp::GreaterThan,
            Expression::aggregation(
                AggregateFunction::Sum,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(500.0),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied, "Sum should be 600 which is > 500");
}

#[test]
fn test_end_to_end_avg_aggregation() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let units = unit_from_string("units");
    let camera = Resource::new("Camera", units);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(camera.clone()).unwrap();

    // Create flows with quantities 100, 200, 300 (average = 200)
    for qty in [100, 200, 300] {
        let flow = Flow::new(
            camera.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(qty),
        );
        graph.add_flow(flow).unwrap();
    }

    let policy = Policy::new(
        "avg_quantity",
        Expression::binary(
            BinaryOp::GreaterThanOrEqual,
            Expression::aggregation(
                AggregateFunction::Avg,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(200),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied, "Average should be 200 which is >= 200");
}

#[test]
fn test_end_to_end_min_aggregation() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let units = unit_from_string("units");
    let camera = Resource::new("Camera", units);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(camera.clone()).unwrap();

    // Create flows with quantities 10, 50, 100 (min = 10)
    for qty in [10, 50, 100] {
        let flow = Flow::new(
            camera.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(qty),
        );
        graph.add_flow(flow).unwrap();
    }

    let policy = Policy::new(
        "min_check",
        Expression::binary(
            BinaryOp::GreaterThan,
            Expression::aggregation(
                AggregateFunction::Min,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(0),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied, "Min should be 10 which is > 0");
}

#[test]
fn test_end_to_end_max_aggregation() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let units = unit_from_string("units");
    let camera = Resource::new("Camera", units);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(camera.clone()).unwrap();

    // Create flows with quantities 100, 500, 900 (max = 900)
    for qty in [100, 500, 900] {
        let flow = Flow::new(
            camera.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(qty),
        );
        graph.add_flow(flow).unwrap();
    }

    let policy = Policy::new(
        "max_check",
        Expression::binary(
            BinaryOp::LessThan,
            Expression::aggregation(
                AggregateFunction::Max,
                Expression::variable("flows"),
                Some("quantity"),
                None,
            ),
            Expression::literal(1000),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied, "Max should be 900 which is < 1000");
}

#[test]
fn test_complex_aggregation_policy() {
    let mut graph = Graph::new();
    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)).unwrap();
    let gold = Resource::new("Gold", kg);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    for i in 1..=5 {
        let flow = Flow::new(
            gold.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(i * 50),
        );
        graph.add_flow(flow).unwrap();
    }

    // Complex policy: (count > 3) AND (sum < 1000)
    let policy = Policy::new(
        "complex_check",
        Expression::binary(
            BinaryOp::And,
            Expression::binary(
                BinaryOp::GreaterThan,
                Expression::aggregation(
                    AggregateFunction::Count,
                    Expression::variable("flows"),
                    None::<&str>,
                    None,
                ),
                Expression::literal(3),
            ),
            Expression::binary(
                BinaryOp::LessThan,
                Expression::aggregation(
                    AggregateFunction::Sum,
                    Expression::variable("flows"),
                    Some("quantity"),
                    None,
                ),
                Expression::literal(1000.0),
            ),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();
    assert!(
        result.is_satisfied,
        "Should satisfy both count > 3 and sum < 1000"
    );
}

#[test]
fn test_all_aggregate_functions_syntax() {
    let test_cases = vec![
        r#"Policy p1 as: count(flows) > 0"#,
        r#"Policy p2 as: sum(flows.quantity) > 0"#,
        r#"Policy p3 as: avg(flows.quantity) > 0"#,
        r#"Policy p4 as: min(flows.quantity) > 0"#,
        r#"Policy p5 as: max(flows.quantity) > 0"#,
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);
    }
}
