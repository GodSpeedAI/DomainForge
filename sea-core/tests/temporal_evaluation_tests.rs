use sea_core::parser::parse_expression_from_str;
use sea_core::policy::Policy;
use sea_core::Graph;

#[test]
fn test_temporal_before_evaluation_true() {
    let graph = Graph::new();

    // Create a policy that checks if one timestamp is before another
    let expr_str = r#""2025-01-01T00:00:00Z" before "2025-12-31T23:59:59Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_before", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    assert!(
        result.is_satisfied,
        "Expected policy to be satisfied (earlier date before later date)"
    );
    assert_eq!(result.is_satisfied_tristate, Some(true));
}

#[test]
fn test_temporal_before_evaluation_false() {
    let graph = Graph::new();

    // Later date is NOT before earlier date
    let expr_str = r#""2025-12-31T23:59:59Z" before "2025-01-01T00:00:00Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_before_false", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    assert!(
        !result.is_satisfied,
        "Expected policy to fail (later date not before earlier date)"
    );
    assert_eq!(result.is_satisfied_tristate, Some(false));
}

#[test]
fn test_temporal_after_evaluation_true() {
    let graph = Graph::new();

    // Later date is after earlier date
    let expr_str = r#""2025-12-31T23:59:59Z" after "2025-01-01T00:00:00Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_after", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    assert!(
        result.is_satisfied,
        "Expected policy to be satisfied (later date after earlier date)"
    );
    assert_eq!(result.is_satisfied_tristate, Some(true));
}

#[test]
fn test_temporal_after_evaluation_false() {
    let graph = Graph::new();

    // Earlier date is NOT after later date
    let expr_str = r#""2025-01-01T00:00:00Z" after "2025-12-31T23:59:59Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_after_false", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    assert!(
        !result.is_satisfied,
        "Expected policy to fail (earlier date not after later date)"
    );
    assert_eq!(result.is_satisfied_tristate, Some(false));
}

#[test]
fn test_temporal_timezone_aware_comparison() {
    let graph = Graph::new();

    // Test that timezone-aware comparison works correctly
    // 2025-01-01 00:00:00 UTC is BEFORE 2025-01-01 00:00:00 +05:00 (which is earlier in UTC)
    let expr_str = r#""2025-01-01T00:00:00+05:00" before "2025-01-01T00:00:00Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_timezone", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    // +05:00 means 5 hours ahead of UTC, so 2025-01-01T00:00:00+05:00 is actually 2024-12-31T19:00:00Z
    // which is BEFORE 2025-01-01T00:00:00Z
    assert!(
        result.is_satisfied,
        "Expected timezone-aware comparison to work correctly"
    );
}

#[test]
fn test_temporal_invalid_timestamp_error() {
    let graph = Graph::new();

    // Invalid timestamp should produce Null (unsatisfied) in 3VL
    let expr_str = r#""not-a-timestamp" before "2025-01-01T00:00:00Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_invalid", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    assert!(
        !result.is_satisfied,
        "Expected policy to be unsatisfied for invalid timestamp"
    );
    assert!(
        result.is_satisfied_tristate.is_none(),
        "Expected Null result for invalid timestamp"
    );
}

#[test]
fn test_temporal_during_not_implemented() {
    let graph = Graph::new();

    // 'during' operator should return a clear error
    let expr_str = r#""2025-06-15T12:00:00Z" during interval("09:00", "17:00")"#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_during", expr);
    let result = policy.evaluate(&graph);

    assert!(result.is_err(), "Expected error for 'during' operator");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("during"),
        "Error should mention 'during' operator: {}",
        err_msg
    );
    assert!(
        err_msg.contains("not yet implemented"),
        "Error should indicate not implemented: {}",
        err_msg
    );
}

#[test]
fn test_temporal_same_timestamp_not_before() {
    let graph = Graph::new();

    // Same timestamp should NOT be before itself
    let expr_str = r#""2025-01-01T00:00:00Z" before "2025-01-01T00:00:00Z""#;
    let expr = parse_expression_from_str(expr_str).expect("Failed to parse expression");

    let policy = Policy::new("test_same", expr);
    let result = policy.evaluate(&graph).expect("Failed to evaluate policy");

    assert!(
        !result.is_satisfied,
        "Same timestamp should not be before itself"
    );
    assert_eq!(result.is_satisfied_tristate, Some(false));
}
