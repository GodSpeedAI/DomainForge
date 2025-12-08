//! Parser -> AST -> Registry mapping tests for dimension normalization
use sea_core::parser::parse_to_graph;
use sea_core::units::{Dimension, UnitRegistry};

#[test]
fn test_ast_to_graph_registers_unit_with_case_insensitive_dimension() {
    // Source with lowercase dimension name in the unit declaration
    let source = r#"
        Dimension "Currency"
        Unit "NCASE" of "currency" factor 1 base "USD"
    "#;

    // Parse to graph; this will register units using UnitRegistry::global()
    let _graph = parse_to_graph(source).expect("Failed to parse to graph");
    // After parsing, the registry should contain the unit
    let registry = UnitRegistry::global();
    let registry = registry.read().expect("Failed to lock registry");
    let unit = registry.get_unit("NCASE").expect("Unit not found");
    assert_eq!(unit.dimension(), &Dimension::Currency);
}
