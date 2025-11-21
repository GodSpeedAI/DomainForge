mod core;
mod expression;
mod quantifier;
pub mod three_valued;
mod type_infer;
mod violation;

#[cfg(test)]
mod three_valued_microbench;

pub use core::{DeonticModality, EvaluationResult, Policy, PolicyKind, PolicyModality};
pub use expression::{AggregateFunction, BinaryOp, Expression, Quantifier, UnaryOp};
pub use three_valued::ThreeValuedBool;
pub use type_infer::{infer_expression_type, ExpressionType};
pub use violation::{Severity, Violation};
