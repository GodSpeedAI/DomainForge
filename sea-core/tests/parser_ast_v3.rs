use sea_core::parser::{ast_schema, parse};
use serde_json::Value;

#[test]
fn test_ast_v3_flow_annotations() {
    let source = r#"
        Entity "Sender"
        Entity "Receiver"
        Resource "Data"

        Flow "Data" 
            @cqrs "command"
            @idempotency true
        from "Sender" to "Receiver"
    "#;

    let internal_ast = parse(source).expect("Failed to parse source");
    let schema_ast: ast_schema::Ast = (&internal_ast).into();

    let json_output = serde_json::to_string_pretty(&schema_ast).expect("Failed to serialize");
    println!("AST v3 Output:\nd{}", json_output);

    // Verify annotations are present in the JSON AST
    let output: Value = serde_json::from_str(&json_output).expect("Failed to deserialize JSON");

    let declarations = output
        .get("declarations")
        .expect("Missing declarations")
        .as_array()
        .expect("Declarations not an array");

    let flow_decl = declarations
        .iter()
        .find(|d| {
            d.get("node")
                .and_then(|n| n.get("type"))
                .and_then(|t| t.as_str())
                == Some("Flow")
        })
        .expect("Flow declaration not found");

    let node = flow_decl.get("node").expect("Missing node object");
    let annotations = node.get("annotations").expect("Missing annotations object");

    assert_eq!(
        annotations.get("cqrs").and_then(|v| v.as_str()),
        Some("command")
    );
    assert_eq!(
        annotations.get("idempotency").and_then(|v| v.as_bool()),
        Some(true)
    );
}
