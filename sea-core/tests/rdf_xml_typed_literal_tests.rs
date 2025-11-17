#[cfg(test)]
mod rdf_xml_typed_literal_tests {
    use roxmltree::Document;
    use rust_decimal::Decimal;
    use sea_core::primitives::{Entity, Flow, Resource};
    use sea_core::{
        kg::{KnowledgeGraph, Triple},
        unit_from_string, Graph,
    };

    #[test]
    fn test_decimal_datatype_preserved() {
        let mut graph = Graph::new();

        let supplier = Entity::new_with_namespace("Supplier", "default".to_string());
        let manufacturer = Entity::new_with_namespace("Manufacturer", "default".to_string());
        let money =
            Resource::new_with_namespace("Money", unit_from_string("USD"), "default".to_string());

        let supplier_id = supplier.id().clone();
        let manufacturer_id = manufacturer.id().clone();
        let money_id = money.id().clone();

        graph.add_entity(supplier).unwrap();
        graph.add_entity(manufacturer).unwrap();
        graph.add_resource(money).unwrap();

        let flow = Flow::new_with_namespace(
            money_id,
            supplier_id,
            manufacturer_id,
            Decimal::new(2500, 2),
            "default",
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

        let doc = Document::parse(&rdf_xml).expect("Failed to parse RDF/XML");
        let label_node = doc
            .descendants()
            .find(|node| {
                node.is_element()
                    && node.tag_name().name() == "label"
                    && node.tag_name().namespace() == Some("http://www.w3.org/2000/01/rdf-schema#")
            })
            .expect("Missing rdfs:label element");

        assert_eq!(label_node.text(), Some("Warehouse"));
        // Use namespace-aware attribute lookup: roxmltree allows checking attributes by
        // expanded name (namespace URI, local name). The `xml:lang` attribute lives in
        // the XML namespace "http://www.w3.org/XML/1998/namespace".
        assert_eq!(
            label_node.attribute(("http://www.w3.org/XML/1998/namespace", "lang")),
            Some("en")
        );
    }

    #[test]
    fn test_escaped_literal_in_xml() {
        let mut graph = Graph::new();
        let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
        let resource = Resource::new_with_namespace(
            "Resource<With>&Special\"'Chars",
            unit_from_string("units"),
            "default".to_string(),
        );

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
        assert_eq!(escaped, "&amp;&lt;&gt;&quot;&apos;");
    }
}
