use super::models::{
    CalmModel, CalmNode, CalmRelationship, FlowDetails, NodeType, Parties, RelationshipType,
};
use crate::patterns::Pattern;
use crate::policy::Policy;
use crate::policy::{AggregateFunction, BinaryOp, Expression, Quantifier, UnaryOp};
use crate::primitives::{Entity, Flow, Metric, Resource, ResourceInstance};
use crate::Graph;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;

fn export_entity(entity: &Entity) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Entity"));

    let ser = serde_json::to_value(entity).unwrap_or(Value::Null);
    let attrs = ser["attributes"].clone();
    metadata.insert("sea:attributes".to_string(), attrs);

    CalmNode {
        unique_id: entity.id().to_string(),
        node_type: NodeType::Actor,
        name: entity.name().to_string(),
        namespace: Some(entity.namespace().to_string()),
        metadata,
    }
}

fn export_resource(resource: &Resource) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Resource"));
    metadata.insert("sea:unit".to_string(), json!(resource.unit().symbol()));

    let ser = serde_json::to_value(resource).unwrap_or(Value::Null);
    let attrs = ser["attributes"].clone();
    metadata.insert("sea:attributes".to_string(), attrs);

    CalmNode {
        unique_id: resource.id().to_string(),
        node_type: NodeType::Resource,
        name: resource.name().to_string(),
        namespace: Some(resource.namespace().to_string()),
        metadata,
    }
}

fn export_instance(instance: &ResourceInstance) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Instance"));
    metadata.insert(
        "sea:entity_id".to_string(),
        json!(instance.entity_id().to_string()),
    );
    metadata.insert(
        "sea:resource_id".to_string(),
        json!(instance.resource_id().to_string()),
    );

    let ser = serde_json::to_value(instance).unwrap_or(Value::Null);
    let attrs = ser["attributes"].clone();
    metadata.insert("sea:attributes".to_string(), attrs);

    CalmNode {
        unique_id: instance.id().to_string(),
        node_type: NodeType::Instance,
        name: format!("Instance of {}", instance.id()),
        namespace: Some(instance.namespace().to_string()),
        metadata,
    }
}

fn export_pattern(pattern: &Pattern) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Pattern"));
    metadata.insert("sea:regex".to_string(), json!(pattern.regex()));

    CalmNode {
        unique_id: pattern.id().to_string(),
        node_type: NodeType::Constraint,
        name: pattern.name().to_string(),
        namespace: Some(pattern.namespace().to_string()),
        metadata,
    }
}

fn export_metric(metric: &Metric) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Metric"));
    metadata.insert(
        "sea:expression".to_string(),
        json!(serialize_expression_for_export(&metric.expression)),
    );
    metadata.insert("sea:metric_type".to_string(), json!("Metric"));

    if let Some(ri) = &metric.refresh_interval {
        metadata.insert(
            "sea:refresh_interval".to_string(),
            json!(ri.num_seconds()),
        );
    }
    if let Some(u) = &metric.unit {
        metadata.insert("sea:unit".to_string(), json!(u));
    }
    if let Some(t) = &metric.threshold {
        metadata.insert("sea:threshold".to_string(), json!(t.to_string()));
    }
    if let Some(s) = &metric.severity {
        metadata.insert("sea:severity".to_string(), json!(format!("{:?}", s)));
    }
    if let Some(t) = &metric.target {
        metadata.insert("sea:target".to_string(), json!(t.to_string()));
    }
    if let Some(w) = &metric.window {
        metadata.insert("sea:window".to_string(), json!(w.num_seconds()));
    }

    CalmNode {
        unique_id: metric.id.to_string(),
        node_type: NodeType::Constraint,
        name: metric.name.clone(),
        namespace: Some(metric.namespace.clone()),
        metadata,
    }
}

fn export_flow(flow: &Flow) -> CalmRelationship {
    CalmRelationship {
        unique_id: flow.id().to_string(),
        relationship_type: RelationshipType::Flow {
            flow: FlowDetails {
                resource: flow.resource_id().to_string(),
                quantity: flow.quantity().to_string(),
            },
        },
        parties: Parties::SourceDestination {
            source: flow.from_id().to_string(),
            destination: flow.to_id().to_string(),
        },
    }
}

pub fn export(graph: &Graph) -> Result<Value, String> {
    let mut calm_model = CalmModel::new();

    calm_model.metadata.sea_timestamp = Some(Utc::now().to_rfc3339());

    for entity in graph.all_entities() {
        calm_model.nodes.push(export_entity(entity));
    }

    for resource in graph.all_resources() {
        calm_model.nodes.push(export_resource(resource));
    }

    for instance in graph.all_instances() {
        calm_model.nodes.push(export_instance(instance));
    }

    for pattern in graph.all_patterns() {
        calm_model.nodes.push(export_pattern(pattern));
    }

    for metric in graph.all_metrics() {
        calm_model.nodes.push(export_metric(metric));
    }

    for flow in graph.all_flows() {
        calm_model.relationships.push(export_flow(flow));
    }

    // Export policies as constraint nodes
    for policy in graph.all_policies() {
        calm_model.nodes.push(export_policy(policy));
    }

    // Export associations as Simple relationships when recorded on an entity
    for entity in graph.all_entities() {
        if let Some(Value::Array(arr)) = entity.get_attribute("associations") {
            for entry in arr {
                if let (Some(rel_type), Some(target)) = (
                    entry.get("type").and_then(|v| v.as_str()),
                    entry.get("target").and_then(|v| v.as_str()),
                ) {
                    calm_model.relationships.push(CalmRelationship {
                        unique_id: format!("assoc-{}-{}", entity.id(), target),
                        relationship_type: RelationshipType::Simple(rel_type.to_string()),
                        parties: Parties::SourceDestination {
                            source: entity.id().to_string(),
                            destination: target.to_string(),
                        },
                    });
                }
            }
        }
    }

    serde_json::to_value(&calm_model).map_err(|e| format!("Failed to serialize CALM model: {}", e))
}

fn export_policy(policy: &Policy) -> CalmNode {
    let mut metadata = HashMap::new();
    metadata.insert("sea:primitive".to_string(), json!("Policy"));
    metadata.insert(
        "sea:expression".to_string(),
        json!(serialize_expression_for_export(&policy.expression)),
    );
    metadata.insert("sea:expression_type".to_string(), json!("SEA"));
    metadata.insert("sea:priority".to_string(), json!(policy.priority));
    metadata.insert(
        "sea:modality".to_string(),
        json!(format!("{:?}", policy.modality)),
    );
    metadata.insert("sea:kind".to_string(), json!(format!("{:?}", policy.kind)));

    CalmNode {
        unique_id: policy.id.to_string(),
        node_type: NodeType::Constraint,
        name: policy.name.clone(),
        namespace: Some(policy.namespace.clone()),
        metadata,
    }
}

fn serialize_expression_for_export(expr: &Expression) -> String {
    match expr {
        Expression::Literal(v) => v.to_string(),
        Expression::QuantityLiteral { value, unit } => format!("{} \"{}\"", value, unit),
        Expression::TimeLiteral(timestamp) => format!("\"{}\"", timestamp),
        Expression::IntervalLiteral { start, end } => {
            format!("interval(\"{}\", \"{}\")", start, end)
        }
        Expression::Variable(s) => s.to_string(),
        Expression::Binary { op, left, right } => {
            let op_str = match op {
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
                BinaryOp::Equal => "=",
                BinaryOp::NotEqual => "!=",
                BinaryOp::GreaterThan => ">",
                BinaryOp::LessThan => "<",
                BinaryOp::GreaterThanOrEqual => ">=",
                BinaryOp::LessThanOrEqual => "<=",
                BinaryOp::Plus => "+",
                BinaryOp::Minus => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Contains => "contains",
                BinaryOp::StartsWith => "startswith",
                BinaryOp::EndsWith => "endswith",
                BinaryOp::HasRole => "has_role",
                BinaryOp::Matches => "matches",
                BinaryOp::Before => "before",
                BinaryOp::After => "after",
                BinaryOp::During => "during",
            };
            format!(
                "({} {} {})",
                serialize_expression_for_export(left),
                op_str,
                serialize_expression_for_export(right)
            )
        }
        Expression::Unary { op, operand } => {
            let op_str = match op {
                UnaryOp::Not => "not",
                UnaryOp::Negate => "-",
            };
            format!("{} {}", op_str, serialize_expression_for_export(operand))
        }
        Expression::Quantifier {
            quantifier,
            variable,
            collection,
            condition,
        } => {
            let q_str = match quantifier {
                Quantifier::ForAll => "forall",
                Quantifier::Exists => "exists",
                Quantifier::ExistsUnique => "exists_unique",
            };
            format!(
                "{} {} in {}: ({})",
                q_str,
                variable,
                serialize_expression_for_export(collection),
                serialize_expression_for_export(condition)
            )
        }
        Expression::MemberAccess { object, member } => format!("{}.{}", object, member),
        Expression::Aggregation {
            function,
            collection,
            field,
            filter,
        } => {
            let fn_str = match function {
                AggregateFunction::Count => "count",
                AggregateFunction::Sum => "sum",
                AggregateFunction::Min => "min",
                AggregateFunction::Max => "max",
                AggregateFunction::Avg => "avg",
            };
            if let Some(fld) = field {
                if let Some(flt) = filter {
                    format!(
                        "{}({}.{} where {})",
                        fn_str,
                        serialize_expression_for_export(collection),
                        fld,
                        serialize_expression_for_export(flt)
                    )
                } else {
                    format!(
                        "{}({}.{} )",
                        fn_str,
                        serialize_expression_for_export(collection),
                        fld
                    )
                }
            } else if let Some(flt) = filter {
                format!(
                    "{}({} where {})",
                    fn_str,
                    serialize_expression_for_export(collection),
                    serialize_expression_for_export(flt)
                )
            } else {
                format!(
                    "{}({})",
                    fn_str,
                    serialize_expression_for_export(collection)
                )
            }
        }
        Expression::AggregationComprehension {
            function,
            variable,
            collection,
            window,
            predicate,
            projection,
            target_unit,
        } => {
            let fn_str = match function {
                AggregateFunction::Count => "count",
                AggregateFunction::Sum => "sum",
                AggregateFunction::Min => "min",
                AggregateFunction::Max => "max",
                AggregateFunction::Avg => "avg",
            };
            let window_str = if let Some(w) = window {
                format!(" over last {} \"{}\"", w.duration, w.unit)
            } else {
                "".to_string()
            };
            let predicate_str = serialize_expression_for_export(predicate);
            let where_clause = if matches!(**predicate, Expression::Literal(Value::Bool(true))) {
                "".to_string()
            } else {
                format!(" WHERE {}", predicate_str)
            };
            let projection_str = serialize_expression_for_export(projection);
            let collection_str = serialize_expression_for_export(collection);
            if let Some(unit) = target_unit {
                format!(
                    "{}({} in {}{}{}: {} as \"{}\")",
                    fn_str,
                    variable,
                    collection_str,
                    window_str,
                    where_clause,
                    projection_str,
                    unit
                )
            } else {
                format!(
                    "{}({} in {}{}{}: {})",
                    fn_str, variable, collection_str, window_str, where_clause, projection_str
                )
            }
        }
        Expression::GroupBy {
            variable,
            collection,
            filter,
            key,
            condition,
        } => {
            let filter_str = if let Some(f) = filter {
                format!(" where {}", serialize_expression_for_export(f))
            } else {
                "".to_string()
            };
            format!(
                "group_by({} in {}{}: {}) {{ {} }}",
                variable,
                serialize_expression_for_export(collection),
                filter_str,
                serialize_expression_for_export(key),
                serialize_expression_for_export(condition)
            )
        }
    }
}
