use serde::{Deserialize, Serialize};
use crate::graph::Graph;
use super::expression::{Expression, BinaryOp, UnaryOp};
use super::violation::{Violation, Severity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeonticModality {
    Obligation,
    Prohibition,
    Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub expression: Expression,
    pub modality: DeonticModality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub is_satisfied: bool,
    pub violations: Vec<Violation>,
}

impl Policy {
    pub fn new(name: impl Into<String>, expression: Expression) -> Self {
        Self {
            name: name.into(),
            expression,
            modality: DeonticModality::Obligation,
        }
    }

    pub fn with_modality(mut self, modality: DeonticModality) -> Self {
        self.modality = modality;
        self
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
            Expression::Literal(v) => {
                v.as_bool().ok_or_else(|| format!("Expected boolean literal, got: {}", v))
            }
            Expression::Variable(name) => {
                Err(format!("Cannot evaluate unexpanded variable: {}", name))
            }
            Expression::Binary { op, left, right } => {
                match op {
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
                    BinaryOp::GreaterThan | BinaryOp::LessThan |
                    BinaryOp::GreaterThanOrEqual | BinaryOp::LessThanOrEqual => {
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
                    BinaryOp::Contains | BinaryOp::StartsWith | BinaryOp::EndsWith => {
                        self.compare_strings(left, right, graph, |l, r| match op {
                            BinaryOp::Contains => l.contains(r),
                            BinaryOp::StartsWith => l.starts_with(r),
                            BinaryOp::EndsWith => l.ends_with(r),
                            _ => unreachable!(),
                        })
                    }
                }
            }
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
            Expression::Literal(v) => {
                v.as_f64()
                    .or_else(|| v.as_i64().map(|i| i as f64))
                    .ok_or_else(|| "Expected numeric literal".to_string())
            }
            _ => Err("Expected literal value".to_string()),
        }
    }

    fn get_string_value(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Literal(v) => {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Expected string literal".to_string())
            }
            _ => Err("Expected literal value".to_string()),
        }
    }
}

impl DeonticModality {
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
        self.violations.iter().any(|v| v.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.violations.iter().filter(|v| v.severity == Severity::Error).count()
    }
}
