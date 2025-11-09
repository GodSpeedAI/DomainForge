#[cfg(test)]
mod rdf_xml_typed_literal_tests {
    use rust_decimal::Decimal;
    use sea_core::primitives::{Entity, Flow, Resource};
    use sea_core::{
        kg::{KnowledgeGraph, Triple},
        unit_from_string, Graph,
    };

    #[test]
    fn test_decimal_datatype_preserved() {
        let mut graph = Graph::new();

        let supplier = Entity::new("Supplier");
        let manufacturer = Entity::new("Manufacturer");
        let money = Resource::new("Money", unit_from_string("USD"));

        let supplier_id = supplier.id().clone();
        let manufacturer_id = manufacturer.id().clone();
        let money_id = money.id().clone();

        graph.add_entity(supplier).unwrap();
        graph.add_entity(manufacturer).unwrap();
        graph.add_resource(money).unwrap();

        #[allow(deprecated)]
        let flow = Flow::new(
            money_id,
            supplier_id,
            manufacturer_id,
            Decimal::new(2500, 2),
        );
        graph.add_flow(flow).unwrap();

        let kg = KnowledgeGraph::from_graph(&graph).unwrap();
        let rdf_xml = kg.to_rdf_xml();

        assert!(
            rdf_xml.contains("rdf:datatype=\"http://www.w3.org/2001/XMLSchema#decimal\"")
                || rdf_xml.contains("rdf:datatype=\"http://domainforge.ai/xsd#decimal\"")
        );
    }

    #[test]
    fn test_language_tag_preserved() {
        let mut kg = KnowledgeGraph::new();
        kg.triples.push(Triple {
            subject: "sea:TestEntity".to_string(),
            predicate: "rdfs:label".to_string(),
            object: "\"Warehouse\"@en".to_string(),
        });

        let rdf_xml = kg.to_rdf_xml();

        assert!(rdf_xml.contains("xml:lang=\"en\""));
        assert!(rdf_xml.contains(">Warehouse</http://domainforge.ai/rdfs#label>"));
    }

    #[test]
    fn test_escaped_literal_in_xml() {
        let mut graph = Graph::new();
        let warehouse = Entity::new("Warehouse");
        let resource = Resource::new("Resource<With>&Special\"'Chars", unit_from_string("units"));

        graph.add_entity(warehouse).unwrap();
        graph.add_resource(resource).unwrap();

        let kg = KnowledgeGraph::from_graph(&graph).unwrap();
        let rdf_xml = kg.to_rdf_xml();

        assert!(rdf_xml.contains("&lt;"));
        assert!(rdf_xml.contains("&gt;"));
        assert!(rdf_xml.contains("&amp;"));
        assert!(rdf_xml.contains("&quot;"));
        assert!(rdf_xml.contains("&apos;"));
    }

    #[test]
    fn test_escape_xml_helper_covers_all_entities() {
        let escaped = KnowledgeGraph::escape_xml("&<>\"'");
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&apos;"));
    }
}
