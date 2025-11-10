use rust_decimal::Decimal;
use sea_core::graph::Graph;
use sea_core::kg::KnowledgeGraph;
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::sbvr::SbvrModel;
use sea_core::units::unit_from_string;

#[test]
fn test_export_to_sbvr() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("Supplier", "supply_chain");
    let entity2 = Entity::new_with_namespace("Manufacturer", "supply_chain");
    let resource = Resource::new_with_namespace("Parts", unit_from_string("kg"), "supply_chain");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity1).unwrap();
    graph.add_entity(entity2).unwrap();
    graph.add_resource(resource).unwrap();

    let flow = Flow::new(resource_id, entity1_id, entity2_id, Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let sbvr_xml = graph.export_sbvr().unwrap();

    assert!(sbvr_xml.contains("<sbvr:FactType"));
    assert!(sbvr_xml.contains("<sbvr:GeneralConcept"));
    assert!(sbvr_xml.contains("Supplier"));
    assert!(sbvr_xml.contains("Manufacturer"));
}

#[test]
fn test_sbvr_contains_vocabulary() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Company", "business");
    let resource = Resource::new_with_namespace("Money", unit_from_string("USD"), "finance");

    graph.add_entity(entity).unwrap();
    graph.add_resource(resource).unwrap();

    let sbvr_xml = graph.export_sbvr().unwrap();

    assert!(sbvr_xml.contains("<sbvr:Vocabulary"));
    assert!(sbvr_xml.contains("Company"));
    assert!(sbvr_xml.contains("Money"));
}

#[test]
fn test_sbvr_xml_structure() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("Test", "test");
    graph.add_entity(entity).unwrap();

    let sbvr_xml = graph.export_sbvr().unwrap();

    assert!(sbvr_xml.starts_with("<?xml"));
    assert!(sbvr_xml.contains("xmlns:sbvr"));
    assert!(sbvr_xml.ends_with("</xmi:XMI>\n"));
}

#[test]
fn test_export_to_rdf_turtle() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("Supplier", "supply_chain");
    let entity2 = Entity::new_with_namespace("Manufacturer", "supply_chain");
    let resource = Resource::new_with_namespace("Parts", unit_from_string("kg"), "supply_chain");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity1).unwrap();
    graph.add_entity(entity2).unwrap();
    graph.add_resource(resource).unwrap();

    let flow = Flow::new(resource_id, entity1_id, entity2_id, Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let rdf_turtle = graph.export_rdf("turtle").unwrap();

    assert!(rdf_turtle.contains("sea:Entity"));
    assert!(rdf_turtle.contains("sea:hasResource"));
    assert!(rdf_turtle.contains("@prefix sea:"));
}

#[test]
fn test_rdf_turtle_prefixes() {
    let mut graph = Graph::new();
    let entity = Entity::new_with_namespace("Test", "test");
    graph.add_entity(entity).unwrap();

    let rdf = graph.export_rdf("turtle").unwrap();

    assert!(rdf.contains("@prefix sea:"));
    assert!(rdf.contains("@prefix owl:"));
    assert!(rdf.contains("@prefix rdf:"));
    assert!(rdf.contains("@prefix rdfs:"));
    assert!(rdf.contains("@prefix xsd:"));
    assert!(rdf.contains("@prefix sh:"));
}

#[test]
fn test_rdf_xml_format() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Company", "business");
    graph.add_entity(entity).unwrap();

    let rdf_xml = graph.export_rdf("rdf-xml").unwrap();

    assert!(rdf_xml.contains("<?xml"));
    assert!(rdf_xml.contains("<rdf:RDF"));
    assert!(rdf_xml.contains("xmlns:rdf"));
    assert!(rdf_xml.contains("</rdf:RDF>"));
}

#[test]
fn test_rdf_contains_shacl_shapes() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("A", "test");
    let entity2 = Entity::new_with_namespace("B", "test");
    let resource = Resource::new_with_namespace("R", unit_from_string("kg"), "test");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity1).unwrap();
    graph.add_entity(entity2).unwrap();
    graph.add_resource(resource).unwrap();

    let flow = Flow::new(resource_id, entity1_id, entity2_id, Decimal::new(50, 0));
    graph.add_flow(flow).unwrap();

    let rdf = graph.export_rdf("turtle").unwrap();

    assert!(rdf.contains("# SHACL Shapes"));
    assert!(rdf.contains("sh:NodeShape"));
    assert!(rdf.contains("sh:property"));
    assert!(rdf.contains("sh:targetClass"));
}

#[test]
fn test_rdf_flow_properties() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("From", "test");
    let entity2 = Entity::new_with_namespace("To", "test");
    let resource = Resource::new_with_namespace("Material", unit_from_string("kg"), "test");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity1).unwrap();
    graph.add_entity(entity2).unwrap();
    graph.add_resource(resource).unwrap();

    let flow = Flow::new(resource_id, entity1_id, entity2_id, Decimal::new(100, 0));
    graph.add_flow(flow).unwrap();

    let rdf = graph.export_rdf("turtle").unwrap();

    assert!(rdf.contains("sea:from"));
    assert!(rdf.contains("sea:to"));
    assert!(rdf.contains("sea:quantity"));
    assert!(rdf.contains("xsd:decimal"));
}

#[test]
fn test_unsupported_rdf_format() {
    let graph = Graph::new();
    let result = graph.export_rdf("json-ld");

    assert!(result.is_err());
}

#[test]
fn test_sbvr_model_from_graph() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Entity1", "domain");
    let resource = Resource::new_with_namespace("Resource1", unit_from_string("kg"), "domain");

    graph.add_entity(entity).unwrap();
    graph.add_resource(resource).unwrap();

    let model = SbvrModel::from_graph(&graph).unwrap();

    assert!(model.vocabulary.len() >= 2);
}

#[test]
fn test_knowledge_graph_from_graph() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("TestEntity", "test");
    graph.add_entity(entity).unwrap();

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();

    assert!(!kg.triples.is_empty());
}

#[test]
fn test_sbvr_xml_escaping() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("A&B<C>D\"E'F", "test");
    graph.add_entity(entity).unwrap();

    let sbvr_xml = graph.export_sbvr().unwrap();

    assert!(sbvr_xml.contains("&amp;"));
    assert!(sbvr_xml.contains("&lt;"));
    assert!(sbvr_xml.contains("&gt;"));
}

#[test]
fn test_multiple_entities_sbvr() {
    let mut graph = Graph::new();

    for i in 1..=5 {
        let entity = Entity::new_with_namespace(format!("Entity{}", i), "test");
        graph.add_entity(entity).unwrap();
    }

    let sbvr_xml = graph.export_sbvr().unwrap();

    assert!(sbvr_xml.contains("Entity1"));
    assert!(sbvr_xml.contains("Entity5"));
}

#[test]
fn test_multiple_flows_rdf() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("E1", "test");
    let entity2 = Entity::new_with_namespace("E2", "test");
    let entity3 = Entity::new_with_namespace("E3", "test");
    let resource = Resource::new_with_namespace("R", unit_from_string("kg"), "test");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let entity3_id = entity3.id().clone();
    let resource_id = resource.id().clone();

    graph.add_entity(entity1).unwrap();
    graph.add_entity(entity2).unwrap();
    graph.add_entity(entity3).unwrap();
    graph.add_resource(resource).unwrap();

    let flow1 = Flow::new(
        resource_id.clone(),
        entity1_id,
        entity2_id.clone(),
        Decimal::new(10, 0),
    );
    let flow2 = Flow::new(resource_id, entity2_id, entity3_id, Decimal::new(20, 0));

    graph.add_flow(flow1).unwrap();
    graph.add_flow(flow2).unwrap();

    let rdf = graph.export_rdf("turtle").unwrap();

    let flow_count = rdf.matches("rdf:type").count();
    assert!(flow_count >= 5);
}

#[test]
fn test_empty_graph_exports() {
    let graph = Graph::new();

    let sbvr = graph.export_sbvr().unwrap();
    assert!(sbvr.contains("<sbvr:Vocabulary"));

    let rdf = graph.export_rdf("turtle").unwrap();
    assert!(rdf.contains("@prefix sea:"));
}
