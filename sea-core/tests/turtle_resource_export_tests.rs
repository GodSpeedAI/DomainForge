#[cfg(test)]
mod turtle_resource_export_tests {
    use sea_core::primitives::Resource;
    use sea_core::{kg::KnowledgeGraph, unit_from_string, Graph};

    #[test]
    fn test_resource_name_with_special_chars_exports_correctly() {
        let mut graph = Graph::new();
        let resource = Resource::new_with_namespace(
            "Camera\n(High\tQuality)",
            unit_from_string("units"),
            "test",
        );
        graph.add_resource(resource).unwrap();

        let kg = KnowledgeGraph::from_graph(&graph).unwrap();
        let turtle = kg.to_turtle();

        assert!(turtle.contains(r#"rdfs:label "Camera\n(High\tQuality)""#));
    }

    #[test]
    fn test_resource_unit_display_format() {
        let mut graph = Graph::new();
        let resource =
            Resource::new_with_namespace("Steel", unit_from_string("kg"), "default".to_string());
        graph.add_resource(resource).unwrap();

        let kg = KnowledgeGraph::from_graph(&graph).unwrap();
        let turtle = kg.to_turtle();

        assert!(turtle.contains(r#"sea:unit "kg""#));
    }
}
