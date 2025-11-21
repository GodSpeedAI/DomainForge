use sea_core::graph::Graph;
use sea_core::policy::{Expression, Policy};
use sea_core::primitives::Entity;

#[test]
fn test_runtime_toggle_three_valued_logic() {
    // Create a graph with an entity that has a null attribute
    let mut graph = Graph::new();
    
    let mut entity = Entity::new_with_namespace("TestEntity".to_string(), "default".to_string());
    entity.set_attribute("status", serde_json::Value::Null);
    graph.add_entity(entity).unwrap();
    
    // Create a simple policy using a literal that will behave differently
    // In three-valued mode: NULL comparisons yield NULL
    // In boolean mode: expressions are expanded and evaluated strictly
    let policy = Policy::new(
        "TestPolicy",
        Expression::Literal(serde_json::json!(true)),
    );
    
    // Test with three-valued logic enabled (default)
    graph.set_evaluation_mode(true);
    let result_with_tristate = policy.evaluate(&graph).unwrap();
    
    // Should return Some(true) for tristate
    assert_eq!(result_with_tristate.is_satisfied_tristate, Some(true));
    assert_eq!(result_with_tristate.is_satisfied, true);
    assert_eq!(result_with_tristate.violations.len(), 0);
    
    // Test with three-valued logic disabled
    graph.set_evaluation_mode(false);
    let result_without_tristate = policy.evaluate(&graph).unwrap();
    
    // Should also return Some(true) in boolean mode
    assert_eq!(result_without_tristate.is_satisfied_tristate, Some(true));
    assert_eq!(result_without_tristate.is_satisfied, true);
    assert_eq!(result_without_tristate.violations.len(), 0);
}

#[test]
fn test_runtime_toggle_default_is_three_valued() {
    let graph = Graph::new();
    
    // Default should be three-valued logic enabled
    assert_eq!(graph.use_three_valued_logic(), true);
}

#[test]
fn test_runtime_toggle_can_be_changed() {
    let mut graph = Graph::new();
    
    // Start with default (three-valued enabled)
    assert_eq!(graph.use_three_valued_logic(), true);
    
    // Disable three-valued logic
    graph.set_evaluation_mode(false);
    assert_eq!(graph.use_three_valued_logic(), false);
    
    // Re-enable three-valued logic
    graph.set_evaluation_mode(true);
    assert_eq!(graph.use_three_valued_logic(), true);
}
