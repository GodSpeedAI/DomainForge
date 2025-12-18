use sea_core::calm::export::export;
use sea_core::kg::KnowledgeGraph;
use sea_core::parser::ast::{AstNode, TargetFormat};
use sea_core::parser::parse_source;

#[test]
fn test_parse_mapping_and_projection() {
    let source = r#"
    Mapping "payment_to_calm" for calm {
        Entity "PaymentProcessor" -> Node {
            node_type: "resource",
            metadata: {
                "team": "payments",
                "tier": "critical"
            }
        }
    }

    Projection "custom_kg" for kg {
        Entity "Vendor" {
            rdf_class: "org:Organization",
            properties: {
                "name" -> "foaf:name"
            }
        }
    }
    "#;

    let ast = parse_source(source).expect("Failed to parse source");
    assert_eq!(ast.declarations.len(), 2);

    match &ast.declarations[0].node {
        AstNode::MappingDecl {
            name,
            target,
            rules,
        } => {
            assert_eq!(name, "payment_to_calm");
            assert_eq!(*target, TargetFormat::Calm);
            assert_eq!(rules.len(), 1);
            let rule = &rules[0];
            assert_eq!(rule.primitive_type, "Entity");
            assert_eq!(rule.primitive_name, "PaymentProcessor");
            assert_eq!(rule.target_type, "Node");
            assert!(rule.fields.contains_key("node_type"));
            assert!(rule.fields.contains_key("metadata"));
        }
        _ => panic!("Expected MappingDecl"),
    }

    match &ast.declarations[1].node {
        AstNode::ProjectionDecl {
            name,
            target,
            overrides,
        } => {
            assert_eq!(name, "custom_kg");
            assert_eq!(*target, TargetFormat::Kg);
            assert_eq!(overrides.len(), 1);
            let ov = &overrides[0];
            assert_eq!(ov.primitive_type, "Entity");
            assert_eq!(ov.primitive_name, "Vendor");
            assert!(ov.fields.contains_key("rdf_class"));
            assert!(ov.fields.contains_key("properties"));
        }
        _ => panic!("Expected ProjectionDecl"),
    }
}

#[test]
fn test_graph_integration() {
    let source = r#"
    Mapping "m1" for calm {
        Entity "E1" -> Node { node_type: "actor" }
    }
    "#;
    let ast = parse_source(source).expect("Failed to parse");
    let graph = sea_core::parser::ast::ast_to_graph(ast).expect("Failed to build graph");

    assert_eq!(graph.mapping_count(), 1);
    let mappings = graph.all_mappings();
    assert_eq!(mappings[0].name(), "m1");
}

#[test]
fn test_calm_export_with_mapping() {
    let source = r#"
    Entity "PaymentProcessor"
    
    Mapping "m1" for calm {
        Entity "PaymentProcessor" -> Node {
            node_type: "resource",
            metadata: { "custom": "value" }
        }
    }
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = sea_core::parser::ast::ast_to_graph(ast).expect("Failed to build graph");

    let calm_json = export(&graph).expect("Failed to export");

    let nodes = calm_json["nodes"].as_array().expect("Expected nodes array");
    let node = nodes
        .iter()
        .find(|n| n["name"] == "PaymentProcessor")
        .expect("Node not found");

    // Check node_type is "resource" (serialized as lowercase usually? NodeType uses lowercase rename_all)
    assert_eq!(node["node-type"], "resource");

    // Check metadata
    let metadata = node["metadata"]
        .as_object()
        .expect("Expected metadata object");
    assert_eq!(metadata["custom"], "value");
}

#[test]
fn test_kg_export_with_projection() {
    let source = r#"
    Entity "Vendor"
    
    Projection "custom_kg" for kg {
        Entity "Vendor" {
            rdf_class: "org:Organization",
            properties: {
                "name" -> "foaf:name"
            }
        }
    }
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = sea_core::parser::ast::ast_to_graph(ast).expect("Failed to build graph");

    let kg = KnowledgeGraph::from_graph(&graph).expect("Failed to create KG");

    // Check for rdf:type org:Organization
    let type_triple = kg.triples.iter().find(|t| {
        t.subject.contains("Vendor") && t.predicate == "rdf:type" && t.object == "org:Organization"
    });
    assert!(type_triple.is_some(), "Did not find overridden rdf:type");

    // Check for foaf:name
    let name_triple = kg
        .triples
        .iter()
        .find(|t| t.subject.contains("Vendor") && t.predicate == "foaf:name");
    assert!(
        name_triple.is_some(),
        "Did not find overridden name property"
    );
}
