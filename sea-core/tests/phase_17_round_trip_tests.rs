use rust_decimal::Decimal;
use sea_core::graph::Graph;
use sea_core::kg::KnowledgeGraph;
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::sbvr::SbvrModel;
use sea_core::units::unit_from_string;

#[test]
fn test_sbvr_export_import_entities() {
    let mut graph = Graph::new();

    let entity1 = Entity::new_with_namespace("Supplier", "supply_chain");
    let entity2 = Entity::new_with_namespace("Manufacturer", "supply_chain");

    graph.add_entity(entity1.clone()).unwrap();
    graph.add_entity(entity2.clone()).unwrap();

    let sbvr_xml = graph.export_sbvr().unwrap();

    assert!(sbvr_xml.contains("Supplier"));
    assert!(sbvr_xml.contains("Manufacturer"));
    assert_eq!(graph.entity_count(), 2);
}

#[test]
fn test_sbvr_export_preserves_entity_count() {
    let mut original = Graph::new();

    for i in 1..=5 {
        let entity = Entity::new_with_namespace(format!("Entity{}", i), "test");
        original.add_entity(entity).unwrap();
    }

    let sbvr_xml = original.export_sbvr().unwrap();

    let entity_count = sbvr_xml.matches("<sbvr:GeneralConcept").count();
    assert_eq!(entity_count, 5);
}

#[test]
fn test_sbvr_export_preserves_resource_count() {
    let mut original = Graph::new();

    for i in 1..=3 {
        let resource =
            Resource::new_with_namespace(format!("Resource{}", i), unit_from_string("kg"), "test");
        original.add_resource(resource).unwrap();
    }

    let sbvr_xml = original.export_sbvr().unwrap();

    let resource_count = sbvr_xml.matches("<sbvr:IndividualConcept").count();
    assert_eq!(resource_count, 3);
}

#[test]
fn test_sbvr_export_preserves_flow_count() {
    let mut original = Graph::new();

    let entity1 = Entity::new_with_namespace("E1", "test");
    let entity2 = Entity::new_with_namespace("E2", "test");
    let resource = Resource::new_with_namespace("R", unit_from_string("kg"), "test");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let resource_id = resource.id().clone();

    original.add_entity(entity1).unwrap();
    original.add_entity(entity2).unwrap();
    original.add_resource(resource).unwrap();

    for i in 1..=4 {
        let flow = Flow::new(
            resource_id.clone(),
            entity1_id.clone(),
            entity2_id.clone(),
            Decimal::new(i * 10, 0),
        );
        original.add_flow(flow).unwrap();
    }

    let sbvr_xml = original.export_sbvr().unwrap();

    let flow_count = sbvr_xml.matches("<sbvr:FactType").count();
    assert_eq!(flow_count, 4);
}

#[test]
fn test_rdf_export_preserves_entity_count() {
    let mut original = Graph::new();

    for i in 1..=5 {
        let entity = Entity::new_with_namespace(format!("Entity{}", i), "test");
        original.add_entity(entity).unwrap();
    }

    let rdf = original.export_rdf("turtle").unwrap();

    let entity_type_count = rdf.matches("rdf:type sea:Entity").count();
    assert_eq!(entity_type_count, 5);
}

#[test]
fn test_rdf_export_preserves_resource_count() {
    let mut original = Graph::new();

    for i in 1..=3 {
        let resource =
            Resource::new_with_namespace(format!("Resource{}", i), unit_from_string("kg"), "test");
        original.add_resource(resource).unwrap();
    }

    let rdf = original.export_rdf("turtle").unwrap();

    let resource_type_count = rdf.matches("rdf:type sea:Resource").count();
    assert_eq!(resource_type_count, 3);
}

#[test]
fn test_rdf_export_preserves_flow_relationships() {
    let mut original = Graph::new();

    let entity1 = Entity::new_with_namespace("Sender", "test");
    let entity2 = Entity::new_with_namespace("Receiver", "test");
    let resource = Resource::new_with_namespace("Material", unit_from_string("kg"), "test");

    let entity1_id = entity1.id().clone();
    let entity2_id = entity2.id().clone();
    let resource_id = resource.id().clone();

    original.add_entity(entity1).unwrap();
    original.add_entity(entity2).unwrap();
    original.add_resource(resource).unwrap();

    let flow = Flow::new(resource_id, entity1_id, entity2_id, Decimal::new(100, 0));
    original.add_flow(flow).unwrap();

    let rdf = original.export_rdf("turtle").unwrap();

    assert!(rdf.contains("sea:from"));
    assert!(rdf.contains("sea:to"));
    assert!(rdf.contains("sea:hasResource"));
    assert!(rdf.contains("sea:quantity"));
}

#[test]
fn test_sbvr_model_round_trip_structure() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Company", "business");
    let resource = Resource::new_with_namespace("Money", unit_from_string("USD"), "finance");

    graph.add_entity(entity).unwrap();
    graph.add_resource(resource).unwrap();

    let model = SbvrModel::from_graph(&graph).unwrap();

    assert!(model.vocabulary.len() >= 2);

    let xmi = model.to_xmi().unwrap();
    assert!(xmi.contains("Company"));
    assert!(xmi.contains("Money"));
}

#[test]
fn test_knowledge_graph_round_trip_structure() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("Organization", "org");
    graph.add_entity(entity).unwrap();

    let kg = KnowledgeGraph::from_graph(&graph).unwrap();

    assert!(!kg.triples.is_empty());

    let turtle = kg.to_turtle();
    assert!(turtle.contains("Organization"));
}

#[test]
fn test_export_formats_consistency() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("TestEntity", "test");
    graph.add_entity(entity).unwrap();

    let sbvr = graph.export_sbvr().unwrap();
    let rdf_turtle = graph.export_rdf("turtle").unwrap();
    let rdf_xml = graph.export_rdf("rdf-xml").unwrap();

    assert!(sbvr.contains("TestEntity"));
    assert!(rdf_turtle.contains("TestEntity"));
    assert!(rdf_xml.contains("TestEntity"));
}

#[test]
fn test_complex_graph_export() {
    let mut graph = Graph::new();

    let e1 = Entity::new_with_namespace("A", "test");
    let e2 = Entity::new_with_namespace("B", "test");
    let e3 = Entity::new_with_namespace("C", "test");
    let r1 = Resource::new_with_namespace("X", unit_from_string("kg"), "test");
    let r2 = Resource::new_with_namespace("Y", unit_from_string("units"), "test");

    let e1_id = e1.id().clone();
    let e2_id = e2.id().clone();
    let e3_id = e3.id().clone();
    let r1_id = r1.id().clone();
    let r2_id = r2.id().clone();

    graph.add_entity(e1).unwrap();
    graph.add_entity(e2).unwrap();
    graph.add_entity(e3).unwrap();
    graph.add_resource(r1).unwrap();
    graph.add_resource(r2).unwrap();

    let f1 = Flow::new(r1_id, e1_id.clone(), e2_id.clone(), Decimal::new(10, 0));
    let f2 = Flow::new(r2_id, e2_id, e3_id, Decimal::new(20, 0));

    graph.add_flow(f1).unwrap();
    graph.add_flow(f2).unwrap();

    let sbvr = graph.export_sbvr().unwrap();
    let rdf = graph.export_rdf("turtle").unwrap();

    assert_eq!(graph.entity_count(), 3);
    assert_eq!(graph.resource_count(), 2);
    assert_eq!(graph.flow_count(), 2);

    assert!(sbvr.contains("sbvr:FactType"));
    assert!(rdf.contains("sea:Flow"));
}

#[test]
fn test_unicode_names_in_export() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("北京公司", "china");
    let resource = Resource::new_with_namespace("Société", unit_from_string("units"), "france");

    graph.add_entity(entity).unwrap();
    graph.add_resource(resource).unwrap();

    let sbvr = graph.export_sbvr().unwrap();
    let rdf = graph.export_rdf("turtle").unwrap();

    assert!(sbvr.contains("北京公司"));
    assert!(rdf.contains("北京公司"));
}

#[test]
fn test_special_characters_escaping() {
    let mut graph = Graph::new();

    let entity = Entity::new_with_namespace("A&B<C>", "test");
    graph.add_entity(entity).unwrap();

    let sbvr = graph.export_sbvr().unwrap();

    assert!(sbvr.contains("&amp;"));
    assert!(sbvr.contains("&lt;"));
    assert!(sbvr.contains("&gt;"));
}
