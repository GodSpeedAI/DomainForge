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

    let err = ValidationError::undefined_entity_with_candidates("Warehous", "line 10", &candidates);

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

    let err = ValidationError::undefined_resource_with_candidates("Stel", "line 15", &candidates);

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

    let err = ValidationError::undefined_entity_with_candidates("XYZ", "line 10", &candidates);

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
    assert_eq!(
        ErrorCode::E001_UndefinedEntity.description(),
        "Undefined entity"
    );
    assert_eq!(ErrorCode::E003_UnitMismatch.description(), "Unit mismatch");
    assert_eq!(
        ErrorCode::E402_DeterminismViolation.description(),
        "Determinism violation"
    );
}

// Formatter tests
use sea_core::error::{DiagnosticFormatter, HumanFormatter, JsonFormatter, LspFormatter};

#[test]
fn test_json_formatter_output() {
    let error = ValidationError::syntax_error_with_range("unexpected token", 10, 5, 10, 15);
    let formatter = JsonFormatter;
    let output = formatter.format(&error, None);

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
    assert_eq!(parsed["code"], "E005");
    assert_eq!(parsed["severity"], "error");
    assert_eq!(parsed["range"]["start"]["line"], 10);
    assert_eq!(parsed["range"]["start"]["column"], 5);
}

#[test]
fn test_json_formatter_with_fuzzy_suggestion() {
    let candidates = vec!["Warehouse".to_string(), "Factory".to_string()];
    let error =
        ValidationError::undefined_entity_with_candidates("Warehous", "line 10", &candidates);
    let formatter = JsonFormatter;
    let output = formatter.format(&error, None);

    let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
    assert_eq!(parsed["code"], "E001");
    assert!(parsed["hint"].as_str().unwrap().contains("Warehouse"));
}

#[test]
fn test_human_formatter_basic() {
    let error = ValidationError::undefined_entity("TestEntity", "line 5");
    let formatter = HumanFormatter::new(false, false);
    let output = formatter.format(&error, None);

    assert!(output.contains("error[E001]"));
    assert!(output.contains("TestEntity"));
}

#[test]
fn test_human_formatter_with_source() {
    let source = r#"Entity "Warehouse" in logistics
Entity "Factory" in manufacturing
Flow "Steel" from "Warehous" to "Factory" quantity 100
"#;

    let error = ValidationError::syntax_error_with_range("typo in entity name", 3, 19, 3, 28);
    let formatter = HumanFormatter::new(false, true);
    let output = formatter.format(&error, Some(source));

    // Should include source snippet
    assert!(output.contains("Warehous"));
    assert!(output.contains("^^^"));
}

#[test]
fn test_lsp_formatter_output() {
    let error = ValidationError::syntax_error_with_range("test", 10, 5, 10, 15);
    let formatter = LspFormatter;
    let output = formatter.format(&error, None);

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
    assert_eq!(parsed["severity"], 1); // Error
    assert_eq!(parsed["code"], "E005");
    assert_eq!(parsed["source"], "sea-dsl");
    // LSP uses 0-indexed lines
    assert_eq!(parsed["range"]["start"]["line"], 9);
}

#[test]
fn test_format_multiple_errors_json() {
    use sea_core::error::diagnostics::format_errors_json;

    let errors = vec![
        ValidationError::syntax_error("error 1", 1, 1),
        ValidationError::undefined_entity("Test", "line 5"),
        ValidationError::unit_mismatch(
            sea_core::units::Dimension::Mass,
            sea_core::units::Dimension::Currency,
            "line 10",
        ),
    ];

    let output = format_errors_json(&errors);
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");

    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 3);
    assert_eq!(parsed[0]["code"], "E005");
    assert_eq!(parsed[1]["code"], "E001");
    assert_eq!(parsed[2]["code"], "E003");
}
