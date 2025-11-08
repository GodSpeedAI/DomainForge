use crate::graph::Graph;
use super::expression::{Expression, BinaryOp, Quantifier};

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
            Expression::Binary { op, left, right } => {
                Ok(Expression::binary(
                    op.clone(),
                    left.expand(graph)?,
                    right.expand(graph)?,
                ))
            }
            Expression::Unary { op, operand } => {
                Ok(Expression::unary(op.clone(), operand.expand(graph)?))
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
            Expression::Binary { op, left, right } => {
                Ok(Expression::binary(
                    op.clone(),
                    left.substitute(var, value)?,
                    right.substitute(var, value)?,
                ))
            }
            Expression::Unary { op, operand } => {
                Ok(Expression::unary(
                    op.clone(),
                    operand.substitute(var, value)?,
                ))
            }
            Expression::Quantifier { quantifier, variable, collection, condition } => {
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
            _ => Ok(self.clone()),
        }
    }

    fn get_collection(expr: &Expression, graph: &Graph) -> Result<Vec<serde_json::Value>, String> {
        match expr {
            Expression::Variable(name) => {
                match name.as_str() {
                    "flows" => {
                        let flows: Vec<serde_json::Value> = graph.all_flows()
                            .iter()
                            .map(|f| serde_json::json!({
                                "id": f.id().to_string(), // Keep as string for JSON compatibility
                                "from_entity": f.from_id().to_string(),
                                "to_entity": f.to_id().to_string(),
                                "resource": f.resource_id().to_string(),
                                "quantity": f.quantity().to_string().parse::<f64>().unwrap_or(0.0),
                            }))
                            .collect();
                        Ok(flows)
                    }
                    "entities" => {
                        let entities: Vec<serde_json::Value> = graph.all_entities()
                            .iter()
                            .map(|e| serde_json::json!({
                                "id": e.id().to_string(),
                                "name": e.name(),
                                "namespace": e.namespace(),
                            }))
                            .collect();
                        Ok(entities)
                    }
                    "resources" => {
                        let resources: Vec<serde_json::Value> = graph.all_resources()
                            .iter()
                            .map(|r| serde_json::json!({
                                "id": r.id().to_string(),
                                "name": r.name(),
                                "namespace": r.namespace(),
                                "unit": r.unit(),
                            }))
                            .collect();
                        Ok(resources)
                    }
                    "instances" => {
                        let instances: Vec<serde_json::Value> = graph.all_instances()
                            .iter()
                            .map(|i| serde_json::json!({
                                "id": i.id().to_string(),
                                "entity": i.entity_id().to_string(),
                                "resource": i.resource_id().to_string(),
                            }))
                            .collect();
                        Ok(instances)
                    }
                    _ => Err(format!("Unknown collection: {}", name))
                }
            }
            _ => Err("Collection expression must be a variable".to_string())
        }
    }

    fn is_true_literal(expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(v) if v.as_bool() == Some(true))
    }
}
