use sea_core::policy::type_inference::{check_comparison, ExprType};
use sea_core::units::Dimension;

#[test]
fn test_check_comparison_valid() {
    let q1 = ExprType::Quantity {
        dimension: Dimension::Mass,
    };
    let q2 = ExprType::Quantity {
        dimension: Dimension::Mass,
    };
    assert!(check_comparison(&q1, &q2).is_ok());

    let n1 = ExprType::Numeric;
    let n2 = ExprType::Numeric;
    assert!(check_comparison(&n1, &n2).is_ok());
}

#[test]
fn test_check_comparison_unit_mismatch() {
    let q1 = ExprType::Quantity {
        dimension: Dimension::Mass,
    };
    let q2 = ExprType::Quantity {
        dimension: Dimension::Length,
    };
    let result = check_comparison(&q1, &q2);
    assert!(result.is_err());
    // Check error message content if possible, or just that it matches expected variant
}

#[test]
fn test_check_comparison_mixed_types() {
    let q = ExprType::Quantity {
        dimension: Dimension::Mass,
    };
    let n = ExprType::Numeric;
    assert!(check_comparison(&q, &n).is_err());
    assert!(check_comparison(&n, &q).is_err());
}
