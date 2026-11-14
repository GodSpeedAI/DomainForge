//! WP-10 spike: three-valued per-policy evaluation through the PUBLIC library API only.
//! Throwaway; not for merge.

use domainforge_core::application::{
    evaluate_precondition, resolve_application_contract, ApplicationPolicyContext,
    EvaluationResult as AppResult,
};
use domainforge_core::policy::{Policy, PolicyKind};
use domainforge_core::primitives::Instance;
use domainforge_core::{parse_to_graph, Graph};
use serde_json::json;

const MODEL: &str = include_str!("../../spike/wp10/model.sea");
const NS: &str = "wp10";

fn graph() -> Graph {
    parse_to_graph(MODEL).expect("model parses")
}

fn inst(g: &mut Graph, name: &str, ty: &str, fields: serde_json::Value) {
    let mut i = Instance::new_with_namespace(name, ty, NS);
    for (k, v) in fields.as_object().unwrap() {
        i.set_field(k.clone(), v.clone());
    }
    g.add_entity_instance(i)
        .unwrap_or_else(|e| panic!("add {name}: {e}"));
}

/// Per-policy tri-state through public API. Skips operation-bound policies exactly as
/// `Graph::validate` does (they are evaluated via a different path, see case d).
fn tri(g: &Graph, name: &str) -> Option<bool> {
    let p = policy(g, name);
    p.evaluate(g).expect("evaluates").is_satisfied_tristate
}

fn policy<'a>(g: &'a Graph, name: &str) -> &'a Policy {
    g.all_policies()
        .into_iter()
        .find(|p| p.name == name)
        .expect("policy")
}

#[test]
fn case_a_empty_fact_set() {
    let g = graph();
    println!(
        "policies: {:?}",
        g.all_policies()
            .iter()
            .map(|p| (&p.name, p.kind().clone()))
            .collect::<Vec<_>>()
    );
    for n in ["executed_steps_have_approval", "evidence_retained_30_days"] {
        let r = policy(&g, n).evaluate(&g).unwrap();
        println!(
            "EMPTY {n}: tristate={:?} violations={}",
            r.is_satisfied_tristate,
            r.violations.len()
        );
        // Vacuous truth: universal over zero instances is TRUE, with no support count.
        assert_eq!(r.is_satisfied_tristate, Some(true));
    }
    assert!(matches!(
        policy(&g, "executed_steps_have_approval").kind(),
        PolicyKind::Constraint
    ));
    let v = g.validate();
    println!(
        "validate(): evaluated={} errors={}",
        v.total_policies, v.error_count
    );
}

#[test]
fn case_b_satisfying_and_violating() {
    // satisfying: executed step with an approval
    let mut g = graph();
    inst(
        &mut g,
        "s1",
        "Step",
        json!({"step_id": "s1", "status": "executed"}),
    );
    inst(
        &mut g,
        "a1",
        "Approval",
        json!({"approval_id": "a1", "step_ref": "s1"}),
    );
    println!("SAT: {:?}", tri(&g, "executed_steps_have_approval"));
    assert_eq!(tri(&g, "executed_steps_have_approval"), Some(true));

    // violating: executed step without approval
    let mut g = graph();
    inst(
        &mut g,
        "s1",
        "Step",
        json!({"step_id": "s1", "status": "executed"}),
    );
    inst(
        &mut g,
        "s2",
        "Step",
        json!({"step_id": "s2", "status": "planned"}),
    );
    let r = policy(&g, "executed_steps_have_approval")
        .evaluate(&g)
        .unwrap();
    println!(
        "VIOL: {:?} violations={:?}",
        r.is_satisfied_tristate, r.violations
    );
    assert_eq!(r.is_satisfied_tristate, Some(false));
    // Bound instances: the violation context is the only candidate carrier.
    assert_eq!(
        r.violations[0].context,
        json!({}),
        "no bound instances in violation context"
    );
}

#[test]
fn case_b2_mixed_instances_report_one_policy_level_result_only() {
    // one satisfying + one violating Step: result is a single policy-level False; no per-binding results.
    let mut g = graph();
    inst(
        &mut g,
        "s1",
        "Step",
        json!({"step_id": "s1", "status": "executed"}),
    );
    inst(
        &mut g,
        "a1",
        "Approval",
        json!({"approval_id": "a1", "step_ref": "s1"}),
    );
    inst(
        &mut g,
        "s2",
        "Step",
        json!({"step_id": "s2", "status": "executed"}),
    );
    let r = policy(&g, "executed_steps_have_approval")
        .evaluate(&g)
        .unwrap();
    println!(
        "MIXED: {:?} n_violations={} msg={}",
        r.is_satisfied_tristate,
        r.violations.len(),
        r.violations[0].message
    );
    assert_eq!(r.is_satisfied_tristate, Some(false));
    assert_eq!(
        r.violations.len(),
        1,
        "one policy-level violation, not one per failing binding"
    );
}

#[test]
fn case_c_unknown_from_missing_attribute() {
    let mut g = graph();
    inst(&mut g, "e1", "Evidence", json!({"evidence_id": "e1"})); // retention_days absent
    let r = policy(&g, "evidence_retained_30_days")
        .evaluate(&g)
        .unwrap();
    println!(
        "UNKNOWN: {:?} violations={:?}",
        r.is_satisfied_tristate, r.violations
    );
    assert_eq!(r.is_satisfied_tristate, None);
    assert!(
        !r.is_satisfied,
        "unknown collapses to is_satisfied=false in the bool field"
    );

    // known-bad beats unknown under Kleene forall (False dominates Null)
    inst(
        &mut g,
        "e2",
        "Evidence",
        json!({"evidence_id": "e2", "retention_days": 5}),
    );
    println!("UNKNOWN+FALSE: {:?}", tri(&g, "evidence_retained_30_days"));

    // all known good
    let mut g = graph();
    inst(
        &mut g,
        "e1",
        "Evidence",
        json!({"evidence_id": "e1", "retention_days": 90}),
    );
    assert_eq!(tri(&g, "evidence_retained_30_days"), Some(true));
}

#[test]
fn case_d_operation_bound_policy() {
    let g = graph();
    let p = policy(&g, "case_must_be_open");
    // (1) bound policy is detectable via public API
    assert!(g.is_operation_bound_policy(&p.id));
    // (2) Graph::validate skips it
    assert_eq!(g.validate().total_policies, 2);
    // (3) evaluating against the bare graph is UNKNOWN (the L9 limitation)
    let bare = p.evaluate(&g).unwrap();
    println!("BARE-GRAPH: {:?}", bare.is_satisfied_tristate);
    assert_eq!(bare.is_satisfied_tristate, None);

    // (4) the application evaluator is public and three-valued, with a typed context
    let sources = json!({ "model.sea": MODEL }).to_string();
    let doc = resolve_application_contract("model.sea", &sources).expect("contract resolves");
    let op = &doc.contract.operations[0];
    println!("operation id={} access={:?}", op.id.0, op.access);
    let expr = p.expression();
    let ctx = |input: serde_json::Value| -> ApplicationPolicyContext {
        serde_json::from_value(json!({"operation": op.id.0, "actor": {"kind":"role","data":{"role": serde_json::to_value(domainforge_core::concept_id::ConceptId::from_concept(NS, "Clerk")).unwrap()}}, "input": input})).unwrap()
    };
    let t = evaluate_precondition(
        expr,
        &ctx(json!({"case_status": {"kind":"string","data":"open"}})),
    );
    let f = evaluate_precondition(
        expr,
        &ctx(json!({"case_status": {"kind":"string","data":"closed"}})),
    );
    let u = evaluate_precondition(expr, &ctx(json!({})));
    println!("OP-BOUND: open={t:?} closed={f:?} missing={u:?}");
    assert_eq!(t, AppResult::True);
    assert_eq!(f, AppResult::False);
    assert_eq!(u, AppResult::Unknown);
}

#[test]
fn facts_as_sea_source_round_trip() {
    // Facts expressed as `instance` declarations appended to the model text (library-only).
    let src =
        format!("{MODEL}\ninstance s1 of \"Step\" {{ step_id: \"s1\", status: \"executed\" }}\n");
    let g = parse_to_graph(&src).expect("model+facts parse");
    assert_eq!(g.entity_instance_count(), 1);
    assert_eq!(tri(&g, "executed_steps_have_approval"), Some(false));
}
