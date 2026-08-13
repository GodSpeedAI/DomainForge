//! The graph has long supported an entity-to-role binding
//! (`Graph::entity_roles`, `Graph::assign_role_to_entity`,
//! `Graph::roles_for_entity`), but no `.sea` text syntax authored it — the
//! `in <identifier>` clause on an entity or role declaration sets a
//! namespace, not a role binding, so `entity_roles` was always empty coming
//! out of the parser (limitation L3). `role_binding "Role" for "Entity"`
//! exposes the existing graph capability through text.

use domainforge_core::application::resolve_semantic_envelope;
use domainforge_core::parser::parse_to_graph;
use serde_json::json;

const SOURCE: &str = r#"
@namespace "role_binding_test"

role "Approver"
role "Reviewer"

export entity "Case" {
    key case_id: string (min_length 1)
}

role_binding "Approver" for "Case"
role_binding "Reviewer" for "Case"
"#;

#[test]
fn role_binding_populates_entity_roles() {
    let graph = parse_to_graph(SOURCE).expect("source with role_binding parses");

    let case = graph
        .all_entities()
        .into_iter()
        .find(|e| e.name() == "Case")
        .expect("Case entity exists");
    let approver = graph
        .all_roles()
        .into_iter()
        .find(|r| r.name() == "Approver")
        .expect("Approver role exists");
    let reviewer = graph
        .all_roles()
        .into_iter()
        .find(|r| r.name() == "Reviewer")
        .expect("Reviewer role exists");

    let bound = graph
        .roles_for_entity(case.id())
        .expect("Case has bound roles");
    assert_eq!(bound.len(), 2, "both role_binding declarations must bind");
    assert!(bound.contains(approver.id()));
    assert!(bound.contains(reviewer.id()));

    let names = graph.role_names_for_entity(case.id());
    assert!(names.contains(&"Approver".to_string()));
    assert!(names.contains(&"Reviewer".to_string()));
}

#[test]
fn role_binding_to_undeclared_entity_is_rejected() {
    let source = r#"
@namespace "role_binding_dangling_entity_test"

role "Approver"

role_binding "Approver" for "MissingEntity"
"#;
    let err = parse_to_graph(source)
        .expect_err("a role_binding referencing an undeclared entity must be rejected");
    let message = err.to_string();
    assert!(
        message.contains("MissingEntity"),
        "error must name the undefined entity: {message}"
    );
}

#[test]
fn role_binding_to_undeclared_role_is_rejected() {
    let source = r#"
@namespace "role_binding_dangling_role_test"

export entity "Case" {
    key case_id: string (min_length 1)
}

role_binding "MissingRole" for "Case"
"#;
    let err = parse_to_graph(source)
        .expect_err("a role_binding referencing an undeclared role must be rejected");
    let message = err.to_string();
    assert!(
        message.contains("MissingRole"),
        "error must name the undefined role: {message}"
    );
}

#[test]
fn role_binding_is_idempotent_for_a_repeated_pair() {
    // Graph::assign_role_to_entity already dedupes a repeated pair; the
    // text syntax must not surface that as an error, since re-declaring the
    // same binding is a harmless no-op, not a conflict.
    let source = r#"
@namespace "role_binding_repeat_test"

role "Approver"

export entity "Case" {
    key case_id: string (min_length 1)
}

role_binding "Approver" for "Case"
role_binding "Approver" for "Case"
"#;
    let graph = parse_to_graph(source).expect("a repeated identical binding must not error");
    let case = graph
        .all_entities()
        .into_iter()
        .find(|e| e.name() == "Case")
        .expect("Case entity exists");
    let bound = graph
        .roles_for_entity(case.id())
        .expect("Case has bound roles");
    assert_eq!(
        bound.len(),
        1,
        "a repeated identical binding must not duplicate"
    );
}

#[test]
fn distinct_role_entity_pairs_with_embedded_colons_get_distinct_envelope_ids() {
    // The envelope's role_binding declaration id used to interpolate the
    // raw role/entity names as "role_binding:{role}:{entity}", so a colon
    // inside either name could make two genuinely distinct pairs collide on
    // the same id: role "A:B" for "C" and role "A" for "B:C" both produced
    // "role_binding:A:B:C". The id must now be built from an unambiguous
    // encoding of the resolved (role, entity) pair.
    let first = json!({
        "main.sea": r#"
@namespace "role_binding_colon_test_one"

role "A:B"

export entity "C" {
    key id: string (min_length 1)
}

role_binding "A:B" for "C"
"#
    })
    .to_string();
    let second = json!({
        "main.sea": r#"
@namespace "role_binding_colon_test_two"

role "A"

export entity "B:C" {
    key id: string (min_length 1)
}

role_binding "A" for "B:C"
"#
    })
    .to_string();

    let doc_one = resolve_semantic_envelope("main.sea", &first).expect("first source resolves");
    let doc_two = resolve_semantic_envelope("main.sea", &second).expect("second source resolves");

    let id_one = doc_one
        .envelope
        .semantic_declarations
        .iter()
        .find(|d| {
            matches!(
                &d.declaration,
                domainforge_core::application::envelope::CanonicalSemanticPayload::RoleBinding(_)
            )
        })
        .expect("first envelope has a role_binding declaration")
        .id
        .clone();
    let id_two = doc_two
        .envelope
        .semantic_declarations
        .iter()
        .find(|d| {
            matches!(
                &d.declaration,
                domainforge_core::application::envelope::CanonicalSemanticPayload::RoleBinding(_)
            )
        })
        .expect("second envelope has a role_binding declaration")
        .id
        .clone();

    assert_ne!(
        id_one, id_two,
        "role \"A:B\" for \"C\" and role \"A\" for \"B:C\" must not collide on the same \
         declaration id just because they share namespaces and their raw names contain colons"
    );
}
