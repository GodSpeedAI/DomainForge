//! D3 shared identity proof (M0 gate findings 2 and 12): Graph and contract
//! resolve one multi-namespace closure to identical `ConceptId`s, with no
//! first-module namespace leakage and no specifier suffix-matching.

use domainforge_core::application::{
    resolve_application_contract, resolve_application_graph, FieldType,
};
use domainforge_core::concept_id::ConceptId;
use serde_json::json;

/// Diamond: app -> {b, c} -> d, four distinct namespaces, one duplicate
/// entity name ("Dup") in two namespaces, aliased named imports.
fn diamond_sources() -> String {
    json!({
        "app.sea": "@namespace \"app\"\nimport { Dup as BDup } from \"./b.sea\"\nimport { Dup as GDup } from \"./c.sea\"\nimport { Shared } from \"./d.sea\"\n\nentity \"Main\" {\n    key id: uuid\n    bref: ref<BDup>\n    gref: ref<GDup>\n    shared: ref<Shared>\n}\n",
        "b.sea": "@namespace \"beta\"\nimport { Shared } from \"./d.sea\"\n\nexport entity \"Dup\" {\n    key id: uuid\n    shared: ref<Shared>\n}\n",
        "c.sea": "@namespace \"gamma\"\nimport { Shared } from \"./d.sea\"\n\nexport entity \"Dup\" {\n    key id: uuid\n}\n",
        "d.sea": "@namespace \"delta\"\n\nexport entity \"Shared\" {\n    key id: uuid\n}\nrole \"Auditor\"\n",
    })
    .to_string()
}

#[test]
fn graph_and_contract_share_resolved_identity_across_namespaces() {
    let sources = diamond_sources();
    let contract = resolve_application_contract("app.sea", &sources)
        .expect("diamond contract resolves")
        .contract;
    let graph = resolve_application_graph("app.sea", &sources).expect("diamond graph resolves");

    let expected = [
        ConceptId::from_concept("app", "Main"),
        ConceptId::from_concept("beta", "Dup"),
        ConceptId::from_concept("gamma", "Dup"),
        ConceptId::from_concept("delta", "Shared"),
    ];
    for id in &expected {
        assert!(
            contract.entities.iter().any(|e| e.concept_id == *id),
            "contract is missing entity {id}"
        );
        assert!(
            graph.get_entity(id).is_some(),
            "graph is missing entity {id}"
        );
    }
    assert!(
        graph
            .get_role(&ConceptId::from_concept("delta", "Auditor"))
            .is_some(),
        "graph is missing the delta role"
    );
}

#[test]
fn no_first_module_namespace_leaks_into_shared_identity() {
    let graph =
        resolve_application_graph("app.sea", &diamond_sources()).expect("diamond graph resolves");
    // Under the old first-module merge every declaration adopted "app".
    for name in ["Dup", "Shared"] {
        assert!(
            graph
                .get_entity(&ConceptId::from_concept("app", name))
                .is_none(),
            "entity '{name}' leaked into the entry module namespace"
        );
    }
    assert!(graph
        .get_role(&ConceptId::from_concept("app", "Auditor"))
        .is_none());
}

#[test]
fn aliased_imports_resolve_to_exact_target_identities() {
    let contract = resolve_application_contract("app.sea", &diamond_sources())
        .expect("diamond contract resolves")
        .contract;
    let main = contract
        .entities
        .iter()
        .find(|e| e.concept_id == ConceptId::from_concept("app", "Main"))
        .expect("Main entity present");
    let entity_ref = |field: &str| -> ConceptId {
        let f = main.fields.iter().find(|f| f.name == field).unwrap();
        let FieldType::EntityRef { entity } = &f.field_type else {
            panic!("field '{field}' is not an entity ref: {:?}", f.field_type);
        };
        entity.clone()
    };
    assert_eq!(entity_ref("bref"), ConceptId::from_concept("beta", "Dup"));
    assert_eq!(entity_ref("gref"), ConceptId::from_concept("gamma", "Dup"));
    assert_eq!(
        entity_ref("shared"),
        ConceptId::from_concept("delta", "Shared")
    );
}

#[test]
fn same_suffix_modules_resolve_through_recorded_edges() {
    // Two modules whose logical paths share the suffix "orders/x.sea"; the
    // old suffix matcher could bind either. The recorded edge is exact.
    let sources = json!({
        "main.sea": "@namespace \"m\"\nimport { A } from \"./one/orders/x.sea\"\nimport { B } from \"./two/orders/x.sea\"\n\nentity \"Root\" {\n    key id: uuid\n    a: ref<A>\n    b: ref<B>\n}\n",
        "one/orders/x.sea": "@namespace \"one\"\nexport entity \"A\" { key id: uuid }\n",
        "two/orders/x.sea": "@namespace \"two\"\nexport entity \"B\" { key id: uuid }\n",
    })
    .to_string();
    let contract = resolve_application_contract("main.sea", &sources)
        .expect("suffix-sharing closure resolves")
        .contract;
    let root = contract
        .entities
        .iter()
        .find(|e| e.concept_id == ConceptId::from_concept("m", "Root"))
        .expect("Root present");
    let ids: Vec<&ConceptId> = root
        .fields
        .iter()
        .filter_map(|f| match &f.field_type {
            FieldType::EntityRef { entity } => Some(entity),
            _ => None,
        })
        .collect();
    assert_eq!(
        ids,
        [
            &ConceptId::from_concept("one", "A"),
            &ConceptId::from_concept("two", "B"),
        ]
    );
}
