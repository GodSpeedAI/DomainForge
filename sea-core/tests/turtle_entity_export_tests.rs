use sea_core::primitives::Entity;
use sea_core::{kg::KnowledgeGraph, Graph};

#[test]
fn test_entity_name_with_quotes_exports_correctly() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace(r#"Entity "With Quotes""#, "test");
    graph.add_entity(entity).unwrap();

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let turtle = kg.to_turtle();

    assert!(turtle.contains(r#"rdfs:label "Entity \"With Quotes\""#));
}

#[test]
fn test_entity_namespace_with_newline_exports_correctly() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("TestEntity", "namespace\nwith\nnewlines");
    graph.add_entity(entity).unwrap();

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let turtle = kg.to_turtle();

    assert!(turtle.contains(r#"sea:namespace "namespace\nwith\nnewlines""#));
}

#[test]
fn test_entity_with_backslashes_exports_correctly() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace(r#"Entity\With\Backslashes"#, "default".to_string());
    graph.add_entity(entity).unwrap();

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();
    let turtle = kg.to_turtle();

    assert!(turtle.contains(r#"rdfs:label "Entity\\With\\Backslashes""#));
}
