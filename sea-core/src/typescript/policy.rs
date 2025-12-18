use crate::policy::{
    AggregateFunction as RustAggregateFunction, BinaryOp as RustBinaryOp,
    Expression as RustExpression, NormalizedExpression as RustNormalizedExpression,
    Quantifier as RustQuantifier, UnaryOp as RustUnaryOp, WindowSpec as RustWindowSpec,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rust_decimal::Decimal;
use std::str::FromStr;

// =============================================================================
// Enums
// =============================================================================

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

impl From<RustAggregateFunction> for AggregateFunction {
    fn from(rust_agg: RustAggregateFunction) -> Self {
        match rust_agg {
            RustAggregateFunction::Count => AggregateFunction::Count,
            RustAggregateFunction::Sum => AggregateFunction::Sum,
            RustAggregateFunction::Min => AggregateFunction::Min,
            RustAggregateFunction::Max => AggregateFunction::Max,
            RustAggregateFunction::Avg => AggregateFunction::Avg,
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
    Matches,
    HasRole,
    Before,
    After,
    During,
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
            BinaryOp::Matches => RustBinaryOp::Matches,
            BinaryOp::HasRole => RustBinaryOp::HasRole,
            BinaryOp::Before => RustBinaryOp::Before,
            BinaryOp::After => RustBinaryOp::After,
            BinaryOp::During => RustBinaryOp::During,
        }
    }
}

impl From<RustBinaryOp> for BinaryOp {
    fn from(rust_op: RustBinaryOp) -> Self {
        match rust_op {
            RustBinaryOp::And => BinaryOp::And,
            RustBinaryOp::Or => BinaryOp::Or,
            RustBinaryOp::Equal => BinaryOp::Equal,
            RustBinaryOp::NotEqual => BinaryOp::NotEqual,
            RustBinaryOp::GreaterThan => BinaryOp::GreaterThan,
            RustBinaryOp::LessThan => BinaryOp::LessThan,
            RustBinaryOp::GreaterThanOrEqual => BinaryOp::GreaterThanOrEqual,
            RustBinaryOp::LessThanOrEqual => BinaryOp::LessThanOrEqual,
            RustBinaryOp::Plus => BinaryOp::Plus,
            RustBinaryOp::Minus => BinaryOp::Minus,
            RustBinaryOp::Multiply => BinaryOp::Multiply,
            RustBinaryOp::Divide => BinaryOp::Divide,
            RustBinaryOp::Contains => BinaryOp::Contains,
            RustBinaryOp::StartsWith => BinaryOp::StartsWith,
            RustBinaryOp::EndsWith => BinaryOp::EndsWith,
            RustBinaryOp::Matches => BinaryOp::Matches,
            RustBinaryOp::HasRole => BinaryOp::HasRole,
            RustBinaryOp::Before => BinaryOp::Before,
            RustBinaryOp::After => BinaryOp::After,
            RustBinaryOp::During => BinaryOp::During,
        }
    }
}

#[napi]
pub enum UnaryOp {
    Not,
    Negate,
}

impl From<UnaryOp> for RustUnaryOp {
    fn from(ts_op: UnaryOp) -> Self {
        match ts_op {
            UnaryOp::Not => RustUnaryOp::Not,
            UnaryOp::Negate => RustUnaryOp::Negate,
        }
    }
}

impl From<RustUnaryOp> for UnaryOp {
    fn from(rust_op: RustUnaryOp) -> Self {
        match rust_op {
            RustUnaryOp::Not => UnaryOp::Not,
            RustUnaryOp::Negate => UnaryOp::Negate,
        }
    }
}

#[napi]
pub enum Quantifier {
    ForAll,
    Exists,
    ExistsUnique,
}

impl From<Quantifier> for RustQuantifier {
    fn from(ts_q: Quantifier) -> Self {
        match ts_q {
            Quantifier::ForAll => RustQuantifier::ForAll,
            Quantifier::Exists => RustQuantifier::Exists,
            Quantifier::ExistsUnique => RustQuantifier::ExistsUnique,
        }
    }
}

impl From<RustQuantifier> for Quantifier {
    fn from(rust_q: RustQuantifier) -> Self {
        match rust_q {
            RustQuantifier::ForAll => Quantifier::ForAll,
            RustQuantifier::Exists => Quantifier::Exists,
            RustQuantifier::ExistsUnique => Quantifier::ExistsUnique,
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

// =============================================================================
// WindowSpec
// =============================================================================

#[napi(object)]
#[derive(Clone)]
pub struct WindowSpec {
    pub duration: u32,
    pub unit: String,
}

impl From<WindowSpec> for RustWindowSpec {
    fn from(ts_ws: WindowSpec) -> Self {
        RustWindowSpec {
            duration: ts_ws.duration as u64,
            unit: ts_ws.unit,
        }
    }
}

impl From<RustWindowSpec> for WindowSpec {
    fn from(rust_ws: RustWindowSpec) -> Self {
        WindowSpec {
            duration: rust_ws.duration.try_into().unwrap_or(u32::MAX),
            unit: rust_ws.unit,
        }
    }
}

// =============================================================================
// Expression
// =============================================================================

/// A policy expression that can be normalized and compared for equivalence.
#[napi]
pub struct Expression {
    inner: RustExpression,
}

#[napi]
impl Expression {
    // -------------------------------------------------------------------------
    // Factory methods for all Expression variants
    // -------------------------------------------------------------------------

    /// Create a literal expression from a JSON-compatible value (passed as JSON string).
    #[napi(factory)]
    pub fn literal(value_json: String) -> Result<Self> {
        let json_val: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| Error::from_reason(format!("Invalid JSON literal: {}", e)))?;
        Ok(Self {
            inner: RustExpression::Literal(json_val),
        })
    }

    /// Create a literal boolean expression.
    #[napi(factory)]
    pub fn literal_bool(value: bool) -> Self {
        Self {
            inner: RustExpression::Literal(serde_json::Value::Bool(value)),
        }
    }

    /// Create a literal number expression.
    #[napi(factory)]
    pub fn literal_number(value: f64) -> Self {
        Self {
            inner: RustExpression::Literal(serde_json::json!(value)),
        }
    }

    /// Create a literal string expression.
    #[napi(factory)]
    pub fn literal_string(value: String) -> Self {
        Self {
            inner: RustExpression::Literal(serde_json::Value::String(value)),
        }
    }

    /// Create a variable expression.
    #[napi(factory)]
    pub fn variable(name: String) -> Self {
        Self {
            inner: RustExpression::Variable(name),
        }
    }

    /// Create a quantity literal expression (e.g., "100 USD").
    #[napi(factory)]
    pub fn quantity(value: String, unit: String) -> Result<Self> {
        let decimal = Decimal::from_str(&value)
            .map_err(|e| Error::from_reason(format!("Invalid decimal value: {}", e)))?;
        Ok(Self {
            inner: RustExpression::QuantityLiteral {
                value: decimal,
                unit,
            },
        })
    }

    /// Create a time literal expression (ISO 8601 timestamp).
    #[napi(factory)]
    pub fn time(timestamp: String) -> Self {
        Self {
            inner: RustExpression::TimeLiteral(timestamp),
        }
    }

    /// Create an interval literal expression.
    #[napi(factory)]
    pub fn interval(start: String, end: String) -> Self {
        Self {
            inner: RustExpression::IntervalLiteral { start, end },
        }
    }

    /// Create a binary expression (e.g., left AND right).
    #[napi(factory)]
    pub fn binary(op: BinaryOp, left: &Expression, right: &Expression) -> Self {
        Self {
            inner: RustExpression::Binary {
                op: op.into(),
                left: Box::new(left.inner.clone()),
                right: Box::new(right.inner.clone()),
            },
        }
    }

    /// Create a unary expression (e.g., NOT x).
    #[napi(factory)]
    pub fn unary(op: UnaryOp, operand: &Expression) -> Self {
        Self {
            inner: RustExpression::Unary {
                op: op.into(),
                operand: Box::new(operand.inner.clone()),
            },
        }
    }

    /// Create a cast expression (e.g., x as "Money").
    #[napi(factory)]
    pub fn cast(operand: &Expression, target_type: String) -> Self {
        Self {
            inner: RustExpression::Cast {
                operand: Box::new(operand.inner.clone()),
                target_type,
            },
        }
    }

    /// Create a quantifier expression (ForAll, Exists, ExistsUnique).
    #[napi(factory)]
    pub fn quantifier(
        q: Quantifier,
        variable: String,
        collection: &Expression,
        condition: &Expression,
    ) -> Self {
        Self {
            inner: RustExpression::Quantifier {
                quantifier: q.into(),
                variable,
                collection: Box::new(collection.inner.clone()),
                condition: Box::new(condition.inner.clone()),
            },
        }
    }

    /// Create a member access expression (e.g., user.name).
    #[napi(factory)]
    pub fn member_access(object: String, member: String) -> Self {
        Self {
            inner: RustExpression::MemberAccess { object, member },
        }
    }

    /// Create an aggregation expression (e.g., COUNT(items)).
    #[napi(factory)]
    pub fn aggregation(
        function: AggregateFunction,
        collection: &Expression,
        field: Option<String>,
        filter: Option<&Expression>,
    ) -> Self {
        Self {
            inner: RustExpression::Aggregation {
                function: function.into(),
                collection: Box::new(collection.inner.clone()),
                field,
                filter: filter.map(|e| Box::new(e.inner.clone())),
            },
        }
    }

    /// Create an aggregation comprehension expression.
    #[napi(factory)]
    pub fn aggregation_comprehension(
        function: AggregateFunction,
        variable: String,
        collection: &Expression,
        predicate: &Expression,
        projection: &Expression,
        window: Option<WindowSpec>,
        target_unit: Option<String>,
    ) -> Self {
        Self {
            inner: RustExpression::AggregationComprehension {
                function: function.into(),
                variable,
                collection: Box::new(collection.inner.clone()),
                window: window.map(|w| w.into()),
                predicate: Box::new(predicate.inner.clone()),
                projection: Box::new(projection.inner.clone()),
                target_unit,
            },
        }
    }

    /// Create a group-by expression.
    #[napi(factory)]
    pub fn group_by(
        variable: String,
        collection: &Expression,
        key: &Expression,
        condition: &Expression,
        filter: Option<&Expression>,
    ) -> Self {
        Self {
            inner: RustExpression::GroupBy {
                variable,
                collection: Box::new(collection.inner.clone()),
                filter: filter.map(|e| Box::new(e.inner.clone())),
                key: Box::new(key.inner.clone()),
                condition: Box::new(condition.inner.clone()),
            },
        }
    }

    // -------------------------------------------------------------------------
    // Normalization methods
    // -------------------------------------------------------------------------

    /// Normalize this expression to canonical form.
    #[napi]
    pub fn normalize(&self) -> NormalizedExpression {
        NormalizedExpression {
            inner: self.inner.normalize(),
        }
    }

    /// Check if this expression is semantically equivalent to another.
    #[napi]
    pub fn is_equivalent(&self, other: &Expression) -> bool {
        self.inner.is_equivalent(&other.inner)
    }

    /// Get the string representation of this expression.
    #[napi]
    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    /// Check equality with another expression.
    #[napi]
    pub fn equals(&self, other: &Expression) -> bool {
        self.inner == other.inner
    }
}

impl From<RustExpression> for Expression {
    fn from(expr: RustExpression) -> Self {
        Self { inner: expr }
    }
}

impl Expression {
    pub fn into_inner(self) -> RustExpression {
        self.inner
    }
}

// =============================================================================
// NormalizedExpression
// =============================================================================

/// A normalized expression with a stable hash for caching and equivalence checks.
#[napi]
pub struct NormalizedExpression {
    inner: RustNormalizedExpression,
}

#[napi]
impl NormalizedExpression {
    /// Get the stable hash value for this normalized expression as a string.
    #[napi]
    pub fn stable_hash(&self) -> String {
        self.inner.stable_hash().to_string()
    }

    /// Get the stable hash as a hex string.
    #[napi]
    pub fn stable_hash_hex(&self) -> String {
        format!("{:#018x}", self.inner.stable_hash())
    }

    /// Get the inner expression.
    #[napi]
    pub fn inner_expression(&self) -> Expression {
        Expression {
            inner: self.inner.inner().clone(),
        }
    }

    /// Get the string representation of this normalized expression.
    #[napi]
    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    /// Check equality with another normalized expression.
    #[napi]
    pub fn equals(&self, other: &NormalizedExpression) -> bool {
        self.inner == other.inner
    }
}

// =============================================================================
// Violation and EvaluationResult
// =============================================================================

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
