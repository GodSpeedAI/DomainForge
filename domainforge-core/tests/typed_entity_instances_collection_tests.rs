//! `forall`/`exists` over the untyped `entity_instances` collection
//! evaluates to `UNKNOWN (NULL)` whenever some item lacks a field the
//! condition references, because a field absent on a differently-typed
//! instance substitutes to NULL and comparisons against NULL are neither
//! true nor false. The existing workaround —
//! `count(i in entity_instances where i.entity = "X": i.field) = N` — still
//! works and is not going away; `entity_instances of "EntityType"` adds a
//! second, more direct way to say the same scoping for genuine universal
//! quantification, where the aggregation workaround doesn't fit as
//! naturally (limitation L6).

use domainforge_core::parser::parse_to_graph;

const TWO_ENTITY_TYPES: &str = r#"
@namespace "typed_collection_test"

export entity "Journey" {
    key journey_id: string (min_length 1)
    rank: int
}

export entity "Surface" {
    key surface_id: string (min_length 1)
    label: string
}

instance j1 of "Journey" { journey_id: "J1", rank: 1 }
instance j2 of "Journey" { journey_id: "J2", rank: 2 }
instance s1 of "Surface" { surface_id: "S1", label: "web" }
"#;

#[test]
fn untyped_forall_over_heterogeneous_instances_is_null_not_false() {
    // `Surface` instances have no `rank` field, so an untyped forall over
    // `entity_instances` sees NULL for them and the whole quantifier
    // evaluates to NULL, not a clean true/false — this is the documented
    // status quo the typed form exists to give authors an alternative to,
    // not a regression this change introduces.
    let source = format!(
        "{TWO_ENTITY_TYPES}\npolicy every_rank_positive as: forall i in entity_instances: (i.rank > 0)\n"
    );
    let graph = parse_to_graph(&source).expect("source parses to a graph");
    let policy = &graph.all_policies()[0];
    let eval = policy
        .evaluate(&graph)
        .expect("evaluation itself does not error");
    assert!(
        !eval.is_satisfied,
        "an untyped forall referencing a field absent on another entity type must not be \
         reported as satisfied (it is NULL, not true)"
    );
}

#[test]
fn typed_forall_scopes_to_one_entity_type_and_evaluates_cleanly() {
    let source = format!(
        "{TWO_ENTITY_TYPES}\npolicy every_journey_rank_positive as: forall i in entity_instances of \"Journey\": (i.rank > 0)\n"
    );
    let graph = parse_to_graph(&source).expect("source with typed collection parses");
    let policy = &graph.all_policies()[0];
    let eval = policy
        .evaluate(&graph)
        .expect("typed forall evaluates without error");
    assert!(
        eval.is_satisfied,
        "scoped to Journey only, every rank is positive: the typed collection must let this \
         evaluate cleanly instead of NULL"
    );
}

#[test]
fn typed_forall_still_catches_a_real_violation_within_its_type() {
    let source = format!(
        "{TWO_ENTITY_TYPES}\npolicy every_journey_rank_high as: forall i in entity_instances of \"Journey\": (i.rank > 1)\n"
    );
    let graph = parse_to_graph(&source).expect("source parses");
    let policy = &graph.all_policies()[0];
    let eval = policy.evaluate(&graph).expect("evaluates without error");
    assert!(
        !eval.is_satisfied,
        "J1 has rank 1, which fails > 1: the typed collection must still catch a real \
         violation within its scoped type, not just avoid NULL"
    );
}

#[test]
fn typed_count_aggregation_matches_the_existing_where_workaround() {
    let typed = format!(
        "{TWO_ENTITY_TYPES}\npolicy typed_count as: count(i in entity_instances of \"Journey\": i.journey_id) = 2\n"
    );
    let untyped = format!(
        "{TWO_ENTITY_TYPES}\npolicy untyped_count as: count(i in entity_instances where i.entity = \"Journey\": i.journey_id) = 2\n"
    );

    let typed_graph = parse_to_graph(&typed).expect("typed source parses");
    let untyped_graph = parse_to_graph(&untyped).expect("untyped source parses");

    assert!(
        typed_graph.all_policies()[0]
            .evaluate(&typed_graph)
            .expect("typed count evaluates")
            .is_satisfied,
        "entity_instances of \"Journey\" must count the same two Journey instances as the \
         existing where i.entity = \"Journey\" workaround"
    );
    assert!(
        untyped_graph.all_policies()[0]
            .evaluate(&untyped_graph)
            .expect("untyped count evaluates")
            .is_satisfied
    );
}

#[test]
fn untyped_entity_instances_collection_is_unaffected() {
    // Backward compatibility: the bare `entity_instances` collection (no
    // `of` suffix) must still return every instance regardless of type.
    let source = format!(
        "{TWO_ENTITY_TYPES}\npolicy total_instance_count as: count(i in entity_instances: i.id) = 3\n"
    );
    let graph = parse_to_graph(&source).expect("source parses");
    let policy = &graph.all_policies()[0];
    assert!(
        policy
            .evaluate(&graph)
            .expect("evaluates without error")
            .is_satisfied,
        "the untyped entity_instances collection must still see all 3 instances (2 Journey + \
         1 Surface), unaffected by the new typed form"
    );
}
