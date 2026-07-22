//! Typed operation-policy subset validation (APP007) and the fail-closed
//! application evaluator (plan Task 9; reference §4).

use domainforge_core::application::{
    evaluate_precondition, resolve_application_contract, ApplicationDiagnostic,
    ApplicationPolicyContext, EvaluationResult,
};
use domainforge_core::concept_id::ConceptId;
use domainforge_core::policy::{BinaryOp, Expression};
use serde_json::json;

// ---- construction-time subset validation (APP007) ----

fn resolve_single(source: &str) -> Result<(), Vec<ApplicationDiagnostic>> {
    let sources = json!({ "main.sea": source }).to_string();
    resolve_application_contract("main.sea", &sources).map(|_| ())
}

fn codes(err: &[ApplicationDiagnostic]) -> Vec<String> {
    err.iter()
        .map(|d| {
            serde_json::to_value(d.code)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

fn policy_op(policy_decl: &str, binding: &str, enforcement: &str) -> String {
    format!(
        "@namespace \"t\"\nrole \"Customer\"\n{policy_decl}\n\
entity \"E\" {{\n    key id: uuid\n    total: int\n}}\n\
record In {{\n    id: uuid\n    total: int\n}}\n\
record Out {{\n    id: uuid\n}}\n\
operation op {{\n    intent \"write\"\n    direction inbound\n    actor Customer\n    access policy_governed by {binding} at {enforcement} fails with limit\n    input In\n    output Out\n    state E\n    effect creates E\n    transaction single_aggregate\n    failure limit for policy \"limit\"\n    failure conflict for idempotency_conflict, concurrency_conflict \"dup\"\n    idempotency keyed_by id\n    concurrency unique_key id\n    evidence operation_trace\n    lifecycle synchronous_request_response\n}}\n"
    )
}

const CONSTRAINT_POLICY: &str = "policy p per Constraint Obligation priority 1 as: total <= 10000";

#[test]
fn evaluable_constraint_policy_binding_resolves() {
    resolve_single(&policy_op(CONSTRAINT_POLICY, "p", "precondition")).unwrap();
}

#[test]
fn actor_role_equality_policy_resolves() {
    let policy = "policy p per Constraint Obligation priority 1 as: actor = role<Customer>";
    resolve_single(&policy_op(policy, "p", "precondition")).unwrap();
}

#[test]
fn unknown_input_field_in_policy_is_app007() {
    let policy = "policy p per Constraint Obligation priority 1 as: missing > 0";
    let err = resolve_single(&policy_op(policy, "p", "precondition")).unwrap_err();
    assert_eq!(codes(&err), ["APP007"]);
}

#[test]
fn non_constraint_policy_binding_is_app007() {
    let policy = "policy p per Derivation Obligation priority 1 as: total <= 10000";
    let err = resolve_single(&policy_op(policy, "p", "precondition")).unwrap_err();
    assert_eq!(codes(&err), ["APP007"]);
}

#[test]
fn dangling_policy_binding_is_app007() {
    let err = resolve_single(&policy_op(CONSTRAINT_POLICY, "nope", "precondition")).unwrap_err();
    assert_eq!(codes(&err), ["APP007"]);
}

#[test]
fn reserved_enforcement_point_is_app007() {
    let err = resolve_single(&policy_op(CONSTRAINT_POLICY, "p", "invariant")).unwrap_err();
    assert_eq!(codes(&err), ["APP007"]);
}

#[test]
fn misplaced_role_reference_is_app007() {
    let policy = "policy p per Constraint Obligation priority 1 as: total = role<Customer>";
    let err = resolve_single(&policy_op(policy, "p", "precondition")).unwrap_err();
    assert_eq!(codes(&err), ["APP007"]);
}

// ---- runtime evaluation ----

fn usd() -> serde_json::Value {
    serde_json::to_value(ConceptId::from_concept("flagship.orders", "USD")).unwrap()
}

fn context(input: serde_json::Value, actor: serde_json::Value) -> ApplicationPolicyContext {
    serde_json::from_value(json!({
        "operation": "flagship.orders.operation.place_order",
        "actor": actor,
        "input": input,
    }))
    .unwrap()
}

fn total_context(total: Option<&str>) -> ApplicationPolicyContext {
    let input = match total {
        Some(value) => json!({
            "total": {"kind": "quantity", "data": {"base_value": value, "unit": usd()}}
        }),
        None => json!({}),
    };
    context(input, json!({"kind": "anonymous"}))
}

fn limit_expression() -> Expression {
    Expression::binary(
        BinaryOp::LessThanOrEqual,
        Expression::variable("total"),
        Expression::literal(10000),
    )
}

#[test]
fn flagship_limit_uses_quantity_unit_and_fails_closed() {
    let expr = limit_expression();
    assert_eq!(
        evaluate_precondition(&expr, &total_context(Some("10000"))),
        EvaluationResult::True
    );
    assert_eq!(
        evaluate_precondition(&expr, &total_context(Some("10000.01"))),
        EvaluationResult::False
    );
    assert_eq!(
        evaluate_precondition(&expr, &total_context(None)),
        EvaluationResult::Unknown
    );
}

#[test]
fn actor_role_equality_evaluates_three_valued() {
    let customer = ConceptId::from_concept("t", "Customer");
    let expr = Expression::binary(
        BinaryOp::Equal,
        Expression::variable("actor"),
        Expression::RoleReference {
            role: serde_json::to_value(&customer)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        },
    );
    let with_role = |role: &ConceptId| {
        context(
            json!({}),
            json!({"kind": "role", "data": {"role": serde_json::to_value(role).unwrap()}}),
        )
    };
    assert_eq!(
        evaluate_precondition(&expr, &with_role(&customer)),
        EvaluationResult::True
    );
    assert_eq!(
        evaluate_precondition(&expr, &with_role(&ConceptId::from_concept("t", "Admin"))),
        EvaluationResult::False
    );
    // Anonymous callers make actor.role UNKNOWN.
    assert_eq!(
        evaluate_precondition(&expr, &context(json!({}), json!({"kind": "anonymous"}))),
        EvaluationResult::Unknown
    );
}

#[test]
fn state_member_access_is_unknown_without_pre_state() {
    let expr = Expression::binary(
        BinaryOp::Equal,
        Expression::member_access("state", "n"),
        Expression::literal(1),
    );
    let with_state: ApplicationPolicyContext = serde_json::from_value(json!({
        "operation": "t.operation.op",
        "actor": {"kind": "anonymous"},
        "input": {},
        "pre_state": {"n": {"kind": "int", "data": 1}},
    }))
    .unwrap();
    assert_eq!(
        evaluate_precondition(&expr, &with_state),
        EvaluationResult::True
    );
    assert_eq!(
        evaluate_precondition(&expr, &total_context(None)),
        EvaluationResult::Unknown
    );
}

#[test]
fn division_by_zero_is_an_error() {
    let expr = Expression::binary(
        BinaryOp::GreaterThan,
        Expression::binary(
            BinaryOp::Divide,
            Expression::literal(1),
            Expression::literal(0),
        ),
        Expression::literal(0),
    );
    assert!(matches!(
        evaluate_precondition(&expr, &total_context(None)),
        EvaluationResult::Error(_)
    ));
}

#[test]
fn mismatched_quantity_units_are_an_error() {
    let eur = serde_json::to_value(ConceptId::from_concept("flagship.orders", "EUR")).unwrap();
    let ctx = context(
        json!({
            "a": {"kind": "quantity", "data": {"base_value": "1", "unit": usd()}},
            "b": {"kind": "quantity", "data": {"base_value": "1", "unit": eur}},
        }),
        json!({"kind": "anonymous"}),
    );
    let expr = Expression::binary(
        BinaryOp::Equal,
        Expression::variable("a"),
        Expression::variable("b"),
    );
    assert!(matches!(
        evaluate_precondition(&expr, &ctx),
        EvaluationResult::Error(_)
    ));
}
