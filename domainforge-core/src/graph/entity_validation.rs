use super::Graph;
use crate::application::{FieldConstraint, FieldContract, FieldType, ScalarType};
use crate::primitives::Instance;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

pub(super) fn validate(graph: &Graph) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut keys: HashMap<crate::ConceptId, HashMap<String, String>> = HashMap::new();

    for instance in graph.entity_instances.values() {
        let Some(entity) =
            graph.find_entity_by_name_and_namespace(instance.entity_type(), instance.namespace())
        else {
            errors.push(format!(
                "Entity instance '{}' references missing entity '{}' in namespace '{}'",
                instance.name(),
                instance.entity_type(),
                instance.namespace()
            ));
            continue;
        };
        let Some(contract) = graph.entity_contracts.get(&entity) else {
            continue;
        };

        for field in &contract.fields {
            match instance.get_field(&field.name) {
                Some(value) => validate_field(graph, instance, field, value, &mut errors),
                None if !field.optional => errors.push(format!(
                    "Entity instance '{}' is missing required field '{}' of entity '{}'",
                    instance.name(),
                    field.name,
                    contract.name
                )),
                None => {}
            }
        }

        let mut unknown: Vec<&String> = instance
            .fields()
            .keys()
            .filter(|name| {
                !contract
                    .fields
                    .iter()
                    .any(|field| field.name.as_str() == name.as_str())
            })
            .collect();
        unknown.sort();
        for name in unknown {
            errors.push(format!(
                "Entity instance '{}' has unknown field '{}' for typed entity '{}'",
                instance.name(),
                name,
                contract.name
            ));
        }

        if let Some(value) = instance.get_field(&contract.key_field) {
            let canonical = serde_json::to_string(value)
                .unwrap_or_else(|_| format!("<unserializable:{value:?}>"));
            let entity_keys = keys.entry(contract.concept_id.clone()).or_default();
            if let Some(first) = entity_keys.insert(canonical.clone(), instance.name().to_string())
            {
                errors.push(format!(
                    "Entity key field '{}' must be unique for entity '{}': instances '{}' and '{}' both use {}",
                    contract.key_field, contract.name, first, instance.name(), canonical
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_field(
    graph: &Graph,
    instance: &Instance,
    field: &FieldContract,
    value: &Value,
    errors: &mut Vec<String>,
) {
    if let Err(reason) = validate_type(graph, &field.field_type, value) {
        errors.push(format!(
            "Entity instance '{}' field '{}' {}",
            instance.name(),
            field.name,
            reason
        ));
        return;
    }

    for constraint in &field.constraints {
        if let Some(reason) = constraint_violation(graph, constraint, value) {
            errors.push(format!(
                "Entity instance '{}' field '{}' violates its {} constraint: {}",
                instance.name(),
                field.name,
                constraint_slug(constraint),
                reason
            ));
        }
    }
}

fn validate_type(graph: &Graph, field_type: &FieldType, value: &Value) -> Result<(), String> {
    match field_type {
        FieldType::Scalar { scalar } => validate_scalar(*scalar, value),
        FieldType::Quantity { .. } => {
            let object = value
                .as_object()
                .ok_or_else(|| "must be a quantity object".to_string())?;
            object
                .get("value")
                .and_then(decimal_value)
                .ok_or_else(|| "must have a decimal quantity value".to_string())?;
            if object.get("unit").and_then(Value::as_str).is_none() {
                return Err("must have a string quantity unit".to_string());
            }
            Ok(())
        }
        FieldType::EntityRef { entity } => {
            let target_contract = graph.entity_contracts.get(entity);
            let found = graph.entity_instances.values().any(|candidate| {
                let Some(candidate_entity) = graph.find_entity_by_name_and_namespace(
                    candidate.entity_type(),
                    candidate.namespace(),
                ) else {
                    return false;
                };
                if &candidate_entity != entity {
                    return false;
                }
                match target_contract {
                    Some(contract) => candidate.get_field(&contract.key_field) == Some(value),
                    None => value.as_str() == Some(candidate.name()),
                }
            });
            if found {
                Ok(())
            } else {
                Err(format!(
                    "contains dangling typed reference {} to entity {}",
                    serde_json::to_string(value).unwrap_or_else(|_| format!("{value:?}")),
                    entity
                ))
            }
        }
        FieldType::Enum { symbol } => {
            let wire = value
                .as_str()
                .ok_or_else(|| format!("must be a string wire value of enum '{}'", symbol.0))?;
            let contract = graph
                .enum_contracts
                .get(symbol)
                .ok_or_else(|| format!("references missing enum contract '{}'", symbol.0))?;
            if contract.members.iter().any(|member| member.wire == wire) {
                Ok(())
            } else {
                Err(format!(
                    "value '{wire}' is not a member of enum '{}'",
                    contract.name
                ))
            }
        }
        FieldType::List { element } => {
            let values = value
                .as_array()
                .ok_or_else(|| "must be a list".to_string())?;
            for (index, value) in values.iter().enumerate() {
                validate_type(graph, element, value)
                    .map_err(|reason| format!("list element {index} {reason}"))?;
            }
            Ok(())
        }
    }
}

fn validate_scalar(scalar: ScalarType, value: &Value) -> Result<(), String> {
    let valid = match scalar {
        ScalarType::String => value.is_string(),
        ScalarType::Int => numeric_json_value(value)
            .is_some_and(|number| number.is_integer() && number.to_i64().is_some()),
        ScalarType::Decimal => numeric_json_value(value).is_some(),
        ScalarType::Bool => value.is_boolean(),
        ScalarType::Timestamp => value
            .as_str()
            .is_some_and(|text| chrono::DateTime::parse_from_rfc3339(text).is_ok()),
        ScalarType::Uuid => value
            .as_str()
            .is_some_and(|text| uuid::Uuid::parse_str(text).is_ok()),
    };
    if valid {
        Ok(())
    } else {
        Err(format!("does not satisfy its {} type", scalar_slug(scalar)))
    }
}

fn numeric_json_value(value: &Value) -> Option<Decimal> {
    match value {
        Value::Number(number) => Decimal::from_str(&number.to_string()).ok(),
        _ => None,
    }
}

fn decimal_value(value: &Value) -> Option<Decimal> {
    match value {
        Value::Number(number) => Decimal::from_str(&number.to_string()).ok(),
        Value::String(text) => Decimal::from_str(text).ok(),
        _ => None,
    }
}

fn constraint_violation(
    graph: &Graph,
    constraint: &FieldConstraint,
    value: &Value,
) -> Option<String> {
    match constraint {
        FieldConstraint::Min { value: bound } => decimal_value(value)
            .filter(|actual| actual < bound)
            .map(|actual| format!("{actual} is below {bound}")),
        FieldConstraint::Max { value: bound } => decimal_value(value)
            .filter(|actual| actual > bound)
            .map(|actual| format!("{actual} is above {bound}")),
        FieldConstraint::ExclusiveMin { value: bound } => decimal_value(value)
            .filter(|actual| actual <= bound)
            .map(|actual| format!("{actual} is not greater than {bound}")),
        FieldConstraint::ExclusiveMax { value: bound } => decimal_value(value)
            .filter(|actual| actual >= bound)
            .map(|actual| format!("{actual} is not less than {bound}")),
        FieldConstraint::MinLength { value: bound } => value.as_str().and_then(|actual| {
            let length = actual.chars().count();
            (length < *bound as usize).then(|| format!("length {length} is below {bound}"))
        }),
        FieldConstraint::MaxLength { value: bound } => value.as_str().and_then(|actual| {
            let length = actual.chars().count();
            (length > *bound as usize).then(|| format!("length {length} is above {bound}"))
        }),
        FieldConstraint::MinItems { value: bound } => value.as_array().and_then(|actual| {
            (actual.len() < *bound as usize)
                .then(|| format!("item count {} is below {bound}", actual.len()))
        }),
        FieldConstraint::MaxItems { value: bound } => value.as_array().and_then(|actual| {
            (actual.len() > *bound as usize)
                .then(|| format!("item count {} is above {bound}", actual.len()))
        }),
        FieldConstraint::Pattern { pattern } => {
            let candidate = value.as_str()?;
            match graph.patterns.get(pattern) {
                Some(pattern) => match pattern.is_match(candidate) {
                    Ok(true) => None,
                    Ok(false) => Some(format!(
                        "value '{}' does not match pattern '{}'",
                        candidate,
                        pattern.name()
                    )),
                    Err(reason) => Some(format!("pattern evaluation failed: {reason}")),
                },
                None => Some(format!("pattern {pattern} is missing from the graph")),
            }
        }
    }
}

fn scalar_slug(scalar: ScalarType) -> &'static str {
    match scalar {
        ScalarType::String => "string",
        ScalarType::Int => "int",
        ScalarType::Decimal => "decimal",
        ScalarType::Bool => "bool",
        ScalarType::Timestamp => "timestamp",
        ScalarType::Uuid => "uuid",
    }
}

fn constraint_slug(constraint: &FieldConstraint) -> &'static str {
    match constraint {
        FieldConstraint::Min { .. } => "min",
        FieldConstraint::Max { .. } => "max",
        FieldConstraint::ExclusiveMin { .. } => "exclusive_min",
        FieldConstraint::ExclusiveMax { .. } => "exclusive_max",
        FieldConstraint::MinLength { .. } => "min_length",
        FieldConstraint::MaxLength { .. } => "max_length",
        FieldConstraint::MinItems { .. } => "min_items",
        FieldConstraint::MaxItems { .. } => "max_items",
        FieldConstraint::Pattern { .. } => "pattern",
    }
}
