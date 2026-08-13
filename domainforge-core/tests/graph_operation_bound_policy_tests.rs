//! A policy referenced by an operation's `access policy_governed by <name>
//! at precondition ...` clause is meant to be evaluated in that operation's
//! typed contract context (its terms may be input-record fields, which do
//! not exist at the graph level). `Graph::validate()` previously evaluated
//! every declared policy against the bare graph regardless, so such a
//! policy always returned `UNKNOWN (NULL)` and failed validation — even
//! DomainForge's own flagship fixture
//! (`fixtures/application_generation/flagship/command-write.sea`) failed
//! `domainforge validate` for this reason (limitation L9).
//!
//! `Graph::validate()` now skips policies named by an
//! `AccessPolicyGoverned` binding; `--application`/`contract`/`envelope`
//! check them in the operation's own context instead.

use domainforge_core::parser::parse_to_graph;

const OPERATION_BOUND_POLICY_SOURCE: &str = r#"
@namespace "operation_bound_policy_test"

policy order_total_within_limit as: total <= 10000

export entity "Order" {
    key order_id: string (min_length 1)
    total: int
}

record PlaceOrderInput {
    order_id: string
    total: int
}

record PlaceOrderOutput {
    order_id: string
}

operation place_order {
    intent "persist one valid order"
    direction inbound
    actor anonymous
    access policy_governed by order_total_within_limit at precondition fails with order_limit_exceeded
    input PlaceOrderInput
    output PlaceOrderOutput
    state Order
    effect creates Order
    transaction single_aggregate
    failure order_limit_exceeded for policy "order total exceeds 10000"
    idempotency inherent
    concurrency unique_key order_id
    evidence operation_trace
    lifecycle synchronous_request_response
}
"#;

#[test]
fn operation_bound_policy_is_excluded_from_graph_validation() {
    let graph = parse_to_graph(OPERATION_BOUND_POLICY_SOURCE)
        .expect("source with an operation-bound policy still parses to a graph");

    // Two policies are declared (the bare `policy` statement plus none
    // implicit), but only the operation-bound one exists here, and it must
    // not be evaluated against the bare graph, where `total` does not
    // resolve.
    let result = graph.validate();
    assert_eq!(
        result.error_count, 0,
        "an operation-bound policy must not fail graph validation with UNKNOWN: {:?}",
        result.violations
    );
}

#[test]
fn a_second_graph_evaluable_policy_still_validates_independently() {
    let source = format!(
        "{OPERATION_BOUND_POLICY_SOURCE}\n\npolicy at_least_one_order as: count(i in entity_instances where i.entity = \"Order\": i.order_id) >= 1\n\ninstance order1 of \"Order\" {{ order_id: \"O1\", total: 500 }}\n"
    );
    let graph = parse_to_graph(&source).expect("source parses to a graph");

    let result = graph.validate();
    assert_eq!(
        result.error_count, 0,
        "a genuinely graph-evaluable policy alongside an operation-bound one must still pass \
         when its own condition holds: {:?}",
        result.violations
    );
}

#[test]
fn a_second_graph_evaluable_policy_still_catches_a_real_violation() {
    let source = format!(
        "{OPERATION_BOUND_POLICY_SOURCE}\n\npolicy at_least_one_order as: count(i in entity_instances where i.entity = \"Order\": i.order_id) >= 1\n"
    );
    let graph = parse_to_graph(&source).expect("source parses to a graph");

    // No `Order` instance declared: the graph-evaluable policy must still
    // catch this — coexisting with an operation-bound policy in the same
    // file must not weaken unrelated graph-level checks.
    let result = graph.validate();
    assert!(
        result.error_count > 0,
        "a violated graph-evaluable policy must still be reported even when an \
         operation-bound policy is present in the same file"
    );
}
