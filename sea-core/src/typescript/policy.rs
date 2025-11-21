use crate::policy::{AggregateFunction as RustAggregateFunction, BinaryOp as RustBinaryOp};
use napi_derive::napi;

#[napi]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

impl From<AggregateFunction> for RustAggregateFunction {
    fn from(ts_agg: AggregateFunction) -> Self {
        match ts_agg {
            AggregateFunction::Count => RustAggregateFunction::Count,
            AggregateFunction::Sum => RustAggregateFunction::Sum,
            AggregateFunction::Min => RustAggregateFunction::Min,
            AggregateFunction::Max => RustAggregateFunction::Max,
            AggregateFunction::Avg => RustAggregateFunction::Avg,
        }
    }
}

#[napi]
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
    fn from(ts_op: BinaryOp) -> Self {
        match ts_op {
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
#[napi]
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
#[napi(object)]
#[derive(Clone)]
pub struct Violation {
    pub name: String,
    pub message: String,
    pub severity: Severity,
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
#[napi(object)]
#[derive(Clone)]
pub struct EvaluationResult {
    /// Backwards compatible boolean: false if evaluation is unknown (NULL)
    pub is_satisfied: bool,
    /// Tri-state evaluation result: true, false, or null (NULL)
    pub is_satisfied_tristate: Option<bool>,
    /// List of violations
    pub violations: Vec<Violation>,
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
// 1. Expression struct with methods for each variant
// 2. Policy struct with evaluate method
// 3. Proper conversion between TypeScript and Rust types
// 4. This is left for future implementation
