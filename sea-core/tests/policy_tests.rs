use sea_core::{
    Graph,
    primitives::{Entity, Resource, Flow},
    policy::{Expression, BinaryOp, UnaryOp, Quantifier, Policy, DeonticModality, Severity},
};
use rust_decimal::Decimal;
use std::str::FromStr;

fn build_sample_graph() -> Graph {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "logistics");
    let factory = Entity::new_with_namespace("Factory", "production");
    let cameras = Resource::new_with_namespace("Cameras", "units", "inventory");

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
        Decimal::from_str("150").unwrap(),
    );
    graph.add_flow(flow).unwrap();

    graph
}

#[test]
fn test_simple_comparison() {
    let expr = Expression::binary(
        BinaryOp::GreaterThan,
        Expression::variable("quantity"),
        Expression::literal(100)
    );

    assert_eq!(expr.to_string(), "(quantity > 100)");
}

#[test]
fn test_logical_and() {
    let expr = Expression::binary(
        BinaryOp::And,
        Expression::comparison("x", ">", 0).unwrap(),
        Expression::comparison("y", "<", 100).unwrap()
    );

    assert_eq!(expr.to_string(), "((x > 0) AND (y < 100))");
}

#[test]
fn test_quantifier_expression() {
    let expr = Expression::quantifier(
        Quantifier::ForAll,
        "flow",
        Expression::variable("flows"),
        Expression::comparison("flow.quantity", ">", 0).unwrap()
    );

    assert!(matches!(expr, Expression::Quantifier { .. }));
}

#[test]
fn test_literal_expressions() {
    let bool_expr = Expression::literal(true);
    assert_eq!(bool_expr.to_string(), "true");

    let num_expr = Expression::literal(42);
    assert_eq!(num_expr.to_string(), "42");

    let str_expr = Expression::literal("test");
    assert_eq!(str_expr.to_string(), "\"test\"");
}

#[test]
fn test_unary_not() {
    let expr = Expression::unary(
        UnaryOp::Not,
        Expression::literal(true)
    );

    assert_eq!(expr.to_string(), "NOT true");
}

#[test]
fn test_nested_binary_operations() {
    let inner = Expression::binary(
        BinaryOp::And,
        Expression::comparison("a", ">", 5).unwrap(),
        Expression::comparison("b", "<", 10).unwrap()
    );

    let outer = Expression::binary(
        BinaryOp::Or,
        inner,
        Expression::comparison("c", "==", 0).unwrap()
    );

    assert!(outer.to_string().contains("OR"));
    assert!(outer.to_string().contains("AND"));
}

#[test]
fn test_all_comparison_operators() {
    let ops = vec![
        (">", BinaryOp::GreaterThan),
        ("<", BinaryOp::LessThan),
        (">=", BinaryOp::GreaterThanOrEqual),
        ("<=", BinaryOp::LessThanOrEqual),
        ("==", BinaryOp::Equal),
        ("!=", BinaryOp::NotEqual),
    ];

    for (op_str, op) in ops {
        let expr = Expression::comparison("x", op_str, 5).unwrap();
        assert!(matches!(expr, Expression::Binary { .. }));
        assert!(expr.to_string().contains(&op.to_string()));
    }
}

#[test]
fn test_forall_expansion() {
    let graph = build_sample_graph();

    let expr = Expression::quantifier(
        Quantifier::ForAll,
        "flow",
        Expression::variable("flows"),
        Expression::literal(true)
    );

    let expanded = expr.expand(&graph).unwrap();

    assert!(expanded.to_string().contains("AND"));
}

#[test]
fn test_exists_expansion() {
    let graph = build_sample_graph();

    let expr = Expression::quantifier(
        Quantifier::Exists,
        "entity",
        Expression::variable("entities"),
        Expression::literal(true)
    );

    let expanded = expr.expand(&graph).unwrap();

    assert!(expanded.to_string().contains("OR"));
}

#[test]
fn test_exists_unique_expansion() {
    let graph = build_sample_graph();

    let expr = Expression::quantifier(
        Quantifier::ExistsUnique,
        "entity",
        Expression::variable("entities"),
        Expression::literal(true)
    );

    let expanded = expr.expand(&graph).unwrap();

    assert!(matches!(expanded, Expression::Literal(_)));
}

#[test]
fn test_forall_empty_collection() {
    let graph = Graph::new();

    let expr = Expression::quantifier(
        Quantifier::ForAll,
        "flow",
        Expression::variable("flows"),
        Expression::literal(false)
    );

    let expanded = expr.expand(&graph).unwrap();

    assert_eq!(expanded, Expression::literal(true));
}

#[test]
fn test_exists_empty_collection() {
    let graph = Graph::new();

    let expr = Expression::quantifier(
        Quantifier::Exists,
        "resource",
        Expression::variable("resources"),
        Expression::literal(true)
    );

    let expanded = expr.expand(&graph).unwrap();

    assert_eq!(expanded, Expression::literal(false));
}

#[test]
fn test_evaluate_simple_policy_true() {
    let graph = build_sample_graph();

    let policy = Policy::new(
        "Always True Rule",
        Expression::literal(true)
    );

    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
    assert_eq!(result.violations.len(), 0);
}

#[test]
fn test_evaluate_simple_policy_false() {
    let graph = build_sample_graph();

    let policy = Policy::new(
        "Always False Rule",
        Expression::literal(false)
    );

    let result = policy.evaluate(&graph).unwrap();

    assert!(!result.is_satisfied);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].policy_name, "Always False Rule");
}

#[test]
fn test_evaluate_logical_and_true() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::And,
        Expression::literal(true),
        Expression::literal(true)
    );

    let policy = Policy::new("AND True", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_logical_and_false() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::And,
        Expression::literal(true),
        Expression::literal(false)
    );

    let policy = Policy::new("AND False", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(!result.is_satisfied);
}

#[test]
fn test_evaluate_logical_or_true() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::Or,
        Expression::literal(false),
        Expression::literal(true)
    );

    let policy = Policy::new("OR True", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_logical_or_false() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::Or,
        Expression::literal(false),
        Expression::literal(false)
    );

    let policy = Policy::new("OR False", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(!result.is_satisfied);
}

#[test]
fn test_evaluate_not_true() {
    let graph = build_sample_graph();

    let expr = Expression::unary(
        UnaryOp::Not,
        Expression::literal(true)
    );

    let policy = Policy::new("NOT True", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(!result.is_satisfied);
}

#[test]
fn test_evaluate_not_false() {
    let graph = build_sample_graph();

    let expr = Expression::unary(
        UnaryOp::Not,
        Expression::literal(false)
    );

    let policy = Policy::new("NOT False", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_numeric_comparison_greater_than() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::GreaterThan,
        Expression::literal(10),
        Expression::literal(5)
    );

    let policy = Policy::new("10 > 5", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_numeric_comparison_less_than() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::LessThan,
        Expression::literal(3),
        Expression::literal(7)
    );

    let policy = Policy::new("3 < 7", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_equality_true() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::Equal,
        Expression::literal(42),
        Expression::literal(42)
    );

    let policy = Policy::new("42 == 42", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_equality_false() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::Equal,
        Expression::literal(10),
        Expression::literal(20)
    );

    let policy = Policy::new("10 == 20", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(!result.is_satisfied);
}

#[test]
fn test_evaluate_quantified_forall_policy() {
    let graph = build_sample_graph();

    let expr = Expression::quantifier(
        Quantifier::ForAll,
        "entity",
        Expression::variable("entities"),
        Expression::literal(true)
    );

    let policy = Policy::new("All Entities Pass", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_evaluate_quantified_exists_policy() {
    let graph = build_sample_graph();

    let expr = Expression::quantifier(
        Quantifier::Exists,
        "resource",
        Expression::variable("resources"),
        Expression::literal(true)
    );

    let policy = Policy::new("At Least One Resource Exists", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_violation_severity_obligation() {
    let policy = Policy::new("Must Rule", Expression::literal(false))
        .with_modality(DeonticModality::Obligation);

    let result = policy.evaluate(&Graph::new()).unwrap();
    assert_eq!(result.violations[0].severity, Severity::Error);
}

#[test]
fn test_violation_severity_prohibition() {
    let policy = Policy::new("Must Not Rule", Expression::literal(false))
        .with_modality(DeonticModality::Prohibition);

    let result = policy.evaluate(&Graph::new()).unwrap();
    assert_eq!(result.violations[0].severity, Severity::Error);
}

#[test]
fn test_violation_severity_permission() {
    let policy = Policy::new("May Rule", Expression::literal(false))
        .with_modality(DeonticModality::Permission);

    let result = policy.evaluate(&Graph::new()).unwrap();
    assert_eq!(result.violations[0].severity, Severity::Info);
}

#[test]
fn test_evaluation_result_has_errors() {
    let policy = Policy::new("Error Rule", Expression::literal(false))
        .with_modality(DeonticModality::Obligation);

    let result = policy.evaluate(&Graph::new()).unwrap();

    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);
}

#[test]
fn test_violation_with_context() {
    use sea_core::policy::Violation;

    let violation = Violation::new("Test Policy", "Test message", Severity::Warning)
        .with_context(serde_json::json!({
            "entity_id": "123",
            "field": "quantity"
        }));

    assert_eq!(violation.context["entity_id"], "123");
    assert_eq!(violation.context["field"], "quantity");
}

#[test]
fn test_string_comparison_contains() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::Contains,
        Expression::literal("hello world"),
        Expression::literal("world")
    );

    let policy = Policy::new("Contains", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_string_comparison_starts_with() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::StartsWith,
        Expression::literal("hello world"),
        Expression::literal("hello")
    );

    let policy = Policy::new("StartsWith", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_string_comparison_ends_with() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::EndsWith,
        Expression::literal("hello world"),
        Expression::literal("world")
    );

    let policy = Policy::new("EndsWith", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}

#[test]
fn test_nested_quantifiers() {
    let graph = build_sample_graph();

    let inner = Expression::quantifier(
        Quantifier::Exists,
        "flow",
        Expression::variable("flows"),
        Expression::literal(true)
    );

    let outer = Expression::quantifier(
        Quantifier::ForAll,
        "entity",
        Expression::variable("entities"),
        inner
    );

    let expanded = outer.expand(&graph).unwrap();

    assert!(!matches!(expanded, Expression::Quantifier { .. }));
}

#[test]
fn test_complex_policy_evaluation() {
    let graph = build_sample_graph();

    let expr = Expression::binary(
        BinaryOp::And,
        Expression::binary(
            BinaryOp::Or,
            Expression::literal(true),
            Expression::literal(false)
        ),
        Expression::unary(
            UnaryOp::Not,
            Expression::literal(false)
        )
    );

    let policy = Policy::new("Complex Expression", expr);
    let result = policy.evaluate(&graph).unwrap();

    assert!(result.is_satisfied);
}
