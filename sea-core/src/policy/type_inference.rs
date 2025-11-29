use crate::units::Dimension;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    Quantity { dimension: Dimension },
    Numeric,
    String,
    Boolean,
    Time,
    Interval,
    Collection(Box<ExprType>),
}

#[derive(Error, Debug, PartialEq)]
pub enum TypeError {
    #[error("Unit mismatch: expected {expected}, found {found}. Hint: {hint}")]
    UnitMismatch {
        expected: Dimension,
        found: Dimension,
        hint: String,
    },
    #[error("Mixed quantity and numeric types. Hint: {hint}")]
    MixedQuantityNumeric { hint: String },
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
}

pub fn check_comparison(left: &ExprType, right: &ExprType) -> Result<(), TypeError> {
    match (left, right) {
        (ExprType::Quantity { dimension: d1 }, ExprType::Quantity { dimension: d2 }) => {
            if d1 != d2 {
                return Err(TypeError::UnitMismatch {
                    expected: d1.clone(),
                    found: d2.clone(),
                    hint: format!("Convert using 'as \"{}\"'", d1), // Simplified hint
                });
            }
        }
        (ExprType::Quantity { .. }, ExprType::Numeric) => {
            return Err(TypeError::MixedQuantityNumeric {
                hint: "Quantities require explicit units. Did you mean 'value as \"unit\"'?"
                    .to_string(),
            });
        }
        (ExprType::Numeric, ExprType::Quantity { .. }) => {
            return Err(TypeError::MixedQuantityNumeric {
                hint: "Quantities require explicit units. Did you mean 'value as \"unit\"'?"
                    .to_string(),
            });
        }
        (ExprType::Numeric, ExprType::Numeric) => {}
        (ExprType::String, ExprType::String) => {}
        (ExprType::Boolean, ExprType::Boolean) => {}
        (ExprType::Time, ExprType::Time) => {} // Time before/after Time
        (ExprType::Time, ExprType::Interval) => {} // Time during Interval
        (ExprType::Interval, ExprType::Time) => {} // Interval contains Time (reverse)
        (ExprType::Interval, ExprType::Interval) => {} // Interval comparisons
        (t1, t2) => {
            return Err(TypeError::TypeMismatch {
                expected: format!("{:?}", t1),
                found: format!("{:?}", t2),
            });
        }
    }
    Ok(())
}
