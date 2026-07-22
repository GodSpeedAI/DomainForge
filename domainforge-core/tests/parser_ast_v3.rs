use domainforge_core::parser::{ast_schema, parse};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

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

#[test]
fn application_nodes_are_additive_ast_v3_variants() {
    let source =
        std::fs::read_to_string("../fixtures/application_generation/flagship/command-write.sea")
            .expect("flagship command fixture must exist");
    let internal = parse(&source).expect("flagship command must parse");
    let schema: ast_schema::Ast = (&internal).into();
    let value = serde_json::to_value(&schema).expect("schema must serialize");

    let declarations = value["declarations"]
        .as_array()
        .expect("declarations must be an array");

    // Find the typed Entity node (export wrapper has node.type == "Export";
    // unwrap by name to avoid coupling to declaration index).
    let entity_node = declarations
        .iter()
        .map(|d| &d["node"])
        .find(|node| match node["type"].as_str() {
            Some("Entity") => node["name"].as_str() == Some("Order"),
            Some("Export") => {
                node["declaration"]["node"]["type"].as_str() == Some("Entity")
                    && node["declaration"]["node"]["name"].as_str() == Some("Order")
            }
            _ => false,
        })
        .map(|node| match node["type"].as_str() {
            Some("Export") => &node["declaration"]["node"],
            _ => node,
        })
        .expect("Order entity node must be present");

    assert_eq!(entity_node["type"].as_str(), Some("Entity"));
    assert!(
        entity_node["body"].is_object(),
        "typed entity must serialize an additive body object"
    );

    assert!(
        declarations.iter().any(|d| {
            let node = &d["node"];
            node["type"].as_str() == Some("Operation")
                || (node["type"].as_str() == Some("Export")
                    && node["declaration"]["node"]["type"].as_str() == Some("Operation"))
        }),
        "operation declaration must serialize as an AST v3 Operation"
    );

    assert!(value.get("metadata").is_some(), "metadata must be present");
}

#[test]
fn legacy_bodyless_entity_has_no_body_key_and_matches_oracle() {
    let source =
        std::fs::read_to_string("../fixtures/application_generation/compat/entity-no-body.sea")
            .expect("entity-no-body fixture must exist");
    let internal = parse(&source).expect("legacy entity must parse");
    let schema: ast_schema::Ast = (&internal).into();
    let value = serde_json::to_value(&schema).expect("schema must serialize");

    let entity_node = &value["declarations"][0]["node"];
    assert_eq!(entity_node["type"].as_str(), Some("Entity"));
    assert!(
        entity_node.get("body").is_none(),
        "legacy bodyless entity must not emit a body key (additive contract)"
    );

    // Byte-identical to the pre-change oracle hash captured in Task 1.
    let json = serde_json::to_vec(&schema).expect("schema must re-serialize");
    let hash = sha256_hex(&json);
    assert_eq!(
        hash, "ffeba958f8f68f003b7b66ec3457aa2dc4aa01ff58591ce26fd141dd0a518c27",
        "entity-no-body AST v3 bytes must remain byte-stable"
    );
}
