use super::expression::{AggregateFunction, BinaryOp, Expression, Quantifier};
use crate::graph::Graph;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::convert::TryFrom;
use std::str::FromStr;

impl Expression {
    pub fn expand(&self, graph: &Graph) -> Result<Expression, String> {
        match self {
            Expression::Quantifier {
                quantifier,
                variable,
                collection,
                condition,
            } => {
                let items = Self::get_collection(collection, graph)?;

                match quantifier {
                    Quantifier::ForAll => {
                        if items.is_empty() {
                            return Ok(Expression::literal(true));
                        }

                        let mut result = Expression::literal(true);
                        for item in items {
                            let substituted = condition.substitute(variable, &item)?;
                            let expanded = substituted.expand(graph)?;
                            result = Expression::binary(BinaryOp::And, result, expanded);
                        }
                        Ok(result)
                    }
                    Quantifier::Exists => {
                        if items.is_empty() {
                            return Ok(Expression::literal(false));
                        }

                        let mut result = Expression::literal(false);
                        for item in items {
                            let substituted = condition.substitute(variable, &item)?;
                            let expanded = substituted.expand(graph)?;
                            result = Expression::binary(BinaryOp::Or, result, expanded);
                        }
                        Ok(result)
                    }
                    Quantifier::ExistsUnique => {
                        let mut count = 0;
                        for item in items {
                            let substituted = condition.substitute(variable, &item)?;
                            let expanded = substituted.expand(graph)?;

                            if Self::is_true_literal(&expanded) {
                                count += 1;
                            }
                        }
                        Ok(Expression::literal(count == 1))
                    }
                }
            }
            Expression::GroupBy {
                variable,
                collection,
                filter,
                key,
                condition,
            } => {
                let items = Self::get_collection(collection, graph)?;

                // Filter items if filter is present
                let filtered_items = if let Some(filter_expr) = filter {
                    let mut filtered = Vec::new();
                    for item in items {
                        let substituted = filter_expr.substitute(variable, &item)?;
                        let expanded = substituted.expand(graph)?;
                        if Self::is_true_literal(&expanded) {
                            filtered.push(item);
                        }
                    }
                    filtered
                } else {
                    items
                };

                // Group items
                let mut groups: std::collections::HashMap<String, Vec<serde_json::Value>> =
                    std::collections::HashMap::new();
                for item in filtered_items {
                    let substituted_key = key.substitute(variable, &item)?;
                    let expanded_key = substituted_key.expand(graph)?;
                    let key_str = match expanded_key {
                        Expression::Literal(v) => match v {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => "null".to_string(),
                            other => {
                                return Err(format!(
                                "Group key must be a string, number, bool, or null literal, got {}",
                                other
                            ))
                            }
                        },
                        _ => {
                            return Err("Group key must evaluate to a literal".to_string());
                        }
                    };
                    groups.entry(key_str).or_default().push(item);
                }

                // Evaluate condition for each group
                for (_group_key, group_items) in groups {
                    // Substitute the variable with the group collection (as a Literal Array)
                    // This allows aggregations inside the condition to use the group items
                    let substituted_condition =
                        condition.substitute(variable, &serde_json::Value::Array(group_items))?;
                    let expanded = substituted_condition.expand(graph)?;
                    if !Self::is_true_literal(&expanded) {
                        return Ok(Expression::literal(false));
                    }
                }
                Ok(Expression::literal(true))
            }
            Expression::Binary { op, left, right } => {
                let left_expanded = left.expand(graph)?;
                let right_expanded = right.expand(graph)?;
                Ok(Self::reduce_binary_expression(
                    op,
                    left_expanded,
                    right_expanded,
                )?)
            }
            Expression::Unary { op, operand } => {
                Ok(Expression::unary(op.clone(), operand.expand(graph)?))
            }
            Expression::MemberAccess { .. } => Ok(self.clone()),
            Expression::Aggregation {
                function,
                collection,
                field,
                filter,
            } => {
                // Evaluate the aggregation and return a literal
                let result =
                    Self::evaluate_aggregation(function, collection, field, filter, graph)?;
                Ok(Expression::Literal(result))
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
                let result = Self::evaluate_aggregation_comprehension(
                    function,
                    variable,
                    collection,
                    window,
                    predicate,
                    projection,
                    target_unit.as_deref(),
                    graph,
                )?;
                Ok(Expression::Literal(result))
            }
            _ => Ok(self.clone()),
        }
    }

    pub fn substitute(&self, var: &str, value: &serde_json::Value) -> Result<Expression, String> {
        match self {
            Expression::Variable(n) => {
                if n == var {
                    Ok(Expression::Literal(value.clone()))
                } else if n.starts_with(&format!("{}.", var)) {
                    let field = &n[var.len() + 1..];
                    if let Some(field_value) = value.get(field) {
                        Ok(Expression::Literal(field_value.clone()))
                    } else {
                        // For three-valued semantics, if a field is missing in a substituted
                        // value, treat it as NULL/unknown rather than a fatal error. That way
                        // quantifiers and nested expressions can evaluate to NULL when data
                        // is optional.
                        Ok(Expression::Literal(serde_json::Value::Null))
                    }
                } else {
                    Ok(self.clone())
                }
            }
            Expression::Binary { op, left, right } => Ok(Expression::binary(
                op.clone(),
                left.substitute(var, value)?,
                right.substitute(var, value)?,
            )),
            Expression::Unary { op, operand } => Ok(Expression::unary(
                op.clone(),
                operand.substitute(var, value)?,
            )),
            Expression::Quantifier {
                quantifier,
                variable,
                collection,
                condition,
            } => {
                if var == variable {
                    // Don't substitute into condition when var matches the bound variable
                    // to avoid variable capture
                    Ok(Expression::quantifier(
                        quantifier.clone(),
                        variable,
                        collection.substitute(var, value)?,
                        *condition.clone(),
                    ))
                } else {
                    // Substitute into both collection and condition
                    Ok(Expression::quantifier(
                        quantifier.clone(),
                        variable,
                        collection.substitute(var, value)?,
                        condition.substitute(var, value)?,
                    ))
                }
            }
            Expression::MemberAccess { object, member } => {
                if object == var {
                    if let Some(field_value) = value.get(member) {
                        Ok(Expression::Literal(field_value.clone()))
                    } else {
                        Ok(Expression::Literal(serde_json::Value::Null))
                    }
                } else {
                    Ok(self.clone())
                }
            }
            Expression::Aggregation {
                function,
                collection,
                field,
                filter,
            } => Ok(Expression::aggregation(
                function.clone(),
                collection.substitute(var, value)?,
                field.clone(),
                filter
                    .as_ref()
                    .map(|f| f.substitute(var, value))
                    .transpose()?,
            )),
            Expression::AggregationComprehension {
                function,
                variable,
                collection,
                window,
                predicate,
                projection,
                target_unit,
            } => Ok(Expression::AggregationComprehension {
                function: function.clone(),
                variable: variable.clone(),
                collection: Box::new(collection.substitute(var, value)?),
                window: window.clone(),
                predicate: Box::new(predicate.substitute(var, value)?),
                projection: Box::new(projection.substitute(var, value)?),
                target_unit: target_unit.clone(),
            }),
            Expression::GroupBy {
                variable,
                collection,
                filter,
                key,
                condition,
            } => {
                // Similar to Quantifier, check if variable matches
                if var == variable {
                    Ok(Expression::GroupBy {
                        variable: variable.clone(),
                        collection: Box::new(collection.substitute(var, value)?),
                        filter: filter.clone(),
                        key: key.clone(),
                        condition: condition.clone(),
                    })
                } else {
                    Ok(Expression::GroupBy {
                        variable: variable.clone(),
                        collection: Box::new(collection.substitute(var, value)?),
                        filter: filter
                            .as_ref()
                            .map(|f| f.substitute(var, value))
                            .transpose()?
                            .map(Box::new),
                        key: Box::new(key.substitute(var, value)?),
                        condition: Box::new(condition.substitute(var, value)?),
                    })
                }
            }
            _ => Ok(self.clone()),
        }
    }

    pub(crate) fn get_collection(
        expr: &Expression,
        graph: &Graph,
    ) -> Result<Vec<serde_json::Value>, String> {
        match expr {
            Expression::Variable(name) => match name.as_str() {
                "flows" => {
                    let flows: Result<Vec<serde_json::Value>, String> = graph
                        .all_flows()
                        .iter()
                        .map(|f| {
                            let quantity = f.quantity().to_f64().ok_or_else(|| {
                                format!("Failed to convert flow quantity {} to f64", f.quantity())
                            })?;

                            let mut map = serde_json::Map::new();
                            map.insert("id".to_string(), serde_json::json!(f.id().to_string()));
                            map.insert(
                                "from_entity".to_string(),
                                serde_json::json!(f.from_id().to_string()),
                            );
                            map.insert(
                                "to_entity".to_string(),
                                serde_json::json!(f.to_id().to_string()),
                            );
                            map.insert(
                                "resource".to_string(),
                                serde_json::json!(f.resource_id().to_string()),
                            );
                            map.insert("quantity".to_string(), serde_json::json!(quantity));

                            for (k, v) in f.attributes().iter() {
                                if matches!(
                                    k.as_str(),
                                    "id" | "from_entity" | "to_entity" | "resource" | "quantity"
                                ) || map.contains_key(k)
                                {
                                    continue;
                                }
                                map.insert(k.clone(), v.clone());
                            }
                            Ok(serde_json::Value::Object(map))
                        })
                        .collect();
                    flows
                }
                "entities" => {
                    let entities: Vec<serde_json::Value> = graph
                        .all_entities()
                        .iter()
                        .map(|e| {
                            let mut map = serde_json::Map::new();
                            map.insert("id".to_string(), serde_json::json!(e.id().to_string()));
                            map.insert("name".to_string(), serde_json::json!(e.name()));
                            map.insert("namespace".to_string(), serde_json::json!(e.namespace()));

                            let roles = graph.role_names_for_entity(e.id());
                            if !roles.is_empty() {
                                map.insert("roles".to_string(), serde_json::json!(roles));
                            }

                            for (k, v) in e.attributes().iter() {
                                if matches!(k.as_str(), "id" | "name" | "namespace")
                                    || map.contains_key(k)
                                {
                                    continue;
                                }
                                map.insert(k.clone(), v.clone());
                            }

                            serde_json::Value::Object(map)
                        })
                        .collect();
                    Ok(entities)
                }
                "relations" => {
                    let relations: Vec<serde_json::Value> = graph
                        .all_relations()
                        .iter()
                        .map(|relation| {
                            let mut map = serde_json::Map::new();
                            map.insert(
                                "id".to_string(),
                                serde_json::json!(relation.id().to_string()),
                            );
                            map.insert("name".to_string(), serde_json::json!(relation.name()));
                            map.insert(
                                "predicate".to_string(),
                                serde_json::json!(relation.predicate()),
                            );

                            if let Some(subject) = graph.get_role(relation.subject_role()) {
                                map.insert(
                                    "subject_role".to_string(),
                                    serde_json::json!(subject.name()),
                                );
                            }

                            if let Some(object) = graph.get_role(relation.object_role()) {
                                map.insert(
                                    "object_role".to_string(),
                                    serde_json::json!(object.name()),
                                );
                            }

                            if let Some(flow) = relation.via_flow() {
                                map.insert("via".to_string(), serde_json::json!(flow.to_string()));
                            }

                            serde_json::Value::Object(map)
                        })
                        .collect();

                    Ok(relations)
                }
                "resources" => {
                    let resources: Vec<serde_json::Value> = graph
                        .all_resources()
                        .iter()
                        .map(|r| {
                            let mut map = serde_json::Map::new();
                            map.insert("id".to_string(), serde_json::json!(r.id().to_string()));
                            map.insert("name".to_string(), serde_json::json!(r.name()));
                            map.insert("namespace".to_string(), serde_json::json!(r.namespace()));
                            map.insert("unit".to_string(), serde_json::json!(r.unit()));
                            for (k, v) in r.attributes().iter() {
                                if matches!(k.as_str(), "id" | "name" | "namespace" | "unit")
                                    || map.contains_key(k)
                                {
                                    continue;
                                }
                                map.insert(k.clone(), v.clone());
                            }
                            serde_json::Value::Object(map)
                        })
                        .collect();
                    Ok(resources)
                }
                "instances" => {
                    let instances: Vec<serde_json::Value> = graph
                        .all_instances()
                        .iter()
                        .map(|i| {
                            let mut map = serde_json::Map::new();
                            map.insert("id".to_string(), serde_json::json!(i.id().to_string()));
                            map.insert(
                                "entity".to_string(),
                                serde_json::json!(i.entity_id().to_string()),
                            );
                            map.insert(
                                "resource".to_string(),
                                serde_json::json!(i.resource_id().to_string()),
                            );
                            for (k, v) in i.attributes().iter() {
                                if matches!(k.as_str(), "id" | "entity" | "resource")
                                    || map.contains_key(k)
                                {
                                    continue;
                                }
                                map.insert(k.clone(), v.clone());
                            }
                            serde_json::Value::Object(map)
                        })
                        .collect();
                    Ok(instances)
                }
                _ => Err(format!("Unknown collection: {}", name)),
            },
            Expression::Literal(serde_json::Value::Array(arr)) => Ok(arr.clone()),
            _ => Err("Collection expression must be a variable or array literal".to_string()),
        }
    }

    fn is_true_literal(expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(v) if v.as_bool() == Some(true))
    }

    pub(crate) fn evaluate_aggregation(
        function: &AggregateFunction,
        collection: &Expression,
        field: &Option<String>,
        filter: &Option<Box<Expression>>,
        graph: &Graph,
    ) -> Result<serde_json::Value, String> {
        // Get the collection items
        let items = Self::get_collection(collection, graph)?;

        // Apply filter if present
        let filtered_items = if let Some(filter_expr) = filter {
            // Determine the variable name based on collection type
            let variable_name = match collection {
                Expression::Variable(name) => match name.as_str() {
                    "flows" => "flow",
                    "entities" => "entity",
                    "resources" => "resource",
                    "instances" => "instance",
                    "relations" => "relation",
                    _ => "item",
                },
                _ => "item",
            };

            items
                .into_iter()
                .filter(|item| {
                    // Substitute collection-specific variables in the filter
                    let substituted = filter_expr
                        .substitute(variable_name, item)
                        .unwrap_or_else(|_| filter_expr.as_ref().clone());
                    // Expand and check if true
                    if let Ok(expanded) = substituted.expand(graph) {
                        Self::is_true_literal(&expanded)
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>()
        } else {
            items
        };

        // Apply aggregation function
        match function {
            AggregateFunction::Count => Ok(serde_json::json!(filtered_items.len())),

            AggregateFunction::Sum => {
                let field_name = field.as_ref().ok_or("Sum requires a field specification")?;

                let sum: Decimal = filtered_items
                    .iter()
                    .filter_map(|item| {
                        item.get(field_name).and_then(|v| {
                            if let Some(num) = v.as_f64() {
                                Decimal::from_str(&num.to_string()).ok()
                            } else if let Some(s) = v.as_str() {
                                Decimal::from_str(s).ok()
                            } else {
                                None
                            }
                        })
                    })
                    .sum();

                // Convert Decimal to f64 and propagate parsing errors
                let sum_f64 = sum
                    .to_f64()
                    .ok_or_else(|| format!("Failed to convert sum {} to f64", sum))?;

                Ok(serde_json::json!(sum_f64))
            }

            AggregateFunction::Avg => {
                let field_name = field.as_ref().ok_or("Avg requires a field specification")?;

                let values: Vec<Decimal> = filtered_items
                    .iter()
                    .filter_map(|item| {
                        item.get(field_name).and_then(|v| {
                            if let Some(num) = v.as_f64() {
                                Decimal::from_str(&num.to_string()).ok()
                            } else if let Some(s) = v.as_str() {
                                Decimal::from_str(s).ok()
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                if values.is_empty() {
                    return Ok(serde_json::json!(null));
                }

                let sum: Decimal = values.iter().copied().sum();
                let avg = sum / Decimal::from(values.len());

                // Convert Decimal to f64 and propagate parsing errors
                let avg_f64 = avg
                    .to_f64()
                    .ok_or_else(|| format!("Failed to convert average {} to f64", avg))?;

                Ok(serde_json::json!(avg_f64))
            }

            AggregateFunction::Min => {
                let field_name = field.as_ref().ok_or("Min requires a field specification")?;

                let min = filtered_items
                    .iter()
                    .filter_map(|item| {
                        item.get(field_name).and_then(|v| {
                            if let Some(num) = v.as_f64() {
                                Decimal::from_str(&num.to_string()).ok()
                            } else if let Some(s) = v.as_str() {
                                Decimal::from_str(s).ok()
                            } else {
                                None
                            }
                        })
                    })
                    .min();

                // Convert Decimal to f64 and propagate parsing errors
                if let Some(min_val) = min {
                    let min_f64 = min_val
                        .to_f64()
                        .ok_or_else(|| format!("Failed to convert min {} to f64", min_val))?;
                    Ok(serde_json::json!(min_f64))
                } else {
                    Ok(serde_json::json!(null))
                }
            }

            AggregateFunction::Max => {
                let field_name = field.as_ref().ok_or("Max requires a field specification")?;

                let max = filtered_items
                    .iter()
                    .filter_map(|item| {
                        item.get(field_name).and_then(|v| {
                            if let Some(num) = v.as_f64() {
                                Decimal::from_str(&num.to_string()).ok()
                            } else if let Some(s) = v.as_str() {
                                Decimal::from_str(s).ok()
                            } else {
                                None
                            }
                        })
                    })
                    .max();

                // Convert Decimal to f64 and propagate parsing errors
                if let Some(max_val) = max {
                    let max_f64 = max_val
                        .to_f64()
                        .ok_or_else(|| format!("Failed to convert max {} to f64", max_val))?;
                    Ok(serde_json::json!(max_f64))
                } else {
                    Ok(serde_json::json!(null))
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn evaluate_aggregation_comprehension(
        function: &AggregateFunction,
        variable: &str,
        collection: &Expression,
        window: &Option<crate::policy::WindowSpec>,
        predicate: &Expression,
        projection: &Expression,
        target_unit: Option<&str>,
        graph: &Graph,
    ) -> Result<serde_json::Value, String> {
        let items = Self::get_collection(collection, graph)?;

        // Apply window filtering if present
        let items = if let Some(w) = window {
            let now = chrono::Utc::now();
            let duration_span = i64::try_from(w.duration)
                .map_err(|_| format!("Window duration {} exceeds supported range", w.duration))?;
            let unit_lower = w.unit.to_lowercase();
            let duration = match unit_lower.as_str() {
                "hour" | "hours" => chrono::Duration::hours(duration_span),
                "minute" | "minutes" => chrono::Duration::minutes(duration_span),
                "day" | "days" => chrono::Duration::days(duration_span),
                "second" | "seconds" => chrono::Duration::seconds(duration_span),
                _ => {
                    return Err(format!(
                        "Invalid window unit '{}' in aggregation window",
                        w.unit
                    ))
                }
            };

            items
                .into_iter()
                .filter(|item| {
                    let ts_str = item
                        .get("timestamp")
                        .or_else(|| item.get("created_at"))
                        .and_then(|v| v.as_str());
                    if let Some(s) = ts_str {
                        if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(s) {
                            let ts_utc: chrono::DateTime<chrono::Utc> = ts.into();
                            return ts_utc >= now - duration;
                        }
                    }
                    // If no timestamp, we can't filter, so maybe exclude? Or include?
                    // Safer to exclude if windowing is requested but data is missing.
                    false
                })
                .collect()
        } else {
            items
        };

        let mut projected_values = Vec::new();
        for item in items {
            let substituted_predicate = predicate.substitute(variable, &item)?;
            let predicate_result = substituted_predicate.expand(graph)?;
            if !Self::is_true_literal(&predicate_result) {
                continue;
            }

            let substituted_projection = projection.substitute(variable, &item)?;
            let projection_result = substituted_projection.expand(graph)?;
            match projection_result {
                Expression::Literal(value) => projected_values.push(value),
                Expression::QuantityLiteral { value, unit } => {
                    projected_values.push(serde_json::json!({
                        "__quantity_value": value.to_string(),
                        "__quantity_unit": unit,
                    }));
                }
                _ => {
                    return Err(
                        "Projection in aggregation comprehension must reduce to a literal"
                            .to_string(),
                    );
                }
            }
        }

        match function {
            AggregateFunction::Count => Ok(serde_json::json!(projected_values.len())),
            AggregateFunction::Sum
            | AggregateFunction::Avg
            | AggregateFunction::Min
            | AggregateFunction::Max => {
                Self::fold_numeric(function, &projected_values, target_unit)
            }
        }
    }

    pub(crate) fn fold_numeric(
        function: &AggregateFunction,
        values: &[serde_json::Value],
        target_unit: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        use crate::units::UnitRegistry;

        let mut decimals: Vec<Decimal> = Vec::new();
        let mut source_unit: Option<String> = None;
        for value in values {
            if let Some(num) = value.as_f64() {
                decimals.push(Decimal::from_str(&num.to_string()).map_err(|e| e.to_string())?);
            } else if let Some(s) = value.as_str() {
                decimals.push(Decimal::from_str(s).map_err(|e| e.to_string())?);
            } else if value.is_object() {
                let map = value
                    .as_object()
                    .ok_or_else(|| "Invalid quantity object".to_string())?;
                if let Some(val) = map.get("__quantity_value") {
                    let s = val
                        .as_str()
                        .ok_or_else(|| "Quantity value must be a string".to_string())?;
                    decimals.push(Decimal::from_str(s).map_err(|e| e.to_string())?);
                }
                if let Some(unit) = map.get("__quantity_unit") {
                    if source_unit.is_none() {
                        if let Some(unit_str) = unit.as_str() {
                            source_unit = Some(unit_str.to_string());
                        }
                    }
                }
            }
        }

        if decimals.is_empty() {
            return Ok(serde_json::json!(null));
        }

        if let Some(target_unit) = target_unit {
            if let Some(unit) = source_unit.clone() {
                let registry = UnitRegistry::global();
                let registry = registry
                    .read()
                    .map_err(|e| format!("Failed to lock unit registry: {}", e))?;
                let from = registry
                    .get_unit(&unit)
                    .map_err(|e| format!("{}", e))?
                    .clone();
                let to = registry
                    .get_unit(target_unit)
                    .map_err(|e| format!("{}", e))?
                    .clone();
                decimals = decimals
                    .into_iter()
                    .map(|value| registry.convert(value, &from, &to))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("{}", e))?;
            }
        }

        let result = match function {
            AggregateFunction::Sum => decimals.iter().copied().sum(),
            AggregateFunction::Avg => {
                let sum: Decimal = decimals.iter().copied().sum();
                sum / Decimal::from(decimals.len())
            }
            AggregateFunction::Min => decimals
                .into_iter()
                .min()
                .ok_or_else(|| "No values available for min".to_string())?,
            AggregateFunction::Max => decimals
                .into_iter()
                .max()
                .ok_or_else(|| "No values available for max".to_string())?,
            AggregateFunction::Count => Decimal::from(values.len() as i64),
        };

        let as_f64 = result
            .to_f64()
            .ok_or_else(|| format!("Failed to convert aggregated value {} to f64", result))?;

        Ok(serde_json::json!(as_f64))
    }

    fn reduce_binary_expression(
        op: &BinaryOp,
        left: Expression,
        right: Expression,
    ) -> Result<Expression, String> {
        match (&left, &right) {
            (Expression::Literal(left_value), Expression::Literal(right_value)) => {
                // Preserve tri-state semantics: if either operand is NULL, yield NULL.
                if left_value.is_null() || right_value.is_null() {
                    return Ok(Expression::Literal(serde_json::Value::Null));
                }

                let numeric_cmp = left_value.is_number() && right_value.is_number();
                let reduced = match op {
                    BinaryOp::Equal => {
                        if numeric_cmp {
                            let left_num = Self::value_to_f64(left_value)
                                .ok_or_else(|| "Left operand is not numeric".to_string())?;
                            let right_num = Self::value_to_f64(right_value)
                                .ok_or_else(|| "Right operand is not numeric".to_string())?;
                            serde_json::json!(left_num == right_num)
                        } else {
                            serde_json::json!(left_value == right_value)
                        }
                    }
                    BinaryOp::NotEqual => {
                        if numeric_cmp {
                            let left_num = Self::value_to_f64(left_value)
                                .ok_or_else(|| "Left operand is not numeric".to_string())?;
                            let right_num = Self::value_to_f64(right_value)
                                .ok_or_else(|| "Right operand is not numeric".to_string())?;
                            serde_json::json!(left_num != right_num)
                        } else {
                            serde_json::json!(left_value != right_value)
                        }
                    }
                    BinaryOp::GreaterThan
                    | BinaryOp::LessThan
                    | BinaryOp::GreaterThanOrEqual
                    | BinaryOp::LessThanOrEqual => {
                        let left_num = Self::value_to_f64(left_value)
                            .ok_or_else(|| "Left operand is not numeric".to_string())?;
                        let right_num = Self::value_to_f64(right_value)
                            .ok_or_else(|| "Right operand is not numeric".to_string())?;
                        match op {
                            BinaryOp::GreaterThan => serde_json::json!(left_num > right_num),
                            BinaryOp::LessThan => serde_json::json!(left_num < right_num),
                            BinaryOp::GreaterThanOrEqual => {
                                serde_json::json!(left_num >= right_num)
                            }
                            BinaryOp::LessThanOrEqual => serde_json::json!(left_num <= right_num),
                            _ => unreachable!(),
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        let left_bool = match left_value {
                            serde_json::Value::Bool(b) => Some(*b),
                            serde_json::Value::Null => None,
                            _ => return Err("Left operand is not boolean".to_string()),
                        };
                        let right_bool = match right_value {
                            serde_json::Value::Bool(b) => Some(*b),
                            serde_json::Value::Null => None,
                            _ => return Err("Right operand is not boolean".to_string()),
                        };

                        match op {
                            BinaryOp::And => {
                                if left_bool == Some(false) || right_bool == Some(false) {
                                    serde_json::json!(false)
                                } else if left_bool.is_none() || right_bool.is_none() {
                                    serde_json::Value::Null
                                } else {
                                    serde_json::json!(left_bool.unwrap() && right_bool.unwrap())
                                }
                            }
                            BinaryOp::Or => {
                                if left_bool == Some(true) || right_bool == Some(true) {
                                    serde_json::json!(true)
                                } else if left_bool.is_none() || right_bool.is_none() {
                                    serde_json::Value::Null
                                } else {
                                    serde_json::json!(left_bool.unwrap() || right_bool.unwrap())
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    BinaryOp::Contains | BinaryOp::StartsWith | BinaryOp::EndsWith => {
                        let left_str = left_value
                            .as_str()
                            .ok_or_else(|| "Left operand is not string".to_string())?;
                        let right_str = right_value
                            .as_str()
                            .ok_or_else(|| "Right operand is not string".to_string())?;
                        match op {
                            BinaryOp::Contains => serde_json::json!(left_str.contains(right_str)),
                            BinaryOp::StartsWith => {
                                serde_json::json!(left_str.starts_with(right_str))
                            }
                            BinaryOp::EndsWith => serde_json::json!(left_str.ends_with(right_str)),
                            _ => unreachable!(),
                        }
                    }
                    _ => return Ok(Expression::binary(op.clone(), left.clone(), right.clone())),
                };
                Ok(Expression::Literal(reduced))
            }
            _ => Ok(Expression::binary(op.clone(), left, right)),
        }
    }

    fn value_to_f64(value: &serde_json::Value) -> Option<f64> {
        value
            .as_f64()
            .or_else(|| value.as_i64().map(|v| v as f64))
            .or_else(|| value.as_u64().map(|v| v as f64))
    }
}
