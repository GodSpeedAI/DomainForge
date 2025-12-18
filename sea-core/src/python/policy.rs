use crate::policy::{
    AggregateFunction as RustAggregateFunction, BinaryOp as RustBinaryOp,
    Expression as RustExpression, NormalizedExpression as RustNormalizedExpression,
    Quantifier as RustQuantifier, UnaryOp as RustUnaryOp, WindowSpec as RustWindowSpec,
};
use chrono::DateTime;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

// =============================================================================
// Enums
// =============================================================================

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
    Matches,
    HasRole,
    Before,
    After,
    During,
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

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
}

impl From<UnaryOp> for RustUnaryOp {
    fn from(py_op: UnaryOp) -> Self {
        match py_op {
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

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum Quantifier {
    ForAll,
    Exists,
    ExistsUnique,
}

impl From<Quantifier> for RustQuantifier {
    fn from(py_q: Quantifier) -> Self {
        match py_q {
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

// =============================================================================
// WindowSpec
// =============================================================================

#[pyclass]
#[derive(Clone)]
pub struct WindowSpec {
    #[pyo3(get)]
    pub duration: u64,
    #[pyo3(get)]
    pub unit: String,
}

#[pymethods]
impl WindowSpec {
    #[new]
    fn new(duration: u64, unit: String) -> Self {
        Self { duration, unit }
    }

    fn __repr__(&self) -> String {
        format!(
            "WindowSpec(duration={}, unit='{}')",
            self.duration, self.unit
        )
    }
}

impl From<WindowSpec> for RustWindowSpec {
    fn from(py_ws: WindowSpec) -> Self {
        RustWindowSpec {
            duration: py_ws.duration,
            unit: py_ws.unit,
        }
    }
}

impl From<RustWindowSpec> for WindowSpec {
    fn from(rust_ws: RustWindowSpec) -> Self {
        WindowSpec {
            duration: rust_ws.duration,
            unit: rust_ws.unit,
        }
    }
}

// =============================================================================
// Expression
// =============================================================================

/// A policy expression that can be normalized and compared for equivalence.
#[pyclass]
#[derive(Clone)]
pub struct Expression {
    inner: RustExpression,
}

#[pymethods]
impl Expression {
    // -------------------------------------------------------------------------
    // Factory methods for all Expression variants
    // -------------------------------------------------------------------------

    /// Create a literal expression from a JSON-compatible value.
    #[staticmethod]
    fn literal(value: &Bound<'_, pyo3::types::PyAny>) -> PyResult<Self> {
        let json_val = python_to_json(value)?;
        Ok(Self {
            inner: RustExpression::Literal(json_val),
        })
    }

    /// Create a variable expression.
    #[staticmethod]
    fn variable(name: &str) -> Self {
        Self {
            inner: RustExpression::Variable(name.to_string()),
        }
    }

    /// Create a quantity literal expression (e.g., "100 USD").
    #[staticmethod]
    fn quantity(value: &str, unit: &str) -> PyResult<Self> {
        let decimal = Decimal::from_str(value)
            .map_err(|e| PyValueError::new_err(format!("Invalid decimal value: {}", e)))?;
        Ok(Self {
            inner: RustExpression::QuantityLiteral {
                value: decimal,
                unit: unit.to_string(),
            },
        })
    }

    /// Create a time literal expression (ISO 8601 timestamp).
    #[staticmethod]
    fn time(timestamp: &str) -> PyResult<Self> {
        DateTime::parse_from_rfc3339(timestamp)
            .map_err(|e| PyValueError::new_err(format!("Invalid timestamp: {}", e)))?;
        Ok(Self {
            inner: RustExpression::TimeLiteral(timestamp.to_string()),
        })
    }

    /// Create an interval literal expression.
    #[staticmethod]
    fn interval(start: &str, end: &str) -> PyResult<Self> {
        let start_dt = DateTime::parse_from_rfc3339(start)
            .map_err(|e| PyValueError::new_err(format!("Invalid start timestamp: {}", e)))?;
        let end_dt = DateTime::parse_from_rfc3339(end)
            .map_err(|e| PyValueError::new_err(format!("Invalid end timestamp: {}", e)))?;

        if start_dt > end_dt {
            return Err(PyValueError::new_err(
                "Start time must be before or equal to end time",
            ));
        }

        Ok(Self {
            inner: RustExpression::IntervalLiteral {
                start: start.to_string(),
                end: end.to_string(),
            },
        })
    }

    /// Create a binary expression (e.g., left AND right).
    #[staticmethod]
    fn binary(op: BinaryOp, left: &Expression, right: &Expression) -> Self {
        Self {
            inner: RustExpression::Binary {
                op: op.into(),
                left: Box::new(left.inner.clone()),
                right: Box::new(right.inner.clone()),
            },
        }
    }

    /// Create a unary expression (e.g., NOT x).
    #[staticmethod]
    fn unary(op: UnaryOp, operand: &Expression) -> Self {
        Self {
            inner: RustExpression::Unary {
                op: op.into(),
                operand: Box::new(operand.inner.clone()),
            },
        }
    }

    /// Create a cast expression (e.g., x as "Money").
    #[staticmethod]
    fn cast(operand: &Expression, target_type: &str) -> Self {
        Self {
            inner: RustExpression::Cast {
                operand: Box::new(operand.inner.clone()),
                target_type: target_type.to_string(),
            },
        }
    }

    /// Create a quantifier expression (ForAll, Exists, ExistsUnique).
    #[staticmethod]
    fn quantifier(
        q: Quantifier,
        variable: &str,
        collection: &Expression,
        condition: &Expression,
    ) -> Self {
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
    #[staticmethod]
    fn member_access(object: &str, member: &str) -> Self {
        Self {
            inner: RustExpression::MemberAccess {
                object: object.to_string(),
                member: member.to_string(),
            },
        }
    }

    /// Create an aggregation expression (e.g., COUNT(items)).
    #[staticmethod]
    #[pyo3(signature = (function, collection, field=None, filter=None))]
    fn aggregation(
        function: AggregateFunction,
        collection: &Expression,
        field: Option<&str>,
        filter: Option<&Expression>,
    ) -> Self {
        Self {
            inner: RustExpression::Aggregation {
                function: function.into(),
                collection: Box::new(collection.inner.clone()),
                field: field.map(|s| s.to_string()),
                filter: filter.map(|e| Box::new(e.inner.clone())),
            },
        }
    }

    /// Create an aggregation comprehension expression.
    #[staticmethod]
    #[pyo3(signature = (function, variable, collection, predicate, projection, window=None, target_unit=None))]
    fn aggregation_comprehension(
        function: AggregateFunction,
        variable: &str,
        collection: &Expression,
        predicate: &Expression,
        projection: &Expression,
        window: Option<&WindowSpec>,
        target_unit: Option<&str>,
    ) -> Self {
        Self {
            inner: RustExpression::AggregationComprehension {
                function: function.into(),
                variable: variable.to_string(),
                collection: Box::new(collection.inner.clone()),
                window: window.map(|w| w.clone().into()),
                predicate: Box::new(predicate.inner.clone()),
                projection: Box::new(projection.inner.clone()),
                target_unit: target_unit.map(|s| s.to_string()),
            },
        }
    }

    /// Create a group-by expression.
    #[staticmethod]
    #[pyo3(signature = (variable, collection, key, condition, filter=None))]
    fn group_by(
        variable: &str,
        collection: &Expression,
        key: &Expression,
        condition: &Expression,
        filter: Option<&Expression>,
    ) -> Self {
        Self {
            inner: RustExpression::GroupBy {
                variable: variable.to_string(),
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
    fn normalize(&self) -> NormalizedExpression {
        NormalizedExpression {
            inner: self.inner.normalize(),
        }
    }

    /// Check if this expression is semantically equivalent to another.
    fn is_equivalent(&self, other: &Expression) -> bool {
        self.inner.is_equivalent(&other.inner)
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Expression({})", self.inner)
    }

    fn __eq__(&self, other: &Expression) -> bool {
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
#[pyclass]
#[derive(Clone)]
pub struct NormalizedExpression {
    inner: RustNormalizedExpression,
}

#[pymethods]
impl NormalizedExpression {
    /// Get the stable hash value for this normalized expression.
    fn stable_hash(&self) -> u64 {
        self.inner.stable_hash()
    }

    /// Get the inner expression.
    fn inner_expression(&self) -> Expression {
        Expression {
            inner: self.inner.inner().clone(),
        }
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "NormalizedExpression({}, hash={:#018x})",
            self.inner,
            self.inner.stable_hash()
        )
    }

    fn __eq__(&self, other: &NormalizedExpression) -> bool {
        self.inner == other.inner
    }

    fn __hash__(&self) -> u64 {
        self.inner.stable_hash()
    }
}

// =============================================================================
// Violation and EvaluationResult
// =============================================================================

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
            "Violation(name='{}', message='{}', severity='{:?}')",
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

// =============================================================================
// Helper functions
// =============================================================================

/// Convert a Python value to a serde_json::Value.
fn python_to_json(value: &Bound<'_, pyo3::types::PyAny>) -> PyResult<serde_json::Value> {
    if value.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = value.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = value.extract::<i64>() {
        Ok(serde_json::Value::Number(i.into()))
    } else if let Ok(f) = value.extract::<f64>() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| PyValueError::new_err("Invalid float value (NaN or Infinity)"))
    } else if let Ok(s) = value.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(list) = value.downcast::<pyo3::types::PyList>() {
        let arr: Result<Vec<_>, _> = list.iter().map(|v| python_to_json(&v)).collect();
        Ok(serde_json::Value::Array(arr?))
    } else if let Ok(tuple) = value.downcast::<pyo3::types::PyTuple>() {
        let arr: Result<Vec<_>, _> = tuple.iter().map(|v| python_to_json(&v)).collect();
        Ok(serde_json::Value::Array(arr?))
    } else if let Ok(dict) = value.downcast::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, python_to_json(&v)?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        Err(PyValueError::new_err(format!(
            "Cannot convert Python value to JSON: {:?}",
            value
        )))
    }
}
