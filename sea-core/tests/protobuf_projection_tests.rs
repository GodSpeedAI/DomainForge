//! Protobuf projection integration tests
//!
//! Tests the full pipeline: parse SEA -> Graph -> ProtoFile -> .proto text

use sea_core::parser::ast::ast_to_graph;
use sea_core::parser::parse_source;
use sea_core::projection::protobuf::ProtobufEngine;

#[test]
fn test_parse_entity_to_protobuf() {
    let source = r#"
        Entity "Customer"
        Entity "Order"
        Entity "Product"
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    let proto = ProtobufEngine::project(&graph, "", "test.package");

    assert_eq!(proto.package, "test.package");
    assert_eq!(proto.messages.len(), 3);

    // Messages should be sorted alphabetically
    assert_eq!(proto.messages[0].name, "Customer");
    assert_eq!(proto.messages[1].name, "Order");
    assert_eq!(proto.messages[2].name, "Product");

    // Each message should have id and name fields
    for msg in &proto.messages {
        assert!(msg.fields.iter().any(|f| f.name == "id"));
        assert!(msg.fields.iter().any(|f| f.name == "name"));
    }
}

#[test]
fn test_parse_resource_to_protobuf() {
    let source = r#"
        Resource "Money" USD
        Resource "Inventory" units
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    let proto = ProtobufEngine::project(&graph, "", "resources.package");

    assert_eq!(proto.messages.len(), 2);

    // Each resource message should have id, name, quantity, unit fields
    for msg in &proto.messages {
        let field_names: Vec<&str> = msg.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"id"));
        assert!(field_names.contains(&"name"));
        assert!(field_names.contains(&"quantity"));
        assert!(field_names.contains(&"unit"));
    }
}

#[test]
fn test_namespace_filtering() {
    let source = r#"
        @namespace "logistics"
        Entity "Warehouse"
        Entity "Vehicle"
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    // Filter by logistics namespace
    let proto = ProtobufEngine::project(&graph, "logistics", "logistics.package");
    assert_eq!(proto.messages.len(), 2);

    // Filter by non-existent namespace
    let proto_empty = ProtobufEngine::project(&graph, "finance", "finance.package");
    assert_eq!(proto_empty.messages.len(), 0);
}

#[test]
fn test_protobuf_text_output() {
    let source = r#"
        Entity "PaymentProcessor"
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    let proto = ProtobufEngine::project(&graph, "", "payments");
    let proto_text = proto.to_proto_string();

    // Check essential proto3 elements
    assert!(proto_text.contains("syntax = \"proto3\";"));
    assert!(proto_text.contains("package payments;"));
    assert!(proto_text.contains("message Paymentprocessor {"));
    assert!(proto_text.contains("string id = 1;"));
    assert!(proto_text.contains("string name = 2;"));
}

#[test]
fn test_governance_messages() {
    let source = r#"
        Entity "TestEntity"
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    let proto = ProtobufEngine::project_with_options(
        &graph,
        "",
        "governance.test",
        "GovernanceProjection",
        true,
    );

    // Should have entity + 2 governance messages
    assert_eq!(proto.messages.len(), 3);

    let message_names: Vec<&str> = proto.messages.iter().map(|m| m.name.as_str()).collect();
    assert!(message_names.contains(&"PolicyViolation"));
    assert!(message_names.contains(&"MetricEvent"));
    assert!(message_names.contains(&"Testentity"));
}

#[test]
fn test_protobuf_projection_declaration() {
    let source = r#"
        Entity "Vendor"

        Projection "vendor_proto" for protobuf {
            Entity "Vendor" {
                package: "vendor.api"
            }
        }
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    // Verify the projection was parsed
    assert_eq!(graph.projection_count(), 1);

    // Project to protobuf
    let proto = ProtobufEngine::project(&graph, "", "vendor.api");
    assert_eq!(proto.messages.len(), 1);
    assert_eq!(proto.messages[0].name, "Vendor");
}

#[test]
fn test_deterministic_field_ordering() {
    let source = r#"
        Entity "Customer"
        Entity "Order"
    "#;

    let ast1 = parse_source(source).expect("Failed to parse");
    let graph1 = ast_to_graph(ast1).expect("Failed to build graph");
    let proto1 = ProtobufEngine::project(&graph1, "", "test");

    let ast2 = parse_source(source).expect("Failed to parse");
    let graph2 = ast_to_graph(ast2).expect("Failed to build graph");
    let proto2 = ProtobufEngine::project(&graph2, "", "test");

    // Messages should be identical (same count, same names, same fields)
    assert_eq!(proto1.messages.len(), proto2.messages.len());

    for (m1, m2) in proto1.messages.iter().zip(proto2.messages.iter()) {
        assert_eq!(m1.name, m2.name);
        assert_eq!(m1.fields.len(), m2.fields.len());

        for (f1, f2) in m1.fields.iter().zip(m2.fields.iter()) {
            assert_eq!(f1.name, f2.name);
            assert_eq!(f1.number, f2.number);
        }
    }
}

#[test]
fn test_proto_options() {
    let source = r#"
        Entity "Example"
    "#;

    let ast = parse_source(source).expect("Failed to parse");
    let graph = ast_to_graph(ast).expect("Failed to build graph");

    let mut proto = ProtobufEngine::project(&graph, "", "com.example");
    proto.options.java_package = Some("com.example.proto".to_string());
    proto.options.java_multiple_files = true;
    proto.options.go_package = Some("github.com/example/proto".to_string());

    let proto_text = proto.to_proto_string();

    assert!(proto_text.contains("option java_package = \"com.example.proto\";"));
    assert!(proto_text.contains("option java_multiple_files = true;"));
    assert!(proto_text.contains("option go_package = \"github.com/example/proto\";"));
}
