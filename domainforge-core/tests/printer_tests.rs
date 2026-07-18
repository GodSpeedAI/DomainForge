use domainforge_core::parser::ast::{Ast, AstNode, FileMetadata, Spanned};
use domainforge_core::parser::ast::{MappingRule, PolicyKind, PolicyMetadata, PolicyModality};
use domainforge_core::parser::printer::PrettyPrinter;
use domainforge_core::policy::Expression;
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
                body: None,
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
                quantity: Some(rust_decimal::Decimal::from(100)),
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
            target: domainforge_core::parser::ast::TargetFormat::Calm,
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
    use domainforge_core::parser::ast::ProjectionOverride;

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
            target: domainforge_core::parser::ast::TargetFormat::Calm,
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

#[test]
fn flagship_format_is_canonical_and_idempotent() {
    use domainforge_core::formatter::{format, FormatConfig};
    use domainforge_core::parser::parse;

    let source =
        std::fs::read_to_string("../fixtures/application_generation/flagship/command-write.sea")
            .expect("flagship command fixture must exist");
    let once = format(&source, FormatConfig::default()).expect("first format");
    let twice = format(&once, FormatConfig::default()).expect("second format");
    assert_eq!(once, twice, "format must be idempotent");
    assert!(
        once.contains("    key order_id: uuid\n"),
        "entity body field must be canonically indented: {}",
        once
    );
    assert!(
        once.contains("    placed = \"placed\"\n"),
        "enum member must serialize as `name = \"wire\"`: {}",
        once
    );
    // Round-trip must be node-identical; span line/column legitimately shifts
    // when whitespace is normalized, so we compare node-for-node.
    let source_ast = parse(&source).expect("source parses");
    let once_ast = parse(&once).expect("formatted output parses");
    assert_eq!(
        node_list(&source_ast.declarations),
        node_list(&once_ast.declarations),
        "format must round-trip parse-equal (modulo spans)"
    );
}

fn node_list(decls: &[domainforge_core::parser::ast::Spanned<AstNode>]) -> Vec<AstNode> {
    decls.iter().map(|d| strip_spans(&d.node)).collect()
}

/// Return a clone of `node` with all `Spanned` line/column zeroed so two
/// ASTs that differ only by source positions compare equal.
fn strip_spans(node: &AstNode) -> AstNode {
    match node {
        AstNode::Export(inner) => {
            AstNode::Export(Box::new(domainforge_core::parser::ast::Spanned {
                node: strip_spans(&inner.node),
                line: 0,
                column: 0,
            }))
        }
        other => other.clone(),
    }
}

#[test]
fn query_fixture_round_trips() {
    use domainforge_core::formatter::{format, FormatConfig};
    use domainforge_core::parser::parse;

    let source =
        std::fs::read_to_string("../fixtures/application_generation/flagship/query-read.sea")
            .expect("flagship query fixture must exist");
    let once = format(&source, FormatConfig::default()).expect("format");
    let twice = format(&once, FormatConfig::default()).expect("re-format");
    assert_eq!(once, twice);
    let s = parse(&source).unwrap();
    let o = parse(&once).unwrap();
    assert_eq!(node_list(&s.declarations), node_list(&o.declarations));
}

#[test]
fn partial_operation_preserves_authored_clause_order() {
    use domainforge_core::formatter::{format, FormatConfig};
    use domainforge_core::parser::parse;

    // Authored order: direction, then intent. Valid canonical order would be
    // intent then direction. Partial operation must keep authored order so
    // APP001 evidence (missing/duplicate/out-of-order) survives formatting.
    let src = "@namespace \"app\"\noperation o {\n  direction inbound\n  intent \"late\"\n}";
    let once = format(src, FormatConfig::default()).expect("format");
    let dir_pos = once.find("direction").unwrap();
    let intent_pos = once.find("intent").unwrap();
    assert!(
        dir_pos < intent_pos,
        "partial operation must keep authored order"
    );
    let s = parse(src).unwrap();
    let o = parse(&once).unwrap();
    assert_eq!(node_list(&s.declarations), node_list(&o.declarations));
}
