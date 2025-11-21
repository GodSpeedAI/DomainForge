use crate::policy::{AggregateFunction as RustAggregateFunction, BinaryOp as RustBinaryOp};
use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

impl From<AggregateFunction> for RustAggregateFunction {
    fn from(py_agg: AggregateFunction) -> Self {
        match py_agg {
            AggregateFunction::Count => RustAggregateFunction::Count,
            AggregateFunction::Sum => RustAggregateFunction::Sum,
            AggregateFunction::Min => RustAggregateFunction::Min,
            AggregateFunction::Max => RustAggregateFunction::Max,
            AggregateFunction::Avg => RustAggregateFunction::Avg,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum BinaryOp {
    And,
    Or,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    Contains,
    StartsWith,
    EndsWith,
}

impl From<BinaryOp> for RustBinaryOp {
    fn from(py_op: BinaryOp) -> Self {
        match py_op {
            BinaryOp::And => RustBinaryOp::And,
            BinaryOp::Or => RustBinaryOp::Or,
            BinaryOp::Equal => RustBinaryOp::Equal,
            BinaryOp::NotEqual => RustBinaryOp::NotEqual,
            BinaryOp::GreaterThan => RustBinaryOp::GreaterThan,
            BinaryOp::LessThan => RustBinaryOp::LessThan,
            BinaryOp::GreaterThanOrEqual => RustBinaryOp::GreaterThanOrEqual,
            BinaryOp::LessThanOrEqual => RustBinaryOp::LessThanOrEqual,
            BinaryOp::Plus => RustBinaryOp::Plus,
            BinaryOp::Minus => RustBinaryOp::Minus,
            BinaryOp::Multiply => RustBinaryOp::Multiply,
            BinaryOp::Divide => RustBinaryOp::Divide,
            BinaryOp::Contains => RustBinaryOp::Contains,
            BinaryOp::StartsWith => RustBinaryOp::StartsWith,
            BinaryOp::EndsWith => RustBinaryOp::EndsWith,
        }
    }
}

/// Severity level for policy violations
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl From<crate::policy::Severity> for Severity {
    fn from(severity: crate::policy::Severity) -> Self {
        match severity {
            crate::policy::Severity::Error => Severity::Error,
            crate::policy::Severity::Warning => Severity::Warning,
            crate::policy::Severity::Info => Severity::Info,
        }
    }
}

/// A policy violation
#[pyclass]
#[derive(Debug, Clone)]
pub struct Violation {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub severity: Severity,
}

#[pymethods]
impl Violation {
    fn __repr__(&self) -> String {
        format!(
            "Violation(name='{}', message='{}', severity={:?})",
            self.name, self.message, self.severity
        )
    }
}

impl From<crate::policy::Violation> for Violation {
    fn from(v: crate::policy::Violation) -> Self {
        Self {
            name: v.policy_name,
            message: v.message,
            severity: v.severity.into(),
        }
    }
}

/// Result of evaluating a policy against a graph
#[pyclass]
#[derive(Clone, Debug)]
pub struct EvaluationResult {
    /// Backwards compatible boolean: false if evaluation is unknown (NULL)
    #[pyo3(get)]
    pub is_satisfied: bool,
    /// Tri-state evaluation result: True, False, or None (NULL)
    #[pyo3(get)]
    pub is_satisfied_tristate: Option<bool>,
    /// List of violations
    #[pyo3(get)]
    pub violations: Vec<Violation>,
}

#[pymethods]
impl EvaluationResult {
    fn __repr__(&self) -> String {
        format!(
            "EvaluationResult(is_satisfied={}, is_satisfied_tristate={:?}, violations={})",
            self.is_satisfied,
            self.is_satisfied_tristate,
            self.violations.len()
        )
    }
}

impl From<crate::policy::EvaluationResult> for EvaluationResult {
    fn from(result: crate::policy::EvaluationResult) -> Self {
        Self {
            is_satisfied: result.is_satisfied,
            is_satisfied_tristate: result.is_satisfied_tristate,
            violations: result.violations.into_iter().map(|v| v.into()).collect(),
        }
    }
}

// Note: Full Expression and Policy bindings would require:
// 1. Expression class with methods for each variant (literal, variable, binary, etc.)
// 2. Policy class with evaluate method
// 3. Proper conversion between Python and Rust types
// 4. This is left for future implementation as it requires significant boilerplate
