use sea_core::parser::{parse, parse_to_graph, parse_to_graph_with_options, AstNode, ParseOptions};

#[test]
fn test_parse_entity_basic() {
    let source = r#"Entity "Warehouse A""#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_entity_with_domain() {
    let source = r#"Entity "Warehouse A" in logistics"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_resource_basic() {
    let source = r#"Resource "Camera Units""#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_resource_with_units() {
    let source = r#"Resource "Camera Units" units"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_resource_with_units_and_domain() {
    let source = r#"Resource "Camera Units" units in inventory"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_resource_with_domain_only() {
    let source = r#"Resource "USD" in finance"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_flow_basic() {
    let source = r#"Flow "Camera Units" from "Warehouse" to "Factory""#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_flow_with_quantity() {
    let source = r#"Flow "Camera Units" from "Warehouse" to "Factory" quantity 100"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_simple() {
    let source = r#"Policy check_qty as: Flow.quantity > 0"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_with_colon() {
    let source = r#"Policy check_qty as: Flow.quantity > 0"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
    match &ast.declarations[0].node {
        AstNode::Policy { name, version, .. } => {
            assert_eq!(name, "check_qty");
            assert!(version.is_none());
        }
        _ => panic!("Expected policy declaration"),
    }

    let missing_colon = r#"Policy check_qty as Flow.quantity > 0"#;
    assert!(parse(missing_colon).is_err());
}

#[test]
fn test_parse_policy_and_expression() {
    let source = r#"Policy check as: (A > 0) and (B < 10)"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_or_expression() {
    let source = r#"Policy check as: (A = "yes") or (B = "no")"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_not_expression() {
    let source = r#"Policy check as: not (A = "bad")"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_comparison_operators() {
    let operators = vec![">", "<", ">=", "<=", "=", "!="];

    for op in operators {
        let source = format!(r#"Policy test as: A {} 10"#, op);
        let ast = parse(&source).unwrap();
        assert_eq!(ast.declarations.len(), 1);
    }
}

#[test]
fn test_parse_policy_string_operators() {
    let operators = vec!["contains", "startswith", "endswith"];

    for op in operators {
        let source = format!(r#"Policy test as: name {} "test""#, op);
        let ast = parse(&source).unwrap();
        assert_eq!(ast.declarations.len(), 1);
    }
}

#[test]
fn test_parse_policy_arithmetic() {
    let source = r#"Policy check as: (A + B) > 10"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_nested_parentheses() {
    let source = r#"Policy check as: ((A and B) or (C and D))"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_quantifier_forall() {
    let source = r#"Policy check as: forall f in flows: (f.quantity > 0)"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_quantifier_exists() {
    let source = r#"Policy check as: exists e in entities: (e.name = "Factory")"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_policy_quantifier_exists_unique() {
    let source = r#"Policy check as: exists_unique r in resources: (r.unit = "kg")"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_multiple_declarations() {
    let source = r#"
        Entity "Warehouse" in logistics
        Entity "Factory" in production
        Resource "Camera" units
        Flow "Camera" from "Warehouse" to "Factory" quantity 50
        Policy check as: Flow.quantity > 0
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 5);
}

#[test]
fn test_parse_with_comments() {
    let source = r#"
        // This is a warehouse
        Entity "Warehouse" in logistics
        // This is a camera resource
        Resource "Camera" units
        // Flow definition
        Flow "Camera" from "Warehouse" to "Factory"
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 3);
}

#[test]
fn test_parse_empty_source() {
    let source = "";
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 0);
}

#[test]
fn test_parse_whitespace_only() {
    let source = "   \n\t  \n  ";
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 0);
}

#[test]
fn test_parse_comments_only() {
    let source = r#"
        // Just comments
        // Nothing else
    "#;

    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 0);
}

#[test]
fn test_parse_member_access() {
    let source = r#"Policy check as: Entity.name = "Warehouse""#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_boolean_literals() {
    let source = r#"Policy check as: active = true"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);

    let source2 = r#"Policy check as: active = false"#;
    let ast2 = parse(source2).unwrap();

    assert_eq!(ast2.declarations.len(), 1);
}

#[test]
fn test_parse_numeric_literals() {
    let source = r#"Policy check as: quantity = 42"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);

    let source2 = r#"Policy check as: price = 19.99"#;
    let ast2 = parse(source2).unwrap();

    assert_eq!(ast2.declarations.len(), 1);
}

#[test]
fn test_parse_negative_numbers() {
    let source = r#"Policy check as: temperature > -10"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_parse_unary_negation() {
    let source = r#"Policy check as: result = -value"#;
    let ast = parse(source).unwrap();

    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_ast_to_graph_entities() {
    let source = r#"
        Entity "Warehouse" in logistics
        Entity "Factory" in production
    "#;

    let graph = parse_to_graph(source).unwrap();
    assert_eq!(graph.all_entities().len(), 2);
}

#[test]
fn test_ast_to_graph_resources() {
    let source = r#"
        Resource "Camera" units
        Resource "USD" currency in finance
    "#;

    let graph = parse_to_graph(source).unwrap();
    assert_eq!(graph.all_resources().len(), 2);
}

#[test]
fn test_ast_to_graph_flows() {
    let source = r#"
        Entity "Warehouse"
        Entity "Factory"
        Resource "Camera" units
        Flow "Camera" from "Warehouse" to "Factory" quantity 100
    "#;

    let graph = parse_to_graph(source).unwrap();
    assert_eq!(graph.all_entities().len(), 2);
    assert_eq!(graph.all_resources().len(), 1);
    assert_eq!(graph.all_flows().len(), 1);
}

#[test]
fn test_ast_to_graph_undefined_entity_error() {
    let source = r#"
        Resource "Camera" units
        Flow "Camera" from "Warehouse" to "Factory"
    "#;

    let result = parse_to_graph(source);
    assert!(result.is_err());
}

#[test]
fn test_ast_to_graph_undefined_resource_error() {
    let source = r#"
        Entity "Warehouse"
        Entity "Factory"
        Flow "Camera" from "Warehouse" to "Factory"
    "#;

    let result = parse_to_graph(source);
    assert!(result.is_err());
}

#[test]
fn test_ast_to_graph_duplicate_entity_error() {
    let source = r#"
        Entity "Warehouse"
        Entity "Warehouse"
    "#;

    let result = parse_to_graph(source);
    assert!(result.is_err());
}

#[test]
fn test_ast_to_graph_duplicate_resource_error() {
    let source = r#"
        Resource "Camera" units
        Resource "Camera" units
    "#;

    let result = parse_to_graph(source);
    assert!(result.is_err());
}

#[test]
fn test_parse_to_graph_with_options_overrides_namespace() {
    let source = r#"
        Entity "Warehouse"
        Resource "Camera" units
    "#;

    let options = ParseOptions {
        default_namespace: Some("logistics".to_string()),
        namespace_registry: None,
        entry_path: None,
        ..Default::default()
    };

    let graph = parse_to_graph_with_options(source, &options).unwrap();
    let entity = graph.all_entities().into_iter().next().expect("entity");

    assert_eq!(entity.namespace(), "logistics");
}

#[test]
fn test_parse_to_graph_with_options_preserves_explicit_namespace() {
    let source = r#"
        Entity "Warehouse" in production
        Resource "Camera" units
    "#;

    let options = ParseOptions {
        default_namespace: Some("logistics".to_string()),
        namespace_registry: None,
        entry_path: None,
        ..Default::default()
    };

    let graph = parse_to_graph_with_options(source, &options).unwrap();
    let entity = graph.all_entities().into_iter().next().expect("entity");

    assert_eq!(entity.namespace(), "production");
}

#[test]
fn test_parse_case_insensitive_keywords() {
    let sources = vec![
        r#"ENTITY "Test""#,
        r#"Entity "Test""#,
        r#"entity "Test""#,
        r#"RESOURCE "Test" units"#,
        r#"Resource "Test" units"#,
        r#"resource "Test" units"#,
    ];

    for source in sources {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);
    }
}

#[test]
fn test_parse_complex_supply_chain() {
    let source = r#"
        // Entities
        Entity "Supplier A" in sourcing
        Entity "Warehouse B" in logistics
        Entity "Factory C" in production
        Entity "Retailer D" in sales

        // Resources
        Resource "Raw Materials" kg in sourcing
        Resource "Components" units in production
        Resource "Finished Product" units in sales

        // Flows
        Flow "Raw Materials" from "Supplier A" to "Warehouse B" quantity 1000
        Flow "Raw Materials" from "Warehouse B" to "Factory C" quantity 800
        Flow "Components" from "Factory C" to "Retailer D" quantity 500

        // Policies
        Policy min_stock as: forall f in flows: (f.quantity > 0)
        Policy max_capacity as: forall e in entities: (e.capacity < 10000)
    "#;

    let graph = parse_to_graph(source).unwrap();
    assert_eq!(graph.all_entities().len(), 4);
    assert_eq!(graph.all_resources().len(), 3);
    assert_eq!(graph.all_flows().len(), 3);
}

#[test]
fn test_parse_error_invalid_syntax() {
    let source = r#"Entity"#; // Missing name
    let result = parse(source);
    assert!(result.is_err());
}

#[test]
fn test_parse_error_unclosed_string() {
    let source = r#"Entity "Warehouse"#; // Unclosed quote
    let result = parse(source);
    assert!(result.is_err());
}

#[test]
fn test_parse_error_missing_from() {
    let source = r#"Flow "Camera" to "Factory""#; // Missing 'from'
    let result = parse(source);
    assert!(result.is_err());
}
