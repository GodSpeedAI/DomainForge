use super::expression::{AggregateFunction, BinaryOp, Expression, Quantifier};
use crate::graph::Graph;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
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
            Expression::Binary { op, left, right } => Ok(Expression::binary(
                op.clone(),
                left.expand(graph)?,
                right.expand(graph)?,
            )),
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
                        Err(format!("Field '{}' not found in value", field))
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
            Expression::MemberAccess { .. } => Ok(self.clone()),
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
            _ => Ok(self.clone()),
        }
    }

    fn get_collection(expr: &Expression, graph: &Graph) -> Result<Vec<serde_json::Value>, String> {
        match expr {
            Expression::Variable(name) => {
                match name.as_str() {
                    "flows" => {
                        let flows: Result<Vec<serde_json::Value>, String> = graph
                            .all_flows()
                            .iter()
                            .map(|f| {
                                let quantity = f.quantity().to_f64()
                                    .ok_or_else(|| format!("Failed to convert flow quantity {} to f64", f.quantity()))?;
                                Ok(serde_json::json!({
                                    "id": f.id().to_string(), // Keep as string for JSON compatibility
                                    "from_entity": f.from_id().to_string(),
                                    "to_entity": f.to_id().to_string(),
                                    "resource": f.resource_id().to_string(),
                                    "quantity": quantity,
                                }))
                            })
                            .collect();
                        flows
                    }
                    "entities" => {
                        let entities: Vec<serde_json::Value> = graph
                            .all_entities()
                            .iter()
                            .map(|e| {
                                serde_json::json!({
                                    "id": e.id().to_string(),
                                    "name": e.name(),
                                    "namespace": e.namespace(),
                                })
                            })
                            .collect();
                        Ok(entities)
                    }
                    "resources" => {
                        let resources: Vec<serde_json::Value> = graph
                            .all_resources()
                            .iter()
                            .map(|r| {
                                serde_json::json!({
                                    "id": r.id().to_string(),
                                    "name": r.name(),
                                    "namespace": r.namespace(),
                                    "unit": r.unit(),
                                })
                            })
                            .collect();
                        Ok(resources)
                    }
                    "instances" => {
                        let instances: Vec<serde_json::Value> = graph
                            .all_instances()
                            .iter()
                            .map(|i| {
                                serde_json::json!({
                                    "id": i.id().to_string(),
                                    "entity": i.entity_id().to_string(),
                                    "resource": i.resource_id().to_string(),
                                })
                            })
                            .collect();
                        Ok(instances)
                    }
                    _ => Err(format!("Unknown collection: {}", name)),
                }
            }
            _ => Err("Collection expression must be a variable".to_string()),
        }
    }

    fn is_true_literal(expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(v) if v.as_bool() == Some(true))
    }

    fn evaluate_aggregation(
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
}
