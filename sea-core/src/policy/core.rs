use super::expression::{BinaryOp, Expression, UnaryOp};
use super::violation::{Severity, Violation};
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
    pub is_satisfied: bool,
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
        let expanded = self.expression.expand(graph)?;

        let is_satisfied = self.evaluate_expression(&expanded, graph)?;

        let violations = if !is_satisfied {
            vec![Violation::new(
                &self.name,
                format!("Policy '{}' was violated", self.name),
                self.modality.to_severity(),
            )]
        } else {
            vec![]
        };

        Ok(EvaluationResult {
            is_satisfied,
            violations,
        })
    }

    fn evaluate_expression(&self, expr: &Expression, graph: &Graph) -> Result<bool, String> {
        match expr {
            Expression::Literal(v) => v
                .as_bool()
                .ok_or_else(|| format!("Expected boolean literal, got: {}", v)),
            Expression::Variable(name) => {
                Err(format!("Cannot evaluate unexpanded variable: {}", name))
            }
            Expression::Binary { op, left, right } => match op {
                BinaryOp::And | BinaryOp::Or => {
                    let left_val = self.evaluate_expression(left, graph)?;
                    let right_val = self.evaluate_expression(right, graph)?;

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
                let val = self.evaluate_expression(operand, graph)?;
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
            Expression::MemberAccess { .. } => {
                Err("Cannot evaluate non-expanded member access".to_string())
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

    fn compare_values<F>(
        &self,
        left: &Expression,
        right: &Expression,
        _graph: &Graph,
        op: F,
    ) -> Result<bool, String>
    where
        F: Fn(&serde_json::Value, &serde_json::Value) -> bool,
    {
        let left_val = self.get_literal_value(left)?;
        let right_val = self.get_literal_value(right)?;
        Ok(op(&left_val, &right_val))
    }

    fn compare_numeric<F>(
        &self,
        left: &Expression,
        right: &Expression,
        _graph: &Graph,
        op: F,
    ) -> Result<bool, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_val = self.get_numeric_value(left)?;
        let right_val = self.get_numeric_value(right)?;
        Ok(op(left_val, right_val))
    }

    fn compare_strings<F>(
        &self,
        left: &Expression,
        right: &Expression,
        _graph: &Graph,
        op: F,
    ) -> Result<bool, String>
    where
        F: Fn(&str, &str) -> bool,
    {
        let left_val = self.get_string_value(left)?;
        let right_val = self.get_string_value(right)?;
        Ok(op(&left_val, &right_val))
    }

    fn get_literal_value(&self, expr: &Expression) -> Result<serde_json::Value, String> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            _ => Err("Expected literal value".to_string()),
        }
    }

    fn get_numeric_value(&self, expr: &Expression) -> Result<f64, String> {
        match expr {
            Expression::Literal(v) => v
                .as_f64()
                .or_else(|| v.as_i64().map(|i| i as f64))
                .ok_or_else(|| "Expected numeric literal".to_string()),
            _ => Err("Expected literal value".to_string()),
        }
    }

    fn get_string_value(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Literal(v) => v
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Expected string literal".to_string()),
            _ => Err("Expected literal value".to_string()),
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
