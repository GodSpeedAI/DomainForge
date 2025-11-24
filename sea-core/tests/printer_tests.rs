use sea_core::parser::ast::{Ast, AstNode, FileMetadata};
use sea_core::parser::printer::PrettyPrinter;

#[test]
fn test_pretty_print_ast() {
    let ast = Ast {
        metadata: FileMetadata {
            namespace: Some("test_ns".to_string()),
            ..Default::default()
        },
        declarations: vec![
            AstNode::Entity {
                name: "Factory".to_string(),
                domain: None,
            },
            AstNode::Resource {
                name: "Widget".to_string(),
                unit_name: Some("units".to_string()),
                domain: None,
            },
            AstNode::Flow {
                resource_name: "Widget".to_string(),
                from_entity: "Warehouse".to_string(),
                to_entity: "Factory".to_string(),
                quantity: Some(100),
                unit: None,
            },
        ],
    };

    let printer = PrettyPrinter::new();
    let output = printer.print(&ast);

    assert!(output.contains("namespace test_ns"));
    assert!(output.contains("Entity \"Factory\""));
    assert!(output.contains("Resource \"Widget\" (units)"));
    assert!(output.contains("Flow \"Widget\" from \"Warehouse\" to \"Factory\" quantity 100"));
}

#[test]
fn test_pretty_print_dsl_v2() {
    let ast = Ast {
        metadata: FileMetadata::default(),
        declarations: vec![
            AstNode::Role {
                name: "Payer".to_string(),
                entity: "Party".to_string(),
            },
            AstNode::Relation {
                name: "Payment".to_string(),
                subject_role: Some("Payer".to_string()),
                object_role: Some("Payee".to_string()),
                via_flow: Some("Money".to_string()),
            },
            AstNode::Instance {
                name: "Batch-1".to_string(),
                resource: "Steel".to_string(),
                entity: "Warehouse".to_string(),
                properties: std::collections::HashMap::new(),
            },
        ],
    };

    let printer = PrettyPrinter::new();
    let output = printer.print(&ast);

    assert!(output.contains("Role \"Payer\" for entity \"Party\""));
    assert!(output.contains("Relation \"Payment\" {"));
    assert!(output.contains("subject role \"Payer\""));
    assert!(output.contains("via flow \"Money\""));
    assert!(output.contains("Instance \"Batch-1\" of resource \"Steel\" at entity \"Warehouse\""));
}
