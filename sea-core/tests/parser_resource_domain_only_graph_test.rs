use sea_core::parser::parse_to_graph;
use sea_core::units::Dimension;

#[test]
fn test_resource_domain_only_defaults_to_units_dimension() {
    let source = r#"
        Resource "Counter" in ops
    "#;

    let graph = parse_to_graph(source).expect("Failed to parse to graph");
    // find resource "Counter" in graph (by name key)
    let res = graph
        .all_resources()
        .into_iter()
        .find(|r| r.name() == "Counter")
        .expect("Counter resource not found");

    // Default unit should be "units"
    assert_eq!(res.unit_symbol(), "units");
    assert_eq!(res.unit().dimension(), &Dimension::Count);
}
