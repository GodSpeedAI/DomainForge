pub mod type_inference;

mod core;
mod expression;
mod normalize;
mod quantifier;
mod three_valued;
mod violation;

#[cfg(test)]
mod three_valued_microbench;

pub use core::{DeonticModality, EvaluationResult, Policy, PolicyKind, PolicyModality};
pub use expression::{AggregateFunction, BinaryOp, Expression, Quantifier, UnaryOp, WindowSpec};
pub use normalize::NormalizedExpression;
pub use three_valued::ThreeValuedBool;
pub use type_inference::*;
pub use violation::{Severity, Violation};
