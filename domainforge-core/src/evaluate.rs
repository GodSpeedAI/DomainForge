//! Per-binding, three-valued policy evaluation of a model against a facts file
//! (`domainforge evaluate`).
//!
//! No evaluation semantics live here. Graph policies are evaluated by
//! [`Policy::evaluate_bindings`] (the existing three-valued evaluator);
//! operation-bound policies by [`evaluate_precondition`] (the application
//! evaluator). This module only loads facts, routes each policy to its
//! evaluator and shapes the result.

use crate::application::{
    evaluate_precondition, AccessMode, ApplicationContractDocument, ApplicationPolicyContext,
    EvaluationResult as AppResult, FieldType, ScalarType, TypedValue,
};
use crate::graph::Graph;
use crate::policy::Policy;
use crate::primitives::Instance;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashSet};

pub const FACTS_SCHEMA: &str = "domainforge.evaluate-facts.v1";
pub const OUTPUT_SCHEMA: &str = "domainforge.evaluate-output.v1";

/// Parsed `domainforge.evaluate-facts.v1` document.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub schema: String,
    #[serde(default)]
    pub instances: Vec<FactInstance>,
    #[serde(default)]
    pub operation_inputs: Vec<OperationInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactInstance {
    pub namespace: String,
    pub entity: String,
    pub name: String,
    #[serde(default)]
    pub fields: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationInput {
    pub id: String,
    pub operation: String,
    /// Typed values exactly as the application layer expects them
    /// (`{"kind": "string", "data": "open"}`).
    #[serde(default)]
    pub input: IndexMap<String, TypedValue>,
}

impl Facts {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let facts: Facts =
            serde_json::from_str(text).map_err(|e| format!("invalid facts file: {e}"))?;
        if facts.schema != FACTS_SCHEMA {
            return Err(format!(
                "unsupported facts schema '{}': expected '{FACTS_SCHEMA}'",
                facts.schema
            ));
        }
        Ok(facts)
    }
}

fn kind_matches(declared: &FieldType, value: &TypedValue) -> bool {
    match (declared, value) {
        (FieldType::Scalar { scalar }, v) => matches!(
            (scalar, v),
            (ScalarType::String, TypedValue::String(_))
                | (ScalarType::Int, TypedValue::Int(_))
                | (ScalarType::Decimal, TypedValue::Decimal(_))
                | (ScalarType::Bool, TypedValue::Bool(_))
                | (ScalarType::Timestamp, TypedValue::Timestamp(_))
                | (ScalarType::Uuid, TypedValue::Uuid(_))
        ),
        (FieldType::Quantity { .. }, TypedValue::Quantity { .. }) => true,
        (FieldType::EntityRef { .. }, TypedValue::EntityRef { .. }) => true,
        (FieldType::Enum { .. }, TypedValue::Enum { .. }) => true,
        (FieldType::List { element }, TypedValue::List(items)) => {
            items.iter().all(|i| kind_matches(element, i))
        }
        _ => false,
    }
}

fn tri(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

/// Add the fact instances to `graph`. Unknown entities, unknown or mistyped
/// fields and duplicate names are errors, never silently dropped.
pub fn load_instances(graph: &mut Graph, facts: &Facts) -> Result<(), String> {
    // Insert every fact, then validate once: a fact may reference a fact that
    // appears later in the file. The batch is all-or-nothing.
    let instances = facts
        .instances
        .iter()
        .map(|fact| {
            let mut instance =
                Instance::new_with_namespace(&fact.name, &fact.entity, &fact.namespace);
            for (key, value) in &fact.fields {
                instance.set_field(key.clone(), value.clone());
            }
            instance
        })
        .collect();
    graph
        .add_entity_instances(instances)
        .map_err(|e| format!("facts instances: {e}"))
}

fn policy_head(policy: &Policy) -> Map<String, Value> {
    let mut row = Map::new();
    row.insert("policy_id".into(), json!(policy.id.to_string()));
    row.insert("policy".into(), json!(policy.name));
    row.insert("namespace".into(), json!(policy.namespace));
    row.insert(
        "kind".into(),
        json!(format!("{:?}", policy.kind()).to_lowercase()),
    );
    row.insert(
        "modality".into(),
        json!(format!("{:?}", policy.modality).to_lowercase()),
    );
    row
}

/// Evaluate every policy of `graph` (after [`load_instances`]) and every
/// operation-bound policy against the supplied operation inputs. Rows are
/// ordered by policy id, then operation name; bindings by instance name.
pub fn evaluate_model(
    graph: &Graph,
    contract: &ApplicationContractDocument,
    facts: &Facts,
) -> Result<Vec<Value>, String> {
    if !graph.use_three_valued_logic() {
        return Err(
            "evaluate requires three-valued logic; the graph is in boolean mode".to_string(),
        );
    }
    let operations = &contract.contract.operations;
    let records = &contract.contract.records;

    let mut seen_ids = BTreeSet::new();
    for supplied in &facts.operation_inputs {
        if !seen_ids.insert(supplied.id.as_str()) {
            return Err(format!("duplicate operation input id '{}'", supplied.id));
        }
        let mut matching = operations.iter().filter(|op| op.name == supplied.operation);
        let operation = matching.next().ok_or_else(|| {
            format!(
                "operation input '{}': unknown operation '{}'",
                supplied.id, supplied.operation
            )
        })?;
        if matching.next().is_some() {
            return Err(format!(
                "operation input '{}': ambiguous operation '{}'",
                supplied.id, supplied.operation
            ));
        }
        let governed = matches!(&operation.access,
            AccessMode::PolicyGoverned { bindings } if !bindings.is_empty());
        if !governed {
            return Err(format!(
                "operation input '{}': operation '{}' is not policy-governed, so no policy would evaluate it",
                supplied.id, supplied.operation
            ));
        }
        let record = records.iter().find(|r| r.id == operation.input);
        for (field, value) in &supplied.input {
            let declared = record.and_then(|r| r.fields.iter().find(|f| &f.name == field));
            let Some(declared) = declared else {
                return Err(format!(
                    "operation input '{}': unknown input field '{}' for operation '{}'",
                    supplied.id, field, supplied.operation
                ));
            };
            if !kind_matches(&declared.field_type, value) {
                return Err(format!(
                    "operation input '{}': field '{}' has the wrong value kind for its declared type {:?}",
                    supplied.id, field, declared.field_type
                ));
            }
        }
    }

    let fact_names: HashSet<&str> = facts.instances.iter().map(|i| i.name.as_str()).collect();
    let mut bound_ids = HashSet::new();
    let mut rows: Vec<(String, String, Value)> = Vec::new();

    for operation in operations {
        let AccessMode::PolicyGoverned { bindings } = &operation.access else {
            continue;
        };
        for binding in bindings {
            bound_ids.insert(binding.policy.clone());
            let Some(policy) = graph.get_policy(&binding.policy) else {
                return Err(format!(
                    "operation '{}' binds a policy that is not in the graph",
                    operation.name
                ));
            };
            let mut supplied: Vec<&OperationInput> = facts
                .operation_inputs
                .iter()
                .filter(|s| s.operation == operation.name)
                .collect();
            supplied.sort_by(|a, b| a.id.cmp(&b.id));
            if supplied.is_empty() {
                let mut row = policy_head(policy);
                row.insert("operation_bound".into(), json!(true));
                row.insert("operation".into(), json!(operation.name));
                row.insert("result".into(), json!("unknown"));
                row.insert("reason".into(), json!("no_operation_input"));
                rows.push((
                    policy.id.to_string(),
                    format!("{}\u{0}", operation.name),
                    Value::Object(row),
                ));
            }
            for input in &supplied {
                let context = ApplicationPolicyContext {
                    operation: operation.id.clone(),
                    actor: operation.actor.clone(),
                    input: input.input.clone(),
                    pre_state: None,
                };
                let mut row = policy_head(policy);
                row.insert("operation_bound".into(), json!(true));
                row.insert("operation".into(), json!(operation.name));
                row.insert("input_set_id".into(), json!(input.id));
                let result = match evaluate_precondition(policy.expression(), &context) {
                    AppResult::True => Some(true),
                    AppResult::False => Some(false),
                    AppResult::Unknown => {
                        row.insert("reason".into(), json!("unknown_input"));
                        None
                    }
                    AppResult::Error(message) => {
                        row.insert("reason".into(), json!("evaluation_error"));
                        row.insert("error".into(), json!(message));
                        None
                    }
                };
                row.insert("result".into(), json!(tri(result)));
                rows.push((
                    policy.id.to_string(),
                    format!("{}\u{0}{}", operation.name, input.id),
                    Value::Object(row),
                ));
            }
        }
    }

    for policy in graph.all_policies() {
        if bound_ids.contains(&policy.id) || graph.is_operation_bound_policy(&policy.id) {
            continue;
        }
        let mut row = policy_head(policy);
        row.insert("operation_bound".into(), json!(false));
        match policy.evaluate_bindings(graph) {
            Ok(report) => {
                row.insert("result".into(), json!(tri(report.result)));
                row.insert("support".into(), json!(report.support));
                if let Some(caveat) = report.caveat {
                    row.insert("caveat".into(), json!(caveat));
                    row.insert(
                        "evaluator_result".into(),
                        json!(tri(report.evaluator_result)),
                    );
                }
                if report.support.is_some() {
                    row.insert("binding_depth".into(), json!("outermost"));
                }
                let bindings: Vec<Value> = report
                    .bindings
                    .iter()
                    .map(|b| {
                        let from_facts = fact_names.contains(b.instance.as_str());
                        json!({
                            "source": if from_facts { "facts" } else { "model" },
                            "entity": b.entity,
                            "namespace": b.namespace,
                            "instance": b.instance,
                            "instance_id": b.instance_id,
                            "result": tri(b.result),
                        })
                    })
                    .collect();
                row.insert("bindings".into(), Value::Array(bindings));
            }
            Err(message) => {
                row.insert("result".into(), json!("unknown"));
                row.insert("support".into(), Value::Null);
                row.insert("reason".into(), json!("evaluation_error"));
                row.insert("error".into(), json!(message));
                row.insert("bindings".into(), json!([]));
            }
        }
        rows.push((policy.id.to_string(), String::new(), Value::Object(row)));
    }

    rows.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    Ok(rows.into_iter().map(|(_, _, row)| row).collect())
}
