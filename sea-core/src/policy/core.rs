use super::expression::{BinaryOp, Expression, UnaryOp};
use super::violation::{Severity, Violation};
use crate::policy::ThreeValuedBool;
use crate::graph::Graph;
use crate::{ConceptId, SemanticVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyModality {
    Obligation,
    Prohibition,
    Permission,
}

/// Backwards-compatible alias used by existing tests and documentation
#[allow(unused_imports)]
pub use PolicyModality as DeonticModality;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyKind {
    Constraint,
    Derivation,
    Obligation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: ConceptId,
    pub name: String,
    pub namespace: String,
    pub version: SemanticVersion,
    pub expression: Expression,
    pub modality: PolicyModality,
    pub kind: PolicyKind,
    pub priority: i32,
    pub rationale: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Backwards compatible boolean representing whether a policy was satisfied.
    /// Defaults to `false` if the evaluator produced an unknown (NULL) result.
    pub is_satisfied: bool,
    /// Tri-state evaluation result: Some(true/false) or None when evaluation is unknown.
    pub is_satisfied_tristate: Option<bool>,
    pub violations: Vec<Violation>,
}

impl Policy {
    pub fn new(name: impl Into<String>, expression: Expression) -> Self {
        let name = name.into();
        Self {
            id: ConceptId::from_concept("default", &name),
            name,
            namespace: "default".to_string(),
            version: SemanticVersion::default(),
            expression,
            modality: PolicyModality::Obligation,
            kind: PolicyKind::Constraint,
            priority: 0,
            rationale: None,
            tags: Vec::new(),
        }
    }

    pub fn new_with_namespace(
        name: impl Into<String>,
        namespace: impl Into<String>,
        expression: Expression,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let id = ConceptId::from_concept(&namespace, &name);

        Self {
            id,
            name,
            namespace,
            version: SemanticVersion::default(),
            expression,
            modality: PolicyModality::Obligation,
            kind: PolicyKind::Constraint,
            priority: 0,
            rationale: None,
            tags: Vec::new(),
        }
    }

    pub fn with_modality(mut self, modality: PolicyModality) -> Self {
        self.modality = modality;
        self
    }

    pub fn with_version(mut self, version: SemanticVersion) -> Self {
        self.version = version;
        self
    }

    pub fn with_kind(mut self, kind: PolicyKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_metadata(
        mut self,
        kind: Option<PolicyKind>,
        modality: Option<PolicyModality>,
        priority: Option<i32>,
        rationale: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        if let Some(kind) = kind {
            self.kind = kind;
        }
        if let Some(modality) = modality {
            self.modality = modality;
        }
        if let Some(priority) = priority {
            self.priority = priority;
        }
        self.rationale = rationale;
        if !tags.is_empty() {
            self.tags = tags;
        }
        self
    }

    pub fn kind(&self) -> &PolicyKind {
        &self.kind
    }

    pub fn evaluate(&self, graph: &Graph) -> Result<EvaluationResult, String> {
        self.evaluate_with_mode(graph, graph.use_three_valued_logic())
    }

    pub fn evaluate_with_mode(
        &self,
        graph: &Graph,
        use_three_valued_logic: bool,
    ) -> Result<EvaluationResult, String> {
        Self::validate_aggregation_usage(&self.expression, true)?;

        let expanded = self.expression.expand(graph)?;

        // Evaluate expression; runtime toggle chooses three-valued vs boolean path.
        // We compute the tri-state result and derive a backward-compatible boolean (false when Null).
        let is_satisfied_tristate: Option<bool> = if use_three_valued_logic {
            match self.evaluate_expression_three_valued(&expanded, graph)? {
                ThreeValuedBool::True => Some(true),
                ThreeValuedBool::False => Some(false),
                ThreeValuedBool::Null => None,
            }
        } else {
            Some(self.evaluate_expression_boolean(&expanded, graph)?)
        };

        let is_satisfied = is_satisfied_tristate.unwrap_or(false);

        let violations = if is_satisfied_tristate == Some(true) {
            vec![]
        } else if is_satisfied_tristate == Some(false) {
            vec![Violation::new(
                &self.name,
                format!("Policy '{}' was violated", self.name),
                self.modality.to_severity(),
            )]
        } else {
            // Unknown (NULL) evaluation: severity follows the policy modality.
            vec![Violation::new(
                &self.name,
                format!("Policy '{}' evaluation is UNKNOWN (NULL)", self.name),
                self.modality.to_severity(),
            )]
        };

        Ok(EvaluationResult {
            is_satisfied,
            is_satisfied_tristate,
            violations,
        })
    }

    fn validate_aggregation_usage(expr: &Expression, in_boolean_context: bool) -> Result<(), String> {
        match expr {
            Expression::Aggregation { .. } | Expression::AggregationComprehension { .. } => {
                if in_boolean_context {
                    Err("Aggregation in boolean context requires explicit comparison (e.g., COUNT(...) > 0)".to_string())
                } else {
                    Ok(())
                }
            }
            Expression::Binary { op, left, right } => {
                let child_boolean = matches!(op, BinaryOp::And | BinaryOp::Or);
                Self::validate_aggregation_usage(left, child_boolean)?;
                Self::validate_aggregation_usage(right, child_boolean)?;
                Ok(())
            }
            Expression::Unary { op, operand } => {
                let child_boolean = matches!(op, UnaryOp::Not);
                Self::validate_aggregation_usage(operand, child_boolean)
            }
            Expression::Quantifier { collection, condition, .. } => {
                Self::validate_aggregation_usage(collection, false)?;
                Self::validate_aggregation_usage(condition, true)
            }
            _ => Ok(()),
        }
    }

    fn evaluate_expression_boolean(&self, expr: &Expression, graph: &Graph) -> Result<bool, String> {
        match expr {
            Expression::Literal(v) => v
                .as_bool()
                .ok_or_else(|| format!("Expected boolean literal, got: {}", v)),
            Expression::Variable(name) => {
                Err(format!("Cannot evaluate unexpanded variable: {}", name))
            }
            Expression::Binary { op, left, right } => match op {
                BinaryOp::And | BinaryOp::Or => {
                    let left_val = self.evaluate_expression_boolean(left, graph)?;
                    let right_val = self.evaluate_expression_boolean(right, graph)?;

                    Ok(match op {
                        BinaryOp::And => left_val && right_val,
                        BinaryOp::Or => left_val || right_val,
                        _ => unreachable!(),
                    })
                }
                BinaryOp::Equal | BinaryOp::NotEqual => {
                    self.compare_values(left, right, graph, |l, r| match op {
                        BinaryOp::Equal => l == r,
                        BinaryOp::NotEqual => l != r,
                        _ => unreachable!(),
                    })
                }
                BinaryOp::GreaterThan
                | BinaryOp::LessThan
                | BinaryOp::GreaterThanOrEqual
                | BinaryOp::LessThanOrEqual => {
                    self.compare_numeric(left, right, graph, |l, r| match op {
                        BinaryOp::GreaterThan => l > r,
                        BinaryOp::LessThan => l < r,
                        BinaryOp::GreaterThanOrEqual => l >= r,
                        BinaryOp::LessThanOrEqual => l <= r,
                        _ => unreachable!(),
                    })
                }
                BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Multiply | BinaryOp::Divide => {
                    Err("Arithmetic operations not supported in boolean context".to_string())
                }
                BinaryOp::Contains | BinaryOp::StartsWith | BinaryOp::EndsWith => self
                    .compare_strings(left, right, graph, |l, r| match op {
                        BinaryOp::Contains => l.contains(r),
                        BinaryOp::StartsWith => l.starts_with(r),
                        BinaryOp::EndsWith => l.ends_with(r),
                        _ => unreachable!(),
                    }),
            },
            Expression::Unary { op, operand } => {
                let val = self.evaluate_expression_boolean(operand, graph)?;
                Ok(match op {
                    UnaryOp::Not => !val,
                    UnaryOp::Negate => {
                        return Err("Negate operator not supported in boolean context".to_string())
                    }
                })
            }
            Expression::Quantifier { .. } => {
                Err("Cannot evaluate non-expanded quantifier".to_string())
            }
            Expression::MemberAccess { object, member } => {
                let value = self.get_runtime_value(expr, graph)?;
                match value {
                    serde_json::Value::Bool(v) => Ok(v),
                    serde_json::Value::Null => Ok(false),
                    _ => Err(format!(
                        "Expected boolean value for member '{}.{}', but found {:?}",
                        object, member, value
                    )),
                }
            }
            Expression::Aggregation { .. } => {
                Err("Cannot evaluate non-expanded aggregation".to_string())
            }
            Expression::AggregationComprehension { .. } => {
                Err("Cannot evaluate non-expanded aggregation comprehension".to_string())
            }
            Expression::QuantityLiteral { .. } => {
                Err("Cannot evaluate quantity literal in boolean context".to_string())
            }
        }
    }

    fn evaluate_expression_three_valued(
        &self,
        expr: &Expression,
        graph: &Graph,
    ) -> Result<ThreeValuedBool, String> {
        use ThreeValuedBool as T;

        match expr {
            Expression::Literal(v) => Ok(T::from_option_bool(v.as_bool())),
            Expression::Variable(name) => Err(format!("Cannot evaluate unexpanded variable: {}", name)),
            Expression::Binary { op, left, right } => match op {
                BinaryOp::And | BinaryOp::Or => {
                    let l = self.evaluate_expression_three_valued(left, graph)?;
                    let r = self.evaluate_expression_three_valued(right, graph)?;
                    Ok(match op {
                        BinaryOp::And => l.and(r),
                        BinaryOp::Or => l.or(r),
                        _ => unreachable!(),
                    })
                }
                BinaryOp::Equal | BinaryOp::NotEqual => {
                    let left_val = self.get_runtime_value(left, graph);
                    let right_val = self.get_runtime_value(right, graph);
                    match (left_val, right_val) {
                        (Ok(lv), Ok(rv)) => {
                            // If either operand is JSON Null, the comparison yields Null.
                            if lv.is_null() || rv.is_null() {
                                Ok(T::Null)
                            } else {
                                let eq = match op {
                                    BinaryOp::Equal => lv == rv,
                                    BinaryOp::NotEqual => lv != rv,
                                    _ => unreachable!(),
                                };
                                Ok(T::from_option_bool(Some(eq)))
                            }
                        }
                        _ => Ok(T::Null),
                    }
                }
                BinaryOp::GreaterThan
                | BinaryOp::LessThan
                | BinaryOp::GreaterThanOrEqual
                | BinaryOp::LessThanOrEqual => {
                    // Use runtime retrieval which may yield serialization Null.
                    let left_v = self.get_runtime_value(left, graph);
                    let right_v = self.get_runtime_value(right, graph);
                    match (left_v, right_v) {
                        (Ok(lv), Ok(rv)) => {
                            if lv.is_null() || rv.is_null() {
                                Ok(T::Null)
                            } else if let (Some(l), Some(r)) = (lv.as_f64(), rv.as_f64()) {
                                let res = match op {
                                    BinaryOp::GreaterThan => l > r,
                                    BinaryOp::LessThan => l < r,
                                    BinaryOp::GreaterThanOrEqual => l >= r,
                                    BinaryOp::LessThanOrEqual => l <= r,
                                    _ => unreachable!(),
                                };
                                Ok(T::from_option_bool(Some(res)))
                            } else {
                                Ok(T::Null)
                            }
                        }
                        _ => Ok(T::Null),
                    }
                }
                BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Multiply | BinaryOp::Divide => {
                    Err("Arithmetic operations not supported in boolean context".to_string())
                }
                BinaryOp::Contains | BinaryOp::StartsWith | BinaryOp::EndsWith => {
                    let left_v = self.get_runtime_value(left, graph);
                    let right_v = self.get_runtime_value(right, graph);
                    match (left_v, right_v) {
                        (Ok(lv), Ok(rv)) => {
                            if lv.is_null() || rv.is_null() {
                                Ok(T::Null)
                            } else if let (Some(ls), Some(rs)) = (lv.as_str(), rv.as_str()) {
                                let ok = match op {
                                    BinaryOp::Contains => ls.contains(rs),
                                    BinaryOp::StartsWith => ls.starts_with(rs),
                                    BinaryOp::EndsWith => ls.ends_with(rs),
                                    _ => unreachable!(),
                                };
                                Ok(T::from_option_bool(Some(ok)))
                            } else {
                                Ok(T::Null)
                            }
                        }
                        _ => Ok(T::Null),
                    }
                }
            },
            Expression::Unary { op, operand } => {
                let v = self.evaluate_expression_three_valued(operand, graph)?;
                Ok(match op {
                    UnaryOp::Not => v.not(),
                    UnaryOp::Negate => return Err("Negate operator not supported in boolean context".to_string()),
                })
            }
            Expression::Quantifier { quantifier, variable, collection, condition } => {
                // Evaluate quantifiers with three-valued semantics
                let items = Expression::get_collection(collection, graph)?;
                use super::expression::Quantifier as Q;

                let mut saw_true = 0usize;
                let mut saw_false = 0usize;
                let mut saw_null = 0usize;

                for item in items {
                    let substituted = condition.substitute(variable, &item)?;
                    let val = self.evaluate_expression_three_valued(&substituted, graph)?;
                    match val {
                        T::True => saw_true += 1,
                        T::False => saw_false += 1,
                        T::Null => saw_null += 1,
                    }
                }

                match quantifier {
                    Q::ForAll => {
                        if saw_false > 0 { return Ok(T::False); }
                        if saw_null > 0 { return Ok(T::Null); }
                        Ok(T::True)
                    }
                    Q::Exists => {
                        if saw_true > 0 { return Ok(T::True); }
                        if saw_null > 0 { return Ok(T::Null); }
                        Ok(T::False)
                    }
                    Q::ExistsUnique => {
                        if saw_true > 1 { return Ok(T::False); }
                        if saw_true == 1 && saw_null == 0 { return Ok(T::True); }
                        if saw_true == 1 && saw_null > 0 { return Ok(T::Null); }
                        if saw_true == 0 && saw_null > 0 { return Ok(T::Null); }
                        Ok(T::False)
                    }
                }
            }
            Expression::MemberAccess { object: _, member: _ } => {
                // Resolve object/member to a runtime value and convert to bool
                let value = self.get_runtime_value(expr, graph)?;
                Ok(T::from_option_bool(value.as_bool()))
            }
            Expression::Aggregation { .. } => Err("Aggregation in boolean context requires explicit comparison (e.g., COUNT(...) > 0)".to_string()),
            Expression::AggregationComprehension { .. } => Err("Aggregation in boolean context requires explicit comparison (e.g., COUNT(...) > 0)".to_string()),
            Expression::QuantityLiteral { .. } => Err("Cannot convert quantity to boolean; compare against a threshold instead".to_string()),
        }
    }

    fn compare_values<F>(
        &self,
        left: &Expression,
        right: &Expression,
        graph: &Graph,
        op: F,
    ) -> Result<bool, String>
    where
        F: Fn(&serde_json::Value, &serde_json::Value) -> bool,
    {
        let left_val = self
            .get_literal_value(left)
            .or_else(|_| self.get_runtime_value(left, graph))?;
        let right_val = self
            .get_literal_value(right)
            .or_else(|_| self.get_runtime_value(right, graph))?;
        Ok(op(&left_val, &right_val))
    }

    fn compare_numeric<F>(
        &self,
        left: &Expression,
        right: &Expression,
        graph: &Graph,
        op: F,
    ) -> Result<bool, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_val = self.get_numeric_value(left, graph)?;
        let right_val = self.get_numeric_value(right, graph)?;
        Ok(op(left_val, right_val))
    }

    fn compare_strings<F>(
        &self,
        left: &Expression,
        right: &Expression,
        graph: &Graph,
        op: F,
    ) -> Result<bool, String>
    where
        F: Fn(&str, &str) -> bool,
    {
        let left_val = self.get_string_value(left, graph)?;
        let right_val = self.get_string_value(right, graph)?;
        Ok(op(&left_val, &right_val))
    }

    fn get_literal_value(&self, expr: &Expression) -> Result<serde_json::Value, String> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            _ => Err("Expected literal value".to_string()),
        }
    }

    fn get_numeric_value(&self, expr: &Expression, graph: &Graph) -> Result<f64, String> {
        let v = self
            .get_literal_value(expr)
            .or_else(|_| self.get_runtime_value(expr, graph))?;
        v.as_f64()
            .or_else(|| v.as_i64().map(|i| i as f64))
            .ok_or_else(|| "Expected numeric value".to_string())
    }

    fn get_string_value(&self, expr: &Expression, graph: &Graph) -> Result<String, String> {
        let v = self
            .get_literal_value(expr)
            .or_else(|_| self.get_runtime_value(expr, graph))?;
        v.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Expected string value".to_string())
    }

    fn get_runtime_value(&self, expr: &Expression, graph: &Graph) -> Result<serde_json::Value, String> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            Expression::MemberAccess { object, member } => {
                // Try resolving as entity
                if let Some(id) = graph.find_entity_by_name(object) {
                    if let Some(entity) = graph.get_entity(&id) {
                        // Special known members
                        if member == "id" {
                            return Ok(serde_json::json!(entity.id().to_string()));
                        } else if member == "name" {
                            return Ok(serde_json::json!(entity.name()));
                        } else if member == "namespace" {
                            return Ok(serde_json::json!(entity.namespace()));
                        }
                        if let Some(val) = entity.get_attribute(member) {
                            if val.is_null() {
                                log::debug!(
                                    "Entity '{}' member '{}' present but NULL; returning Null",
                                    object, member
                                );
                            }
                            return Ok(val.clone());
                        }
                        log::debug!(
                            "Entity '{}' found but member '{}' missing; returning Null",
                            object, member
                        );
                        return Ok(serde_json::Value::Null);
                    } else {
                        log::debug!(
                            "Entity lookup for '{}' returned id {} but entity missing; returning Null",
                            object, id
                        );
                    }
                } else {
                    log::debug!(
                        "Entity '{}' not found while resolving member '{}'; continuing lookup",
                        object, member
                    );
                }

                if let Some(id) = graph.find_resource_by_name(object) {
                    if let Some(resource) = graph.get_resource(&id) {
                        if member == "id" {
                            return Ok(serde_json::json!(resource.id().to_string()));
                        } else if member == "name" {
                            return Ok(serde_json::json!(resource.name()));
                        } else if member == "unit" {
                            return Ok(serde_json::json!(resource.unit()));
                        }
                        if let Some(val) = resource.get_attribute(member) {
                            if val.is_null() {
                                log::debug!(
                                    "Resource '{}' member '{}' present but NULL; returning Null",
                                    object, member
                                );
                            }
                            return Ok(val.clone());
                        }
                        log::debug!(
                            "Resource '{}' found but member '{}' missing; returning Null",
                            object, member
                        );
                        return Ok(serde_json::Value::Null);
                    } else {
                        log::debug!(
                            "Resource lookup for '{}' returned id {} but resource missing; returning Null",
                            object, id
                        );
                    }
                } else {
                    log::debug!(
                        "Resource '{}' not found while resolving member '{}'; returning Null",
                        object, member
                    );
                }

                // Not found: return Null to indicate unknown member
                log::debug!(
                    "Member access '{}.{}' did not resolve to entity or resource; returning Null",
                    object, member
                );
                Ok(serde_json::Value::Null)
            }
            Expression::Aggregation { function, collection, field, filter } => {
                let v = Expression::evaluate_aggregation(function, collection, field, filter, graph)?;
                Ok(v)
            }
            Expression::AggregationComprehension { function, variable, collection, predicate, projection, target_unit } => {
                let v = Expression::evaluate_aggregation_comprehension(function, variable, collection, predicate, projection, target_unit.as_deref(), graph)?;
                Ok(v)
            }
            Expression::QuantityLiteral { value, unit } => Ok(serde_json::json!({"__quantity_value": value.to_string(), "__quantity_unit": unit})),
            _ => Err("Expected a runtime-resolvable expression (literal, member access, or aggregation)".to_string()),
        }
    }
}

impl PolicyModality {
    pub fn to_severity(&self) -> Severity {
        match self {
            Self::Obligation => Severity::Error,
            Self::Prohibition => Severity::Error,
            Self::Permission => Severity::Info,
        }
    }
}

impl EvaluationResult {
    pub fn has_errors(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count()
    }
}
