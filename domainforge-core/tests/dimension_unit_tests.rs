//! Tests for parsing dimension and unit declarations.
use domainforge_core::parser::ast::AstNode;
use domainforge_core::parser::parse_source;
use domainforge_core::units::{Dimension, UnitRegistry};
use rust_decimal_macros::dec;
#[test]
fn test_parse_dimension() {
    let source = r#"
        Dimension "Currency"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    match &ast.declarations[0].node {
        AstNode::Dimension { name } => {
            assert_eq!(name, "Currency");
        }
        _ => panic!("Expected Dimension declaration"),
    }
}
#[test]
fn test_parse_unit() {
    let source = r#"
        Unit "USD" of "Currency" factor 1 base "USD"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    match &ast.declarations[0].node {
        AstNode::UnitDeclaration {
            symbol,
            dimension,
            ref factor,
            base_unit,
        } => {
            assert_eq!(symbol, "USD");
            assert_eq!(dimension, "Currency");
            assert_eq!(*factor, dec!(1));
            assert_eq!(base_unit, "USD");
        }
        _ => panic!("Expected UnitDeclaration"),
    }
}
#[test]
fn test_parse_unit_with_decimal_factor() {
    let source = r#"
        Unit "EUR" of "Currency" factor 1.07 base "USD"
    "#;
    let ast = parse_source(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    match &ast.declarations[0].node {
        AstNode::UnitDeclaration { ref factor, .. } => {
            assert_eq!(*factor, dec!(1.07));
        }
        _ => panic!("Expected UnitDeclaration"),
    }
}

#[test]
fn test_register_from_json_case_insensitive_dimension() {
    let mut registry = UnitRegistry::new();
    let json = r#"[{"symbol":"NTEST","name":"Test","dimension":"currency","base_factor":1.0,"base_unit":"USD"}]"#;
    registry
        .register_from_json(json)
        .expect("Failed to register from json");
    let unit = registry.get_unit("NTEST").expect("Unit not registered");
    assert_eq!(unit.dimension(), &Dimension::Currency);
}

#[test]
fn test_dimension_from_str_is_case_insensitive() {
    use std::str::FromStr;
    let d1 = Dimension::from_str("currency").unwrap();
    let d2 = Dimension::from_str("Currency").unwrap();
    let d3 = Dimension::from_str("CURRENCY").unwrap();
    assert_eq!(d1, Dimension::Currency);
    assert_eq!(d2, Dimension::Currency);
    assert_eq!(d3, Dimension::Currency);
    // Custom dims parse to lowercased custom name
    let c = Dimension::from_str("MyDim").unwrap();
    assert_eq!(c, Dimension::Custom("mydim".to_string()));
}

#[test]
fn test_resolve_path_keeps_declared_units_and_dimensions_in_source_order() {
    // The CLI resolve path (parse/validate/project) merges one converted graph
    // per namespace via Graph::absorb. Declared units/dimensions must survive
    // that merge exactly like every other collection.
    use domainforge_core::application::resolve_application_graph;
    use serde_json::json;
    let sources = json!({
        "main.sea": "@namespace \"accessors\"\nDimension \"Length\"\nUnit \"m\" of \"Length\" factor 1 base \"m\"\nUnit \"km\" of \"Length\" factor 1000 base \"m\"\nEntity \"Warehouse\"\n",
    })
    .to_string();
    let graph = resolve_application_graph("main.sea", &sources).expect("resolves");
    let dims: Vec<String> = graph
        .all_declared_dimensions()
        .iter()
        .map(|d| d.name.clone())
        .collect();
    assert_eq!(dims, vec!["Length".to_string()]);
    let units: Vec<(String, String)> = graph
        .all_declared_units()
        .iter()
        .map(|u| (u.name.clone(), u.base_factor.clone()))
        .collect();
    assert_eq!(
        units,
        vec![
            ("m".to_string(), "1".to_string()),
            ("km".to_string(), "1000".to_string())
        ]
    );
}
