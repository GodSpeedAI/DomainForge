//! `reduce_binary_expression` (policy/quantifier.rs) previously had no match
//! arm for `+`/`-`/`*`/`/` on two literal operands: its catch-all returned the
//! *unreduced* `Binary` expression rather than an error. `is_true_literal`
//! then read that non-literal as `false`, so every row silently failed an
//! arithmetic `where` predicate and a `count(...) = 0` invariant reported
//! satisfied regardless of the actual data.
//!
//! These tests fix that at the source (arithmetic on two numeric literals now
//! reduces to a numeric literal) and prove the invariant now evaluates
//! correctly instead of silently passing.

use domainforge_core::parser::parse_to_graph;

const SOURCE_TEMPLATE: &str = r#"
@namespace "arithmetic_where_test"

export entity "Row" {
    key row_id: string (min_length 1)
    a: int
    b: int
    total: int
}

instance row1 of "Row" {
    row_id: "R1",
    a: 2,
    b: 3,
    total: %TOTAL%
}

Policy sum_matches_total as:
    count(i in entity_instances where i.entity = "Row" and i.a + i.b != i.total: i.row_id) = 0
"#;

fn graph_with_total(total: i64) -> domainforge_core::Graph {
    let source = SOURCE_TEMPLATE.replace("%TOTAL%", &total.to_string());
    parse_to_graph(&source).expect("fixture source parses to a graph")
}

#[test]
fn arithmetic_where_predicate_rejects_violating_data() {
    // a + b = 5, total is wrongly declared as 6: the invariant is violated
    // and must NOT be reported as satisfied.
    let graph = graph_with_total(6);
    let policy = &graph.all_policies()[0];
    let result = policy
        .evaluate(&graph)
        .expect("policy evaluates without error");
    assert!(
        !result.is_satisfied,
        "count(... i.a + i.b != i.total ...) = 0 must be false when a row violates the sum, \
         not silently satisfied"
    );
}

#[test]
fn arithmetic_where_predicate_accepts_conforming_data() {
    // a + b = 5 = total: the invariant genuinely holds.
    let graph = graph_with_total(5);
    let policy = &graph.all_policies()[0];
    let result = policy
        .evaluate(&graph)
        .expect("policy evaluates without error");
    assert!(
        result.is_satisfied,
        "count(... i.a + i.b != i.total ...) = 0 must be true when every row's sum is correct"
    );
}

/// 0.1 + 0.2 != 0.3 under naive f64 arithmetic (binary floating point cannot
/// represent 0.1 or 0.2 exactly), so this invariant would have been
/// (wrongly) reported as violated if `reduce_binary_expression` computed
/// arithmetic in f64 rather than `rust_decimal::Decimal`, the same type
/// `fold_numeric` already uses for aggregation to avoid this exact problem.
#[test]
fn arithmetic_where_predicate_is_exact_for_decimal_fields() {
    let source = r#"
@namespace "arithmetic_decimal_precision_test"

export entity "Row" {
    key row_id: string (min_length 1)
    a: decimal
    b: decimal
    total: decimal
}

instance row1 of "Row" {
    row_id: "R1",
    a: 0.1,
    b: 0.2,
    total: 0.3
}

Policy sum_matches_total as:
    count(i in entity_instances where i.entity = "Row" and i.a + i.b != i.total: i.row_id) = 0
"#;
    let graph = parse_to_graph(source).expect("fixture source parses to a graph");
    let policy = &graph.all_policies()[0];
    let result = policy
        .evaluate(&graph)
        .expect("policy evaluates without error");
    assert!(
        result.is_satisfied,
        "0.1 + 0.2 = 0.3 must hold exactly under Decimal arithmetic, not report a false \
         violation from f64 rounding"
    );
}
