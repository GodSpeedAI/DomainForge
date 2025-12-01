use sea_core::parser::parse_expression_from_str;
use sea_core::policy::{BinaryOp, Expression};

#[test]
fn test_parse_time_literal() {
    let source = r#""2025-12-31T23:59:59Z""#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse time literal: {:?}",
        result.err()
    );

    match result.unwrap() {
        Expression::TimeLiteral(timestamp) => {
            assert_eq!(timestamp, "2025-12-31T23:59:59Z");
        }
        other => panic!("Expected TimeLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_time_literal_with_offset() {
    let source = r#""2025-12-31T23:59:59+05:00""#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse time literal with offset: {:?}",
        result.err()
    );

    match result.unwrap() {
        Expression::TimeLiteral(timestamp) => {
            assert_eq!(timestamp, "2025-12-31T23:59:59+05:00");
        }
        other => panic!("Expected TimeLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_interval_literal() {
    let source = r#"interval("09:00", "17:00")"#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse interval literal: {:?}",
        result.err()
    );

    match result.unwrap() {
        Expression::IntervalLiteral { start, end } => {
            assert_eq!(start, "09:00");
            assert_eq!(end, "17:00");
        }
        other => panic!("Expected IntervalLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_before_operator() {
    let source = r#"f.created_at before "2025-12-31T23:59:59Z""#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse before operator: {:?}",
        result.err()
    );

    match result.unwrap() {
        Expression::Binary { op, left, right } => {
            assert_eq!(op, BinaryOp::Before);

            // Verify left side is member access
            match *left {
                Expression::MemberAccess { object, member } => {
                    assert_eq!(object, "f");
                    assert_eq!(member, "created_at");
                }
                other => panic!("Expected MemberAccess on left, got {:?}", other),
            }

            // Verify right side is time literal
            match *right {
                Expression::TimeLiteral(timestamp) => {
                    assert_eq!(timestamp, "2025-12-31T23:59:59Z");
                }
                other => panic!("Expected TimeLiteral on right, got {:?}", other),
            }
        }
        other => panic!("Expected Binary expression, got {:?}", other),
    }
}

#[test]
fn test_parse_after_operator() {
    let source = r#"f.resolved_at after "2025-01-01T00:00:00Z""#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse after operator: {:?}",
        result.err()
    );

    match result.unwrap() {
        Expression::Binary { op, left, right } => {
            assert_eq!(op, BinaryOp::After);

            // Verify left side is member access
            match *left {
                Expression::MemberAccess { object, member } => {
                    assert_eq!(object, "f");
                    assert_eq!(member, "resolved_at");
                }
                other => panic!("Expected MemberAccess on left, got {:?}", other),
            }

            // Verify right side is time literal
            match *right {
                Expression::TimeLiteral(timestamp) => {
                    assert_eq!(timestamp, "2025-01-01T00:00:00Z");
                }
                other => panic!("Expected TimeLiteral on right, got {:?}", other),
            }
        }
        other => panic!("Expected Binary expression, got {:?}", other),
    }
}

#[test]
fn test_parse_during_operator() {
    let source = r#"f.timestamp during interval("09:00", "17:00")"#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse during operator: {:?}",
        result.err()
    );

    match result.unwrap() {
        Expression::Binary { op, left, right } => {
            assert_eq!(op, BinaryOp::During);

            // Verify left side is member access
            match *left {
                Expression::MemberAccess { object, member } => {
                    assert_eq!(object, "f");
                    assert_eq!(member, "timestamp");
                }
                other => panic!("Expected MemberAccess on left, got {:?}", other),
            }

            // Verify right side is interval literal
            match *right {
                Expression::IntervalLiteral { start, end } => {
                    assert_eq!(start, "09:00");
                    assert_eq!(end, "17:00");
                }
                other => panic!("Expected IntervalLiteral on right, got {:?}", other),
            }
        }
        other => panic!("Expected Binary expression, got {:?}", other),
    }
}

#[test]
fn test_temporal_expression_display() {
    let time_expr = Expression::TimeLiteral("2025-12-31T23:59:59Z".to_string());
    assert_eq!(format!("{}", time_expr), r#""2025-12-31T23:59:59Z""#);

    let interval_expr = Expression::IntervalLiteral {
        start: "09:00".to_string(),
        end: "17:00".to_string(),
    };
    assert_eq!(
        format!("{}", interval_expr),
        r#"interval("09:00", "17:00")"#
    );
}

#[test]
fn test_temporal_operators_display() {
    assert_eq!(format!("{}", BinaryOp::Before), "BEFORE");
    assert_eq!(format!("{}", BinaryOp::After), "AFTER");
    assert_eq!(format!("{}", BinaryOp::During), "DURING");
}

#[test]
fn test_complex_temporal_policy() {
    // Test a more complex policy with temporal logic
    let source = r#"(f.created_at before "2025-12-31T23:59:59Z") and (f.timestamp during interval("09:00", "17:00"))"#;
    let result = parse_expression_from_str(source);

    assert!(
        result.is_ok(),
        "Failed to parse complex temporal policy: {:?}",
        result.err()
    );

    // Just verify it parses successfully
    match result.unwrap() {
        Expression::Binary {
            op: BinaryOp::And, ..
        } => {
            // Success - we have an AND expression with temporal sub-expressions
        }
        other => panic!("Expected AND expression, got {:?}", other),
    }
}
