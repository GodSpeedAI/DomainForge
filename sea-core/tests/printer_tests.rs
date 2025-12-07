use sea_core::parser::ast::{Ast, AstNode, FileMetadata};
use sea_core::parser::printer::PrettyPrinter;
use std::collections::HashMap;

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
