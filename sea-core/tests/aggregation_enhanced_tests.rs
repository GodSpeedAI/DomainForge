use rust_decimal::Decimal;
use sea_core::parser::parse_source;
use sea_core::policy::Policy;
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::{Dimension, Unit};
use sea_core::Graph;

fn build_test_graph() -> Graph {
    let mut graph = Graph::new();

    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let shop = Entity::new_with_namespace("Shop", "default".to_string());

    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    let gold = Resource::new_with_namespace("Gold", kg, "default".to_string());

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_entity(shop.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    // 2 flows to Factory
    for i in 1..=2 {
        let flow = Flow::new(
            gold.id().clone(),
            warehouse.id().clone(),
            factory.id().clone(),
            Decimal::from(i * 100),
        );
        graph.add_flow(flow).unwrap();
    }

    // 1 flow to Shop
    let flow = Flow::new(
        gold.id().clone(),
        warehouse.id().clone(),
        shop.id().clone(),
        Decimal::from(500),
    );
    graph.add_flow(flow).unwrap();

    graph
}

#[test]
fn test_group_by_entity() {
    let graph = build_test_graph();

    // group_by(f in flows: f.to_entity) { sum(f.quantity) >= 200 }
    // Factory: 100 + 200 = 300 >= 200 (True)
    // Shop: 500 >= 200 (True)
    // Result: True

    let source = r#"
    Policy group_by_test as:
        group_by(f in flows: f.to_entity) {
            sum(f.quantity) >= 200
        }
    "#;

    let ast = parse_source(source).unwrap();
    let policy_decl = &ast.declarations[0];

    if let sea_core::parser::ast::AstNode::Policy { expression, .. } = &policy_decl.node {
        let policy = Policy::new("group_by_test", expression.clone());
        let result = policy.evaluate(&graph).unwrap();
        assert!(result.is_satisfied);
    } else {
        panic!("Expected policy");
    }
}

#[test]
fn test_group_by_entity_fail() {
    let graph = build_test_graph();

    // group_by(f in flows: f.to_entity) { sum(f.quantity) > 400 }
    // Factory: 300 > 400 (False)
    // Shop: 500 > 400 (True)
    // Result: False

    let source = r#"
    Policy group_by_test_fail as:
        group_by(f in flows: f.to_entity) {
            sum(f.quantity) > 400
        }
    "#;

    let ast = parse_source(source).unwrap();
    let policy_decl = &ast.declarations[0];

    if let sea_core::parser::ast::AstNode::Policy { expression, .. } = &policy_decl.node {
        let policy = Policy::new("group_by_test_fail", expression.clone());
        let result = policy.evaluate(&graph).unwrap();
        assert!(!result.is_satisfied);
    } else {
        panic!("Expected policy");
    }
}

#[test]
fn test_group_by_count() {
    let graph = build_test_graph();

    // group_by(f in flows: f.to_entity) { count(f) >= 1 }
    // Factory: 2 flows
    // Shop: 1 flow
    // Result: True (all groups have at least 1 flow)

    let source = r#"
    Policy group_by_count as:
        group_by(f in flows: f.to_entity) {
            count(f) >= 1
        }
    "#;

    let ast = parse_source(source).unwrap();
    let policy_decl = &ast.declarations[0];

    if let sea_core::parser::ast::AstNode::Policy { expression, .. } = &policy_decl.node {
        let policy = Policy::new("group_by_count", expression.clone());
        let result = policy.evaluate(&graph).unwrap();
        assert!(result.is_satisfied);
    } else {
        panic!("Expected policy");
    }
}

#[test]
fn test_group_by_with_filter() {
    let graph = build_test_graph();

    // group_by(f in flows where f.quantity > 150: f.to_entity) { count(f) = 1 }
    // Filter:
    // - Flow 1 (100) -> Excluded
    // - Flow 2 (200) -> Included (Factory)
    // - Flow 3 (500) -> Included (Shop)
    // Groups:
    // - Factory: 1 flow
    // - Shop: 1 flow
    // Condition: count = 1
    // Result: True

    let source = r#"
    Policy group_by_filter as:
        group_by(f in flows where f.quantity > 150: f.to_entity) {
            count(f) = 1
        }
    "#;

    let ast = parse_source(source).unwrap();
    let policy_decl = &ast.declarations[0];

    if let sea_core::parser::ast::AstNode::Policy { expression, .. } = &policy_decl.node {
        let policy = Policy::new("group_by_filter", expression.clone());
        let result = policy.evaluate(&graph).unwrap();
        assert!(result.is_satisfied);
    } else {
        panic!("Expected policy");
    }
}
