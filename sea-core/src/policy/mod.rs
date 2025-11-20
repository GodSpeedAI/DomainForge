mod core;
mod expression;
mod quantifier;
mod type_infer;
mod violation;
#[cfg(feature = "three_valued_logic")]
mod three_valued;

pub use core::{DeonticModality, EvaluationResult, Policy, PolicyKind, PolicyModality};
pub use expression::{AggregateFunction, BinaryOp, Expression, Quantifier, UnaryOp};
pub use type_infer::{infer_expression_type, ExpressionType};
pub use violation::{Severity, Violation};
#[cfg(feature = "three_valued_logic")]
pub use three_valued::{ThreeValuedBool, aggregators as three_valued_aggregators};
