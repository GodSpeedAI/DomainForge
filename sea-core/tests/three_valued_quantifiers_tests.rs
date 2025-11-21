use rust_decimal::Decimal;
use sea_core::{
    policy::{BinaryOp, Expression, Policy, Quantifier},
    primitives::{Entity, Flow, Resource},
    units::unit_from_string,
    Graph,
};
use std::str::FromStr;

fn build_graph_with_optional_flow_attribute() -> Graph {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default");
    let factory = Entity::new_with_namespace("Factory", "default");
    let product = Resource::new_with_namespace("Widget", unit_from_string("units"), "default");

    let warehouse_id = warehouse.id().clone();
    let factory_id = factory.id().clone();
    let product_id = product.id().clone();

    graph.add_entity(warehouse).unwrap();
    graph.add_entity(factory).unwrap();
    graph.add_resource(product).unwrap();

    // Flow with missing optional attribute "tag"
    let mut flow1 = Flow::new(
        product_id.clone(),
        warehouse_id.clone(),
        factory_id.clone(),
        Decimal::from_str("100").unwrap(),
    );
    // Explicitly set tag to Null so substitution finds the field and the evaluator treats it as NULL
    flow1.set_attribute("tag", serde_json::Value::Null);
    graph.add_flow(flow1).unwrap();

    // Flow with tag present
    let mut flow2 = Flow::new(
        product_id.clone(),
        warehouse_id.clone(),
        factory_id.clone(),
        Decimal::from_str("200").unwrap(),
    );
    flow2.set_attribute("tag", serde_json::json!("X"));
    graph.add_flow(flow2).unwrap();

    graph
}

#[test]
fn test_forall_with_nulls_results_in_unknown() {
    let graph = build_graph_with_optional_flow_attribute();

    // ForAll f in flows: f.tag == "X" -> one flow has tag X, one has null -> overall Unknown (Null)
    let expr = Expression::quantifier(
        Quantifier::ForAll,
        "flow",
        Expression::variable("flows"),
        Expression::comparison("flow.tag", "==", "X").unwrap(),
    );

    let policy = Policy::new("ForAll TagX", expr);
    let result = policy.evaluate(&graph).unwrap();

    // Backward-compatible boolean => false when Unknown
    assert!(!result.is_satisfied);
    // New tri-state shows None for Unknown
    assert_eq!(result.is_satisfied_tristate, None);
    // Violations should indicate Unknown with severity derived from modality
    assert_eq!(result.violations.len(), 1);
    assert_eq!(
        result.violations[0].severity,
        sea_core::policy::Severity::Error
    );
}

#[test]
fn test_exists_with_true_and_null_returns_true() {
    let graph = build_graph_with_optional_flow_attribute();

    // Exists f in flows: f.tag == "X" -> one flow True, one Null -> Should be True
    let expr = Expression::quantifier(
        Quantifier::Exists,
        "flow",
        Expression::variable("flows"),
        Expression::comparison("flow.tag", "==", "X").unwrap(),
    );

    let policy = Policy::new("Exists TagX", expr);
    let result = policy.evaluate(&graph).unwrap();
    assert!(result.is_satisfied);
    assert_eq!(result.is_satisfied_tristate, Some(true));
    assert_eq!(result.violations.len(), 0);
}

#[test]
fn test_exists_unique_with_duplicate_true_returns_false() {
    let graph = build_graph_with_optional_flow_attribute();

    // Add a third flow which has tag "X" to create duplicate true
    let mut graph2 = graph.clone();
    let mut flow3 = Flow::new(
        graph2.all_resources()[0].id().clone(),
        graph2.all_entities()[0].id().clone(),
        graph2.all_entities()[1].id().clone(),
        Decimal::from_str("300").unwrap(),
    );
    flow3.set_attribute("tag", serde_json::json!("X"));
    graph2.add_flow(flow3).unwrap();

    // ExistsUnique should now be false since more than 1 true
    let expr = Expression::quantifier(
        Quantifier::ExistsUnique,
        "flow",
        Expression::variable("flows"),
        Expression::comparison("flow.tag", "==", "X").unwrap(),
    );

    let policy = Policy::new("ExistsUnique TagX", expr);
    let result = policy.evaluate(&graph2).unwrap();
    assert!(!result.is_satisfied);
    assert_eq!(result.is_satisfied_tristate, Some(false));
}

#[test]
fn test_nested_null_propagation_and_forall_unknown() {
    let mut graph = build_graph_with_optional_flow_attribute();

    // Update flows: set tag Null and quantities positive so AND of (tag == X) and (quantity > 0)
    // yields Null for tag==X and True for quantity -> overall Null per AND when no False.
    // The graph contains one flow with tag Null and one with tag "X" and quantity >0 so ForAll yields False.
    // For this test, we modify both flows to have tag Null to force Unknown.
    let flow_id2 = graph.all_flows()[1].id().clone();
    // Update attributes by finding flows via graph.get_flow
    if let Some(mut f2) = graph.get_flow(&flow_id2).cloned() {
        f2.set_attribute("tag", serde_json::Value::Null);
        // replace flow in graph
        let _ = graph.remove_flow(&flow_id2);
        graph.add_flow(f2).unwrap();
    }

    let expr = Expression::quantifier(
        Quantifier::ForAll,
        "flow",
        Expression::variable("flows"),
        Expression::binary(
            BinaryOp::And,
            Expression::comparison("flow.tag", "==", "X").unwrap(),
            Expression::comparison("flow.quantity", ">", 0).unwrap(),
        ),
    );

    let policy = Policy::new("Nested Null ForAll", expr);
    let result = policy.evaluate(&graph).unwrap();
    // All flows have Null tag; equality is Null -> AND is Null -> ForAll over Nulls yields Null
    assert_eq!(result.is_satisfied_tristate, None);
    assert!(
        !result.violations.is_empty(),
        "Expected at least one violation when evaluation is NULL"
    );
    assert_eq!(
        result.violations[0].severity,
        sea_core::policy::Severity::Error
    );
}
