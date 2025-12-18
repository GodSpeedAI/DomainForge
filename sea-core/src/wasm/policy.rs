use crate::policy::{
    AggregateFunction as RustAggregateFunction, BinaryOp as RustBinaryOp,
    Expression as RustExpression, NormalizedExpression as RustNormalizedExpression,
    Quantifier as RustQuantifier, UnaryOp as RustUnaryOp, WindowSpec as RustWindowSpec,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

// =============================================================================
// Enums
// =============================================================================

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum AggregateFunction {
    Count = 0,
    Sum = 1,
    Min = 2,
    Max = 3,
    Avg = 4,
}

impl From<AggregateFunction> for RustAggregateFunction {
    fn from(wasm_agg: AggregateFunction) -> Self {
        match wasm_agg {
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    And = 0,
    Or = 1,
    Equal = 2,
    NotEqual = 3,
    GreaterThan = 4,
    LessThan = 5,
    GreaterThanOrEqual = 6,
    LessThanOrEqual = 7,
    Plus = 8,
    Minus = 9,
    Multiply = 10,
    Divide = 11,
    Contains = 12,
    StartsWith = 13,
    EndsWith = 14,
    Matches = 15,
    HasRole = 16,
    Before = 17,
    After = 18,
    During = 19,
}

impl From<BinaryOp> for RustBinaryOp {
    fn from(wasm_op: BinaryOp) -> Self {
        match wasm_op {
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    Not = 0,
    Negate = 1,
}

impl From<UnaryOp> for RustUnaryOp {
    fn from(wasm_op: UnaryOp) -> Self {
        match wasm_op {
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum Quantifier {
    ForAll = 0,
    Exists = 1,
    ExistsUnique = 2,
}

impl From<Quantifier> for RustQuantifier {
    fn from(wasm_q: Quantifier) -> Self {
        match wasm_q {
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
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Error = 0,
    Warning = 1,
    Info = 2,
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct WindowSpec {
    #[wasm_bindgen(readonly)]
    pub duration: u32,
    #[wasm_bindgen(readonly)]
    pub unit: String,
}

#[wasm_bindgen]
impl WindowSpec {
    #[wasm_bindgen(constructor)]
    pub fn new(duration: u32, unit: String) -> Self {
        Self { duration, unit }
    }
}

impl From<WindowSpec> for RustWindowSpec {
    fn from(wasm_ws: WindowSpec) -> Self {
        RustWindowSpec {
            duration: wasm_ws.duration as u64,
            unit: wasm_ws.unit,
        }
    }
}

impl TryFrom<RustWindowSpec> for WindowSpec {
    type Error = String;

    fn try_from(rust_ws: RustWindowSpec) -> Result<Self, Self::Error> {
        let duration = rust_ws.duration.try_into().map_err(|_| {
            format!(
                "Window duration {} exceeds maximum allowed value (u32::MAX)",
                rust_ws.duration
            )
        })?;

        Ok(WindowSpec {
            duration,
            unit: rust_ws.unit,
        })
    }
}

// =============================================================================
// Expression
// =============================================================================

/// A policy expression that can be normalized and compared for equivalence.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Expression {
    inner: RustExpression,
}

#[wasm_bindgen]
impl Expression {
    // -------------------------------------------------------------------------
    // Factory methods for all Expression variants
    // -------------------------------------------------------------------------

    /// Create a literal expression from a JSON string.
    #[wasm_bindgen]
    pub fn literal(value_json: &str) -> Result<Expression, JsError> {
        let json_val: serde_json::Value = serde_json::from_str(value_json)
            .map_err(|e| JsError::new(&format!("Invalid JSON literal: {}", e)))?;
        Ok(Self {
            inner: RustExpression::Literal(json_val),
        })
    }

    /// Create a literal boolean expression.
    #[wasm_bindgen(js_name = literalBool)]
    pub fn literal_bool(value: bool) -> Expression {
        Self {
            inner: RustExpression::Literal(serde_json::Value::Bool(value)),
        }
    }

    /// Create a literal number expression.
    #[wasm_bindgen(js_name = literalNumber)]
    pub fn literal_number(value: f64) -> Result<Expression, JsError> {
        if !value.is_finite() {
            return Err(JsError::new("literalNumber: value must be a finite number"));
        }
        Ok(Self {
            inner: RustExpression::Literal(serde_json::json!(value)),
        })
    }

    /// Create a literal string expression.
    #[wasm_bindgen(js_name = literalString)]
    pub fn literal_string(value: &str) -> Expression {
        Self {
            inner: RustExpression::Literal(serde_json::Value::String(value.to_string())),
        }
    }

    /// Create a variable expression.
    #[wasm_bindgen]
    pub fn variable(name: &str) -> Expression {
        Self {
            inner: RustExpression::Variable(name.to_string()),
        }
    }

    /// Create a quantity literal expression (e.g., "100 USD").
    #[wasm_bindgen]
    pub fn quantity(value: &str, unit: &str) -> Result<Expression, JsError> {
        let decimal = Decimal::from_str(value)
            .map_err(|e| JsError::new(&format!("Invalid decimal value: {}", e)))?;
        Ok(Self {
            inner: RustExpression::QuantityLiteral {
                value: decimal,
                unit: unit.to_string(),
            },
        })
    }

    /// Create a time literal expression (ISO 8601 timestamp).
    #[wasm_bindgen]
    pub fn time(timestamp: &str) -> Expression {
        Self {
            inner: RustExpression::TimeLiteral(timestamp.to_string()),
        }
    }

    /// Create an interval literal expression.
    #[wasm_bindgen]
    pub fn interval(start: &str, end: &str) -> Expression {
        Self {
            inner: RustExpression::IntervalLiteral {
                start: start.to_string(),
                end: end.to_string(),
            },
        }
    }

    /// Create a binary expression (e.g., left AND right).
    #[wasm_bindgen]
    pub fn binary(op: BinaryOp, left: &Expression, right: &Expression) -> Expression {
        Self {
            inner: RustExpression::Binary {
                op: op.into(),
                left: Box::new(left.inner.clone()),
                right: Box::new(right.inner.clone()),
            },
        }
    }

    /// Create a unary expression (e.g., NOT x).
    #[wasm_bindgen]
    pub fn unary(op: UnaryOp, operand: &Expression) -> Expression {
        Self {
            inner: RustExpression::Unary {
                op: op.into(),
                operand: Box::new(operand.inner.clone()),
            },
        }
    }

    /// Create a cast expression (e.g., x as "Money").
    #[wasm_bindgen]
    pub fn cast(operand: &Expression, target_type: &str) -> Expression {
        Self {
            inner: RustExpression::Cast {
                operand: Box::new(operand.inner.clone()),
                target_type: target_type.to_string(),
            },
        }
    }

    /// Create a quantifier expression (ForAll, Exists, ExistsUnique).
    #[wasm_bindgen]
    pub fn quantifier_expr(
        q: Quantifier,
        variable: &str,
        collection: &Expression,
        condition: &Expression,
    ) -> Expression {
        Self {
            inner: RustExpression::Quantifier {
                quantifier: q.into(),
                variable: variable.to_string(),
                collection: Box::new(collection.inner.clone()),
                condition: Box::new(condition.inner.clone()),
            },
        }
    }

    /// Create a member access expression (e.g., user.name).
    #[wasm_bindgen(js_name = memberAccess)]
    pub fn member_access(object: &str, member: &str) -> Expression {
        Self {
            inner: RustExpression::MemberAccess {
                object: object.to_string(),
                member: member.to_string(),
            },
        }
    }

    /// Create an aggregation expression (e.g., COUNT(items)).
    #[wasm_bindgen]
    pub fn aggregation(
        function: AggregateFunction,
        collection: &Expression,
        field: Option<String>,
        filter: Option<Expression>,
    ) -> Expression {
        Self {
            inner: RustExpression::Aggregation {
                function: function.into(),
                collection: Box::new(collection.inner.clone()),
                field,
                filter: filter.map(|e| Box::new(e.inner)),
            },
        }
    }

    /// Create an aggregation comprehension expression.
    #[wasm_bindgen(js_name = aggregationComprehension)]
    pub fn aggregation_comprehension(
        function: AggregateFunction,
        variable: &str,
        collection: &Expression,
        predicate: &Expression,
        projection: &Expression,
        window: Option<WindowSpec>,
        target_unit: Option<String>,
    ) -> Expression {
        Self {
            inner: RustExpression::AggregationComprehension {
                function: function.into(),
                variable: variable.to_string(),
                collection: Box::new(collection.inner.clone()),
                window: window.map(|w| w.into()),
                predicate: Box::new(predicate.inner.clone()),
                projection: Box::new(projection.inner.clone()),
                target_unit,
            },
        }
    }

    /// Create a group-by expression.
    #[wasm_bindgen(js_name = groupBy)]
    pub fn group_by(
        variable: &str,
        collection: &Expression,
        key: &Expression,
        condition: &Expression,
        filter: Option<Expression>,
    ) -> Expression {
        Self {
            inner: RustExpression::GroupBy {
                variable: variable.to_string(),
                collection: Box::new(collection.inner.clone()),
                filter: filter.map(|e| Box::new(e.inner)),
                key: Box::new(key.inner.clone()),
                condition: Box::new(condition.inner.clone()),
            },
        }
    }

    // -------------------------------------------------------------------------
    // Normalization methods
    // -------------------------------------------------------------------------

    /// Normalize this expression to canonical form.
    #[wasm_bindgen]
    pub fn normalize(&self) -> NormalizedExpression {
        NormalizedExpression {
            inner: self.inner.normalize(),
        }
    }

    /// Check if this expression is semantically equivalent to another.
    #[wasm_bindgen(js_name = isEquivalent)]
    pub fn is_equivalent(&self, other: &Expression) -> bool {
        self.inner.is_equivalent(&other.inner)
    }

    /// Get the string representation of this expression.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    /// Check equality with another expression.
    #[wasm_bindgen]
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
#[wasm_bindgen]
#[derive(Clone)]
pub struct NormalizedExpression {
    inner: RustNormalizedExpression,
}

#[wasm_bindgen]
impl NormalizedExpression {
    /// Get the stable hash value for this normalized expression as a string.
    #[wasm_bindgen(js_name = stableHash)]
    pub fn stable_hash(&self) -> String {
        self.inner.stable_hash().to_string()
    }

    /// Get the stable hash as a hex string.
    #[wasm_bindgen(js_name = stableHashHex)]
    pub fn stable_hash_hex(&self) -> String {
        format!("{:#018x}", self.inner.stable_hash())
    }

    /// Get the inner expression.
    #[wasm_bindgen(js_name = innerExpression)]
    pub fn inner_expression(&self) -> Expression {
        Expression {
            inner: self.inner.inner().clone(),
        }
    }

    /// Get the string representation of this normalized expression.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    /// Check equality with another normalized expression.
    #[wasm_bindgen]
    pub fn equals(&self, other: &NormalizedExpression) -> bool {
        self.inner == other.inner
    }
}

// =============================================================================
// Violation and EvaluationResult
// =============================================================================

/// A policy violation
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Violation {
    #[wasm_bindgen(readonly)]
    pub name: String,
    #[wasm_bindgen(readonly)]
    pub message: String,
    #[wasm_bindgen(readonly)]
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
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct EvaluationResult {
    /// Backward-compatible boolean result (false if evaluation is NULL)
    #[wasm_bindgen(readonly, js_name = isSatisfied)]
    pub is_satisfied: bool,

    /// Three-valued result: true, false, or undefined (NULL)
    /// Note: In WASM, Option<bool> where None becomes undefined in JS
    #[wasm_bindgen(readonly, js_name = isSatisfiedTristate)]
    pub is_satisfied_tristate: Option<bool>,

    /// List of violations
    #[wasm_bindgen(readonly)]
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
