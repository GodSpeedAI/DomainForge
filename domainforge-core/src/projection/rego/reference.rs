//! Reference evaluation of the decision contract.
//!
//! Per-policy results come from DomainForge's own evaluator
//! (`Policy::evaluate_with_mode`), not from a second implementation: the
//! policy's collections are rebound to the facts document as literal arrays,
//! which `Expression::get_collection` already accepts. The differential suite
//! compares this against OPA running the generated modules.

use super::ir::{PolicyIr, RegoPolicyIr};
use crate::graph::Graph;
use crate::policy::{Expression, Policy};
use serde_json::{json, Map, Value};

pub const DECISION_CONTRACT: &str = "domainforge.rego.decision/v1";

/// The facts document for a graph: exactly the items the evaluator ranges
/// over, for the collections the projected policies use.
pub fn facts_from_graph(graph: &Graph, ir: &RegoPolicyIr) -> Result<Value, String> {
    let mut facts = Map::new();
    for name in &ir.collections {
        let items = Expression::get_collection(&Expression::Variable(name.clone()), graph)?;
        facts.insert(name.clone(), Value::Array(items));
    }
    Ok(json!({ "facts": facts }))
}

/// Input-contract violations, in the order and wording `decision.rego` uses.
pub fn input_errors(input: &Value, ir: &RegoPolicyIr) -> Vec<String> {
    let Some(facts) = input.get("facts").and_then(Value::as_object) else {
        return vec!["facts: expected object".to_string()];
    };
    ir.collections
        .iter()
        .filter(|name| {
            !facts
                .get(*name)
                .and_then(Value::as_array)
                .is_some_and(|items| items.iter().all(Value::is_object))
        })
        .map(|name| format!("facts.{name}: expected array of objects"))
        .collect()
}

fn rebind(expr: &Expression, facts: &Map<String, Value>) -> Expression {
    let collection = |c: &Expression| match c {
        Expression::Variable(name) => {
            let (base, entity_type) = match name.strip_prefix("entity_instances:") {
                Some(ty) => ("entity_instances", Some(ty)),
                None => (name.as_str(), None),
            };
            let items = facts
                .get(base)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let items = match entity_type {
                Some(ty) => items
                    .into_iter()
                    .filter(|item| item.get("entity").and_then(Value::as_str) == Some(ty))
                    .collect(),
                None => items,
            };
            Expression::Literal(Value::Array(items))
        }
        other => other.clone(),
    };
    match expr {
        Expression::Quantifier {
            quantifier,
            variable,
            collection: c,
            condition,
        } => Expression::Quantifier {
            quantifier: quantifier.clone(),
            variable: variable.clone(),
            collection: Box::new(collection(c)),
            condition: Box::new(rebind(condition, facts)),
        },
        Expression::Binary { op, left, right } => {
            Expression::binary(op.clone(), rebind(left, facts), rebind(right, facts))
        }
        Expression::Unary { op, operand } => Expression::unary(op.clone(), rebind(operand, facts)),
        Expression::Aggregation {
            function,
            collection: c,
            field,
            filter,
        } => Expression::Aggregation {
            function: function.clone(),
            collection: Box::new(collection(c)),
            field: field.clone(),
            filter: filter.clone(),
        },
        Expression::AggregationComprehension {
            function,
            variable,
            collection: c,
            window,
            predicate,
            projection,
            target_unit,
        } => Expression::AggregationComprehension {
            function: function.clone(),
            variable: variable.clone(),
            collection: Box::new(collection(c)),
            window: window.clone(),
            predicate: Box::new(rebind(predicate, facts)),
            projection: projection.clone(),
            target_unit: target_unit.clone(),
        },
        other => other.clone(),
    }
}

/// One policy's result on a facts document: `true`, `false`, `unknown`
/// (the evaluator's NULL) or `error` (the evaluator returned `Err`).
pub fn evaluate_policy(policy: &PolicyIr, facts: &Map<String, Value>) -> &'static str {
    let rebound = Policy::new(policy.name.clone(), rebind(&policy.source, facts));
    match rebound.evaluate_with_mode(&Graph::new(), true) {
        Ok(result) => match result.is_satisfied_tristate {
            Some(true) => "true",
            Some(false) => "false",
            None => "unknown",
        },
        Err(_) => "error",
    }
}

/// The full decision document for an input, as `data.domainforge.policy.decision`
/// must produce it.
pub fn decide(ir: &RegoPolicyIr, identity: &Value, input: &Value) -> Value {
    let input_errors = input_errors(input, ir);
    let malformed = !input_errors.is_empty();
    let empty = Map::new();
    let facts = input
        .get("facts")
        .and_then(Value::as_object)
        .unwrap_or(&empty);

    let mut results = Map::new();
    let mut rows: Vec<(i64, &str, Value)> = Vec::new();
    for policy in &ir.policies {
        let result = if malformed {
            "error"
        } else {
            evaluate_policy(policy, facts)
        };
        results.insert(policy.key.clone(), json!(result));
        if result == "true" {
            continue;
        }
        let code = match result {
            _ if malformed => "malformed_input",
            "false" => "violated",
            "unknown" => "unknown",
            _ => "evaluation_error",
        };
        rows.push((
            -i64::from(policy.priority),
            &policy.key,
            json!({
                "policy": policy.key,
                "code": code,
                "modality": policy.modality,
                "severity": policy.severity,
                "priority": policy.priority,
                "blocking": policy.severity == "error",
            }),
        ));
    }
    rows.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    let reasons: Vec<Value> = rows.into_iter().map(|row| row.2).collect();

    let blocking = |code: &str| {
        reasons
            .iter()
            .any(|r| r["blocking"] == json!(true) && r["code"] == json!(code))
    };
    let outcome = if malformed
        || blocking("violated")
        || blocking("evaluation_error")
        || blocking("malformed_input")
    {
        "deny"
    } else if blocking("unknown") {
        "escalate"
    } else {
        "allow"
    };

    json!({
        "contract": DECISION_CONTRACT,
        "outcome": outcome,
        "results": results,
        "reasons": reasons,
        "input_errors": input_errors,
        "model": identity,
    })
}
