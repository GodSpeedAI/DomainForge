use super::expression::{AggregateFunction, BinaryOp, Expression, UnaryOp};
use crate::graph::Graph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionType {
    Boolean,
    Numeric,
    Quantity,
    String,
    Unknown,
}

/// Very small static type inference pass to classify expression into basic categories
pub fn infer_expression_type(expr: &Expression, _graph: &Graph) -> ExpressionType {
    match expr {
        Expression::Literal(v) => {
            if v.is_boolean() {
                ExpressionType::Boolean
            } else if v.is_number() {
                ExpressionType::Numeric
            } else if v.is_string() {
                ExpressionType::String
            } else {
                ExpressionType::Unknown
            }
        }
        Expression::QuantityLiteral { .. } => ExpressionType::Quantity,
        Expression::Variable(_) => ExpressionType::Unknown,
        Expression::Binary { op, .. } => match op {
            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::GreaterThan | BinaryOp::LessThan | BinaryOp::GreaterThanOrEqual | BinaryOp::LessThanOrEqual | BinaryOp::Contains | BinaryOp::StartsWith | BinaryOp::EndsWith | BinaryOp::And | BinaryOp::Or => {
                // Comparisons and boolean ops typically yield boolean
                ExpressionType::Boolean
            }
            BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Multiply | BinaryOp::Divide => {
                // arithmetic yields numeric
                ExpressionType::Numeric
            }
        },
        Expression::Unary { op, .. } => match op {
            UnaryOp::Not => ExpressionType::Boolean,
            UnaryOp::Negate => ExpressionType::Numeric,
        },
        Expression::Quantifier { .. } => ExpressionType::Boolean,
        Expression::MemberAccess { .. } => ExpressionType::Unknown,
        Expression::Aggregation { function, .. } => match function {
            AggregateFunction::Count => ExpressionType::Numeric,
            AggregateFunction::Sum | AggregateFunction::Avg | AggregateFunction::Min | AggregateFunction::Max => ExpressionType::Numeric,
        },
        Expression::AggregationComprehension { .. } => ExpressionType::Numeric,
        // All expression kinds are covered above
    }
}
