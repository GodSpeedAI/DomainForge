use sea_core::parser::ast::{Ast, AstNode, FileMetadata};
use sea_core::parser::printer::PrettyPrinter;
use std::collections::HashMap;
use serde_json::json;
use sea_core::parser::ast::{PolicyMetadata, PolicyKind, PolicyModality, MappingRule};
use sea_core::policy::Expression;

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
                version: None,
                annotations: HashMap::new(),
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
            },
        ],
    };

    let printer = PrettyPrinter::new();
    let output = printer.print(&ast);

    let expected = r#"@namespace "test_ns"

Entity "Factory"

Resource "Widget" units

Flow "Widget" from "Warehouse" to "Factory" quantity 100
"#;

    assert_eq!(output, expected);
}

#[test]
fn test_pretty_print_policy_header() {
    let metadata = PolicyMetadata {
        kind: Some(PolicyKind::Constraint),
        modality: Some(PolicyModality::Obligation),
        priority: Some(3),
        rationale: Some("Test rationale".to_string()),
        tags: vec![],
    };

    let ast = Ast {
        metadata: FileMetadata::default(),
        declarations: vec![AstNode::Policy {
            name: "Limit".to_string(),
            version: None,
            metadata,
            expression: Expression::literal(true),
        }],
    };

    let printer = PrettyPrinter::new();
    let output = printer.print(&ast);
    assert!(output.contains("per Constraint Obligation"));
    assert!(output.contains("priority 3"));
}

#[test]
fn test_pretty_print_mapping_nested_and_trailing_commas() {
    let mut fields = HashMap::new();
    fields.insert(
        "nested".to_string(),
        json!({ "bar": 1, "baz": [1, 2] }),
    );
    fields.insert("simple".to_string(), json!("ok"));

    let rule = MappingRule {
        primitive_type: "Resource".to_string(),
        primitive_name: "Camera".to_string(),
        target_type: "calm::Resource".to_string(),
        fields,
    };

    let mapping_ast = Ast {
        metadata: FileMetadata::default(),
        declarations: vec![AstNode::MappingDecl {
            name: "m1".to_string(),
            target: sea_core::parser::ast::TargetFormat::Calm,
            rules: vec![rule],
        }],
    };

    // Default: no trailing commas
    let printer = PrettyPrinter::new();
    let output_default = printer.print(&mapping_ast);
    assert!(output_default.contains("simple: \"ok\""));
    assert!(output_default.contains("\"bar\" : 1"));
    assert!(output_default.contains("\"baz\" : [1, 2]"));

    // With trailing commas
    let printer_comma = PrettyPrinter::new().with_trailing_commas(true);
    let output_comma = printer_comma.print(&mapping_ast);
    assert!(output_comma.contains("simple: \"ok\","));
}
