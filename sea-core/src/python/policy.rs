use crate::policy::{
    AggregateFunction as RustAggregateFunction, BinaryOp as RustBinaryOp,
    Expression as RustExpression, Policy as RustPolicy, Quantifier as RustQuantifier,
    UnaryOp as RustUnaryOp,
};
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
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

#[pyclass]
#[derive(Clone)]
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

#[pymethods]
impl BinaryOp {
    #[classattr]
    fn __doc__() -> &'static str {
        "Binary operations for policy expressions.\n\n\
         This enum defines various binary operators that can be used in policy\n\
         expressions to compare values, combine conditions, and perform arithmetic.\n\n\
         Examples:\n\
         - Logical operations: And, Or\n\
         - Comparison operations: Equal, GreaterThan, LessThan, etc.\n\
         - Arithmetic operations: Plus, Minus, Multiply, Divide\n\
         - String operations: Contains, StartsWith, EndsWith"
    }

    #[classattr]
    const And: BinaryOp = BinaryOp::And;

    #[doc = "Logical AND operation - returns true if both operands are true"]
    #[classattr]
    const Or: BinaryOp = BinaryOp::Or;

    #[doc = "Equality comparison - returns true if operands are equal"]
    #[classattr]
    const Equal: BinaryOp = BinaryOp::Equal;

    #[doc = "Inequality comparison - returns true if operands are not equal"]
    #[classattr]
    const NotEqual: BinaryOp = BinaryOp::NotEqual;

    #[doc = "Greater than comparison - returns true if left operand is greater than right"]
    #[classattr]
    const GreaterThan: BinaryOp = BinaryOp::GreaterThan;

    #[doc = "Less than comparison - returns true if left operand is less than right"]
    #[classattr]
    const LessThan: BinaryOp = BinaryOp::LessThan;

    #[doc = "Greater than or equal comparison - returns true if left operand is greater than or equal to right"]
    #[classattr]
    const GreaterThanOrEqual: BinaryOp = BinaryOp::GreaterThanOrEqual;

    #[doc = "Less than or equal comparison - returns true if left operand is less than or equal to right"]
    #[classattr]
    const LessThanOrEqual: BinaryOp = BinaryOp::LessThanOrEqual;

    #[doc = "Addition operation - adds two numeric operands"]
    #[classattr]
    const Plus: BinaryOp = BinaryOp::Plus;

    #[doc = "Subtraction operation - subtracts right operand from left operand"]
    #[classattr]
    const Minus: BinaryOp = BinaryOp::Minus;

    #[doc = "Multiplication operation - multiplies two numeric operands"]
    #[classattr]
    const Multiply: BinaryOp = BinaryOp::Multiply;

    #[doc = "Division operation - divides left operand by right operand"]
    #[classattr]
    const Divide: BinaryOp = BinaryOp::Divide;

    #[doc = "String containment check - returns true if left string contains right string"]
    #[classattr]
    const Contains: BinaryOp = BinaryOp::Contains;

    #[doc = "String prefix check - returns true if left string starts with right string"]
    #[classattr]
    const StartsWith: BinaryOp = BinaryOp::StartsWith;

    #[doc = "String suffix check - returns true if left string ends with right string"]
    #[classattr]
    const EndsWith: BinaryOp = BinaryOp::EndsWith;
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

// Note: Full Expression and Policy bindings would require:
// 1. Expression class with methods for each variant (literal, variable, binary, etc.)
// 2. Policy class with evaluate method
// 3. Proper conversion between Python and Rust types
// 4. This is left for future implementation as it requires significant boilerplate
