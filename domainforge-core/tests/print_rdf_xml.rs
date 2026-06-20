use domainforge_core::kg::KnowledgeGraph;
use domainforge_core::kg::Triple;

#[test]
fn print_rdf_xml() {
    let mut kg = KnowledgeGraph::new();
    kg.triples.push(Triple {
        subject: "sea:TestEntity".to_string(),
        predicate: "rdfs:label".to_string(),
        object: "\"Warehouse\"@en".to_string(),
    });
    let rdf_xml = kg.to_rdf_xml();
    println!("XML:\n{}", rdf_xml);
}
