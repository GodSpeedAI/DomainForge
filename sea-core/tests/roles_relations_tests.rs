use sea_core::kg::KnowledgeGraph;
use sea_core::parser::parse_to_graph;
use sea_core::sbvr::SbvrModel;

#[test]
fn parses_roles_relations_and_projects() {
    let dsl = r#"
        Role "Payer"
        Role "Payee"

        Resource "Money" units

        Entity "Alice"
        Entity "Bob"

        Flow "Money" from "Alice" to "Bob" quantity 10

        Relation "Payment"
          subject: "Payer"
          predicate: "pays"
          object: "Payee"
          via: flow "Money"

        Policy role_check as:
          exists e in entities: (e has_role "Payer")
    "#;

    let mut graph = parse_to_graph(dsl).expect("should parse role and relation declarations");

    assert_eq!(graph.role_count(), 2);
    assert_eq!(graph.relation_count(), 1);

    let payer_role = graph
        .find_role_by_name("Payer")
        .expect("payer role should exist");
    let alice = graph
        .find_entity_by_name("Alice")
        .expect("alice entity should exist");

    graph
        .assign_role_to_entity(alice, payer_role)
        .expect("role assignment should succeed");

    let validation = graph.validate();
    assert_eq!(
        validation.error_count, 0,
        "policy should pass once role assigned"
    );

    let kg = KnowledgeGraph::from_graph(&graph).expect("kg projection should succeed");
    assert!(kg
        .triples
        .iter()
        .any(|triple| triple.predicate == "sea:subjectRole" && triple.object.contains("Payer")));

    let sbvr = SbvrModel::from_graph(&graph).expect("sbvr projection should succeed");
    assert!(sbvr
        .facts
        .iter()
        .any(|fact| fact.verb == "pays" && fact.object.contains("Payee")));
}
