use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowSpec {
    pub duration: u64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(serde_json::Value),

    QuantityLiteral {
        value: Decimal,
        unit: String,
    },

    TimeLiteral(String), // ISO 8601 timestamp

    IntervalLiteral {
        start: String, // Time string (e.g., "09:00")
        end: String,   // Time string (e.g., "17:00")
    },

    Variable(String),

    GroupBy {
        variable: String,
        collection: Box<Expression>,
        filter: Option<Box<Expression>>,
        key: Box<Expression>,
        condition: Box<Expression>,
    },

    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },

    Cast {
        operand: Box<Expression>,
        target_type: String,
    },

    Quantifier {
        quantifier: Quantifier,
        variable: String,
        collection: Box<Expression>,
        condition: Box<Expression>,
    },

    MemberAccess {
        object: String,
        member: String,
    },

    Aggregation {
        function: AggregateFunction,
        collection: Box<Expression>,
        field: Option<String>,
        filter: Option<Box<Expression>>,
    },

    AggregationComprehension {
        function: AggregateFunction,
        variable: String,
        collection: Box<Expression>,
        window: Option<WindowSpec>,
        predicate: Box<Expression>,
        projection: Box<Expression>,
        target_unit: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    // Temporal operators
    Before,
    After,
    During,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Quantifier {
    ForAll,
    Exists,
    ExistsUnique,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

impl Expression {
    /// Returns a canonical normalized form of this expression.
    ///
    /// The normalized form applies various simplification rules:
    /// - Identity elimination (`true AND x` → `x`)
    /// - Domination (`false AND x` → `false`)
    /// - Idempotence (`a AND a` → `a`)
    /// - Absorption (`a OR (a AND b)` → `a`)
    /// - Double negation (`NOT NOT x` → `x`)
    /// - Commutative sorting (operands sorted lexicographically)
    #[must_use]
    pub fn normalize(&self) -> super::NormalizedExpression {
        super::NormalizedExpression::new(self)
    }

    /// Returns true if two expressions are semantically equivalent.
    ///
    /// Two expressions are equivalent if their normalized forms are identical.
    #[must_use]
    pub fn is_equivalent(&self, other: &Expression) -> bool {
        self.normalize() == other.normalize()
    }

    pub fn literal(value: impl Into<serde_json::Value>) -> Self {
        Expression::Literal(value.into())
    }

    pub fn variable(name: &str) -> Self {
        Expression::Variable(name.to_string())
    }

    pub fn binary(op: BinaryOp, left: Expression, right: Expression) -> Self {
        Expression::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn unary(op: UnaryOp, operand: Expression) -> Self {
        Expression::Unary {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn quantifier(
        q: Quantifier,
        var: &str,
        collection: Expression,
        condition: Expression,
    ) -> Self {
        Expression::Quantifier {
            quantifier: q,
            variable: var.to_string(),
            collection: Box::new(collection),
            condition: Box::new(condition),
        }
    }

    pub fn cast(operand: Expression, target_type: impl Into<String>) -> Self {
        Expression::Cast {
            operand: Box::new(operand),
            target_type: target_type.into(),
        }
    }

    pub fn comparison(
        var: &str,
        op: &str,
        value: impl Into<serde_json::Value>,
    ) -> Result<Self, String> {
        let op = match op {
            ">" => BinaryOp::GreaterThan,
            "<" => BinaryOp::LessThan,
            ">=" => BinaryOp::GreaterThanOrEqual,
            "<=" => BinaryOp::LessThanOrEqual,
            "==" => BinaryOp::Equal,
            "!=" => BinaryOp::NotEqual,
            _ => return Err(format!("Unknown operator: {}", op)),
        };

        Ok(Expression::binary(
            op,
            Expression::variable(var),
            Expression::literal(value),
        ))
    }

    pub fn aggregation(
        function: AggregateFunction,
        collection: Expression,
        field: Option<impl Into<String>>,
        filter: Option<Expression>,
    ) -> Self {
        Expression::Aggregation {
            function,
            collection: Box::new(collection),
            field: field.map(|f| f.into()),
            filter: filter.map(Box::new),
        }
    }

    pub fn member_access(object: &str, member: &str) -> Self {
        Expression::MemberAccess {
            object: object.to_string(),
            member: member.to_string(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Literal(v) => write!(f, "{}", v),
            Expression::QuantityLiteral { value, unit } => {
                write!(f, "{} {}", value, unit)
            }
            Expression::TimeLiteral(timestamp) => write!(f, "\"{}\"", timestamp),
            Expression::IntervalLiteral { start, end } => {
                write!(f, "interval(\"{}\", \"{}\")", start, end)
            }
            Expression::Variable(n) => write!(f, "{}", n),
            Expression::GroupBy {
                variable,
                collection,
                filter,
                key,
                condition,
            } => {
                write!(f, "group_by({} in {}", variable, collection)?;
                if let Some(flt) = filter {
                    write!(f, " WHERE {}", flt)?;
                }
                write!(f, ": {}) {{ {} }}", key, condition)
            }
            Expression::Binary { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expression::Unary { op, operand } => {
                write!(f, "{} {}", op, operand)
            }
            Expression::Cast {
                operand,
                target_type,
            } => {
                write!(f, "{} as \"{}\"", operand, target_type)
            }
            Expression::Quantifier {
                quantifier,
                variable,
                collection,
                condition,
            } => {
                write!(
                    f,
                    "{}({} in {}: {})",
                    quantifier, variable, collection, condition
                )
            }
            Expression::MemberAccess { object, member } => {
                write!(f, "{}.{}", object, member)
            }
            Expression::Aggregation {
                function,
                collection,
                field,
                filter,
            } => {
                write!(f, "{}({}", function, collection)?;
                if let Some(fld) = field {
                    write!(f, ".{}", fld)?;
                }
                if let Some(flt) = filter {
                    write!(f, " WHERE {}", flt)?;
                }
                write!(f, ")")
            }
            Expression::AggregationComprehension {
                function,
                variable,
                collection,
                window,
                predicate,
                projection,
                target_unit,
            } => {
                write!(f, "{}({} in {}", function, variable, collection)?;
                if let Some(w) = window {
                    write!(f, " OVER LAST {} \"{}\"", w.duration, w.unit)?;
                }
                match predicate.as_ref() {
                    Expression::Literal(serde_json::Value::Bool(true)) => {
                        write!(f, ": {}", projection)?;
                    }
                    _ => {
                        write!(f, " WHERE {}: {}", predicate, projection)?;
                    }
                }
                if let Some(unit) = target_unit {
                    write!(f, " AS \"{}\"", unit)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryOp::And => write!(f, "AND"),
            BinaryOp::Or => write!(f, "OR"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::GreaterThan => write!(f, ">"),
            BinaryOp::LessThan => write!(f, "<"),
            BinaryOp::GreaterThanOrEqual => write!(f, ">="),
            BinaryOp::LessThanOrEqual => write!(f, "<="),
            BinaryOp::Plus => write!(f, "+"),
            BinaryOp::Minus => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Contains => write!(f, "CONTAINS"),
            BinaryOp::StartsWith => write!(f, "STARTS_WITH"),
            BinaryOp::EndsWith => write!(f, "ENDS_WITH"),
            BinaryOp::Matches => write!(f, "MATCHES"),
            BinaryOp::HasRole => write!(f, "HAS_ROLE"),
            BinaryOp::Before => write!(f, "BEFORE"),
            BinaryOp::After => write!(f, "AFTER"),
            BinaryOp::During => write!(f, "DURING"),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "NOT"),
            UnaryOp::Negate => write!(f, "-"),
        }
    }
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quantifier::ForAll => write!(f, "ForAll"),
            Quantifier::Exists => write!(f, "Exists"),
            Quantifier::ExistsUnique => write!(f, "ExistsUnique"),
        }
    }
}

impl fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AggregateFunction::Count => write!(f, "COUNT"),
            AggregateFunction::Sum => write!(f, "SUM"),
            AggregateFunction::Min => write!(f, "MIN"),
            AggregateFunction::Max => write!(f, "MAX"),
            AggregateFunction::Avg => write!(f, "AVG"),
        }
    }
}
