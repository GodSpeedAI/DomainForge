use sea_core::graph::Graph;
use sea_core::policy::{BinaryOp, Expression, Policy, Severity};
use sea_core::primitives::Entity;

#[test]
fn test_runtime_toggle_three_valued_logic() {
    // Create a graph with an entity that has a null attribute
    let mut graph = Graph::new();

    let mut entity = Entity::new_with_namespace("TestEntity".to_string(), "default".to_string());
    entity.set_attribute("status", serde_json::Value::Null);
    graph.add_entity(entity).unwrap();

    // Create a policy that inspects the null status attribute so modes diverge
    let policy = Policy::new(
        "TestPolicy",
        Expression::binary(
            BinaryOp::Equal,
            Expression::member_access("TestEntity", "status"),
            Expression::literal(true),
        ),
    );

    // Test with three-valued logic enabled (default)
    graph.set_evaluation_mode(true);
    let result_with_tristate = policy.evaluate(&graph).unwrap();

    // NULL attribute bubbles up to an indeterminate result
    assert_eq!(result_with_tristate.is_satisfied_tristate, None);
    assert!(!result_with_tristate.is_satisfied);
    assert_eq!(result_with_tristate.violations.len(), 1);
    assert_eq!(result_with_tristate.violations[0].severity, Severity::Error);

    // Test with three-valued logic disabled
    graph.set_evaluation_mode(false);
    let result_without_tristate = policy.evaluate(&graph).unwrap();

    // Boolean mode treats missing data as false
    assert_eq!(result_without_tristate.is_satisfied_tristate, Some(false));
    assert!(!result_without_tristate.is_satisfied);
    assert_eq!(result_without_tristate.violations.len(), 1);
    assert_eq!(
        result_without_tristate.violations[0].severity,
        Severity::Error
    );
}

#[test]
fn test_runtime_toggle_default_is_three_valued() {
    let graph = Graph::new();

    // Default should be three-valued logic enabled
    assert!(graph.use_three_valued_logic());
}

#[test]
fn test_runtime_toggle_can_be_changed() {
    let mut graph = Graph::new();

    // Start with default (three-valued enabled)
    assert!(graph.use_three_valued_logic());

    // Disable three-valued logic
    graph.set_evaluation_mode(false);
    assert!(!graph.use_three_valued_logic());

    // Re-enable three-valued logic
    graph.set_evaluation_mode(true);
    assert!(graph.use_three_valued_logic());
}
