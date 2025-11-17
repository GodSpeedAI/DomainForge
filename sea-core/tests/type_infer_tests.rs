use sea_core::graph::Graph;
use sea_core::policy::infer_expression_type;
use sea_core::policy::AggregateFunction;
use sea_core::policy::BinaryOp;
use sea_core::policy::Expression;
use sea_core::policy::ExpressionType;

#[test]
fn test_infer_literal_types() {
    let graph = Graph::new();
    assert_eq!(
        infer_expression_type(&Expression::literal(serde_json::json!(true)), &graph),
        ExpressionType::Boolean
    );
    assert_eq!(
        infer_expression_type(&Expression::literal(serde_json::json!(42)), &graph),
        ExpressionType::Numeric
    );
    assert_eq!(
        infer_expression_type(&Expression::literal(serde_json::json!("x")), &graph),
        ExpressionType::String
    );
}

#[test]
fn test_infer_binary_types() {
    let graph = Graph::new();
    let left = Expression::literal(serde_json::json!(1));
    let right = Expression::literal(serde_json::json!(2));
    let expr = Expression::binary(BinaryOp::GreaterThan, left.clone(), right.clone());
    assert_eq!(
        infer_expression_type(&expr, &graph),
        ExpressionType::Boolean
    );

    let add = Expression::binary(BinaryOp::Plus, left.clone(), right.clone());
    assert_eq!(infer_expression_type(&add, &graph), ExpressionType::Numeric);
}

#[test]
fn test_infer_quantifier_and_aggregation() {
    let graph = Graph::new();
    let q = Expression::quantifier(
        sea_core::policy::Quantifier::ForAll,
        "e",
        Expression::variable("entities"),
        Expression::literal(serde_json::json!(true)),
    );
    assert_eq!(infer_expression_type(&q, &graph), ExpressionType::Boolean);

    let agg = Expression::aggregation(
        AggregateFunction::Count,
        Expression::variable("entities"),
        Some("id".to_string()),
        None,
    );
    assert_eq!(infer_expression_type(&agg, &graph), ExpressionType::Numeric);
}
