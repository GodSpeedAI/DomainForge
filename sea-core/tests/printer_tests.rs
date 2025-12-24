use sea_core::parser::ast::{Ast, AstNode, FileMetadata, Spanned};
use sea_core::parser::ast::{MappingRule, PolicyKind, PolicyMetadata, PolicyModality};
use sea_core::parser::printer::PrettyPrinter;
use sea_core::policy::Expression;
use serde_json::json;
use std::collections::HashMap;

fn spanned<T>(node: T) -> Spanned<T> {
    Spanned {
        node,
        line: 0,
        column: 0,
    }
}

#[test]
fn test_pretty_print_ast() {
    let ast = Ast {
        metadata: FileMetadata {
            namespace: Some("test_ns".to_string()),
            ..Default::default()
        },
        declarations: vec![
            spanned(AstNode::Entity {
                name: "Factory".to_string(),
                version: None,
                annotations: HashMap::new(),
                domain: None,
            }),
            spanned(AstNode::Resource {
                name: "Widget".to_string(),
                annotations: HashMap::new(),
                unit_name: Some("units".to_string()),
                domain: None,
            }),
            spanned(AstNode::Flow {
                resource_name: "Widget".to_string(),
                annotations: HashMap::new(),
                from_entity: "Warehouse".to_string(),
                to_entity: "Factory".to_string(),
                quantity: Some(100),
            }),
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
        declarations: vec![spanned(AstNode::Policy {
            name: "Limit".to_string(),
            version: None,
            metadata,
            expression: Expression::literal(true),
        })],
    };

    let printer = PrettyPrinter::new();
    let output = printer.print(&ast);
    assert!(output.contains("per Constraint Obligation"));
    assert!(output.contains("priority 3"));
}

#[test]
fn test_pretty_print_mapping_nested_and_trailing_commas() {
    let mut fields = HashMap::new();
    fields.insert("nested".to_string(), json!({ "bar": 1, "baz": [1, 2] }));
    fields.insert("simple".to_string(), json!("ok"));

    let rule = MappingRule {
        primitive_type: "Resource".to_string(),
        primitive_name: "Camera".to_string(),
        target_type: "calm::Resource".to_string(),
        fields,
    };

    let mapping_ast = Ast {
        metadata: FileMetadata::default(),
        declarations: vec![spanned(AstNode::MappingDecl {
            name: "m1".to_string(),
            target: sea_core::parser::ast::TargetFormat::Calm,
            rules: vec![rule],
        })],
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

#[test]
fn test_pretty_print_projection_arrow_and_trailing_commas() {
    use sea_core::parser::ast::ProjectionOverride;

    let mut fields = HashMap::new();
    fields.insert("nested".to_string(), json!({ "bar": 1, "baz": [1, 2] }));
    fields.insert("simple".to_string(), json!("ok"));

    let override_entry = ProjectionOverride {
        primitive_type: "Resource".to_string(),
        primitive_name: "Camera".to_string(),
        fields,
    };

    let projection_ast = Ast {
        metadata: FileMetadata::default(),
        declarations: vec![spanned(AstNode::ProjectionDecl {
            name: "p1".to_string(),
            target: sea_core::parser::ast::TargetFormat::Calm,
            overrides: vec![override_entry],
        })],
    };

    let printer = PrettyPrinter::new();
    let output_default = printer.print(&projection_ast);
    assert!(output_default.contains("Projection \"p1\" for CALM"));
    assert!(output_default.contains("Resource \"Camera\""));
    // Arrow style separator should be used in nested object entries
    assert!(output_default.contains("\"bar\" -> 1"));

    let printer_comma = PrettyPrinter::new().with_trailing_commas(true);
    let output_comma = printer_comma.print(&projection_ast);
    // Trailing commas should appear on the simple field line
    assert!(output_comma.contains("simple: \"ok\","));
    // Arrow separators still present
    assert!(output_comma.contains("\"baz\" -> [1, 2]"));
}

#[test]
fn test_pretty_print_policy_kind_modality_display() {
    let kinds = [
        PolicyKind::Constraint,
        PolicyKind::Derivation,
        PolicyKind::Obligation,
    ];
    let modalities = [
        PolicyModality::Obligation,
        PolicyModality::Prohibition,
        PolicyModality::Permission,
    ];

    for kind in kinds.iter() {
        for modality in modalities.iter() {
            let metadata = PolicyMetadata {
                kind: Some(kind.clone()),
                modality: Some(modality.clone()),
                priority: Some(7),
                rationale: None,
                tags: vec![],
            };
            let ast = Ast {
                metadata: FileMetadata::default(),
                declarations: vec![spanned(AstNode::Policy {
                    name: format!("{:?}-{:?}", kind, modality),
                    version: None,
                    metadata: metadata.clone(),
                    expression: Expression::literal(true),
                })],
            };
            let output = PrettyPrinter::new().print(&ast);
            // The printed header must contain "per <Kind> <Modality>"
            let expected_header = format!("per {} {}", kind, modality);
            assert!(
                output.contains(&expected_header),
                "Missing header: {}\nGot: {}",
                expected_header,
                output
            );
            // Also contains priority
            assert!(output.contains("priority 7"));
        }
    }
}
