//! Canonical evaluation-mode tests.
//!
//! The legacy runtime logic toggle (boolean vs three-valued) was removed per the
//! semantic-infrastructure audit (G1): three-valued (Kleene) logic is now the
//! single, authoritative semantics. These tests pin that there is exactly one
//! evaluation mode and that NULL attributes produce an indeterminate result
//! rather than silently coercing to `false`.

use sea_core::graph::Graph;
use sea_core::policy::{BinaryOp, EvaluationMode, Expression, Policy, Severity};
use sea_core::primitives::Entity;

#[test]
fn null_attribute_evaluates_to_unknown_under_canonical_logic() {
    let mut graph = Graph::new();

    let mut entity = Entity::new_with_namespace("TestEntity".to_string(), "default".to_string());
    entity.set_attribute("status", serde_json::Value::Null);
    graph.add_entity(entity).unwrap();

    let policy = Policy::new(
        "TestPolicy",
        Expression::binary(
            BinaryOp::Equal,
            Expression::member_access("TestEntity", "status"),
            Expression::literal(true),
        ),
    );

    let result = policy.evaluate(&graph).unwrap();

    // NULL attribute bubbles up to an indeterminate (UNKNOWN) result.
    assert_eq!(result.is_satisfied_tristate, None);
    assert!(!result.is_satisfied, "fail-closed boolean must be false");
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].severity, Severity::Error);
}

#[test]
fn evaluation_mode_is_always_three_valued() {
    let graph = Graph::new();
    let policy = Policy::new("Trivial", Expression::literal(true));

    let result = policy.evaluate(&graph).unwrap();

    // There is only one canonical mode; every result self-describes as three-valued.
    assert_eq!(result.evaluation_mode, EvaluationMode::ThreeValued);
    assert_eq!(result.evaluation_mode.as_str(), "three_valued");
}
