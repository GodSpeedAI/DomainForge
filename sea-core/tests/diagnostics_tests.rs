// Tests for enhanced error diagnostics with fuzzy matching
use sea_core::{ErrorCode, ValidationError};

#[test]
fn test_error_code_assignment() {
    let syntax_err = ValidationError::syntax_error("test", 1, 1);
    assert_eq!(syntax_err.error_code(), ErrorCode::E005_SyntaxError);

    let entity_err = ValidationError::undefined_entity("Warehouse", "line 10");
    assert_eq!(entity_err.error_code(), ErrorCode::E001_UndefinedEntity);

    let resource_err = ValidationError::undefined_resource("Steel", "line 15");
    assert_eq!(resource_err.error_code(), ErrorCode::E002_UndefinedResource);
}

#[test]
fn test_source_range_from_syntax_error() {
    let err = ValidationError::syntax_error_with_range("test", 10, 5, 10, 15);
    let range = err.range().expect("Should have range");
    
    assert_eq!(range.start.line, 10);
    assert_eq!(range.start.column, 5);
    assert_eq!(range.end.line, 10);
    assert_eq!(range.end.column, 15);
}

#[test]
fn test_location_string() {
    let err = ValidationError::syntax_error("test", 10, 5);
    assert_eq!(err.location_string(), Some("10:5".to_string()));

    let err2 = ValidationError::undefined_entity("Warehouse", "line 20");
    assert_eq!(err2.location_string(), Some("line 20".to_string()));
}

#[test]
fn test_fuzzy_matching_undefined_entity() {
    let candidates = vec![
        "Warehouse".to_string(),
        "Factory".to_string(),
        "Supplier".to_string(),
    ];

    let err = ValidationError::undefined_entity_with_candidates(
        "Warehous",
        "line 10",
        &candidates,
    );

    let suggestion = match err {
        ValidationError::UndefinedReference { suggestion, .. } => suggestion,
        _ => panic!("Expected UndefinedReference"),
    };

    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("Warehouse"));
}

#[test]
fn test_fuzzy_matching_undefined_resource() {
    let candidates = vec![
        "Steel".to_string(),
        "Iron".to_string(),
        "Copper".to_string(),
    ];

    let err = ValidationError::undefined_resource_with_candidates(
        "Stel",
        "line 15",
        &candidates,
    );

    let suggestion = match err {
        ValidationError::UndefinedReference { suggestion, .. } => suggestion,
        _ => panic!("Expected UndefinedReference"),
    };

    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("Steel"));
}

#[test]
fn test_fuzzy_matching_no_candidates() {
    let candidates = vec!["Warehouse".to_string(), "Factory".to_string()];

    let err = ValidationError::undefined_entity_with_candidates(
        "XYZ",
        "line 10",
        &candidates,
    );

    let suggestion = match err {
        ValidationError::UndefinedReference { suggestion, .. } => suggestion,
        _ => panic!("Expected UndefinedReference"),
    };

    // Should fall back to generic suggestion
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("Entity \"XYZ\""));
}

#[test]
fn test_error_code_display() {
    assert_eq!(ErrorCode::E001_UndefinedEntity.as_str(), "E001");
    assert_eq!(ErrorCode::E005_SyntaxError.as_str(), "E005");
    assert_eq!(ErrorCode::E300_VariableNotInScope.as_str(), "E300");
}

#[test]
fn test_error_code_description() {
    assert_eq!(ErrorCode::E001_UndefinedEntity.description(), "Undefined entity");
    assert_eq!(ErrorCode::E003_UnitMismatch.description(), "Unit mismatch");
    assert_eq!(ErrorCode::E402_DeterminismViolation.description(), "Determinism violation");
}
