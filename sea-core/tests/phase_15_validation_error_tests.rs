use sea_core::units::Dimension;
use sea_core::validation_error::ValidationError;

#[test]
fn test_syntax_error_with_range() {
    let error = ValidationError::syntax_error_with_range("Unexpected token", 10, 5, 10, 12);

    let message = format!("{}", error);
    assert!(message.contains("10:5 to 10:12"));
    assert!(message.contains("Unexpected token"));
}

#[test]
fn test_unit_error_with_suggestion() {
    let error = ValidationError::unit_error(Dimension::Mass, Dimension::Volume, "Flow line 42")
        .with_suggestion("Convert volume to mass using density, or use compatible units");

    let message = format!("{}", error);
    assert!(message.contains("Unit error"));
    assert!(message.contains("Mass"));
    assert!(message.contains("Volume"));
    assert!(message.contains("Suggestion: Convert volume to mass"));
}

#[test]
fn test_scope_error() {
    let error = ValidationError::scope_error(
        "quantity",
        vec!["Flow".to_string(), "Instance".to_string()],
        "Policy line 15",
    )
    .with_suggestion("Use 'flow.quantity' to access the quantity field");

    let message = format!("{}", error);
    assert!(message.contains("Scope error"));
    assert!(message.contains("quantity"));
    assert!(message.contains("Available in: Flow, Instance"));
    assert!(message.contains("Suggestion: Use 'flow.quantity'"));
}

#[test]
fn test_determinism_error() {
    let error = ValidationError::determinism_error(
        "Random function used in policy evaluation",
        "Remove random() calls and use deterministic expressions",
    );

    let message = format!("{}", error);
    assert!(message.contains("Determinism error"));
    assert!(message.contains("Random function"));
    assert!(message.contains("Hint: Remove random()"));
}

#[test]
fn test_undefined_reference_with_suggestion() {
    let error = ValidationError::undefined_reference("Entity", "Supplier", "Flow line 8")
        .with_suggestion("Did you mean 'SupplierCorp'?");

    let message = format!("{}", error);
    assert!(message.contains("Undefined Entity"));
    assert!(message.contains("Supplier"));
    assert!(message.contains("Flow line 8"));
    assert!(message.contains("Suggestion: Did you mean"));
}

#[test]
fn test_duplicate_declaration() {
    let error = ValidationError::duplicate_declaration("MyEntity", "line 10", "line 25");

    let message = format!("{}", error);
    assert!(message.contains("Duplicate declaration"));
    assert!(message.contains("MyEntity"));
    assert!(message.contains("first at line 10"));
    assert!(message.contains("duplicate at line 25"));
}

#[test]
fn test_type_error_with_types() {
    let error = ValidationError::type_error(
        "Cannot compare string with number",
        "Policy expression line 5",
    )
    .with_types("Number", "String")
    .with_suggestion("Cast the string to a number or compare with another string");

    let message = format!("{}", error);
    assert!(message.contains("Type error"));
    assert!(message.contains("expected Number, found String"));
    assert!(message.contains("Suggestion: Cast the string"));
}

#[test]
fn test_invalid_expression_with_suggestion() {
    let error = ValidationError::invalid_expression("Division by zero", "Flow calculation line 12")
        .with_suggestion("Ensure denominator is never zero");

    let message = format!("{}", error);
    assert!(message.contains("Invalid expression"));
    assert!(message.contains("Division by zero"));
    assert!(message.contains("Suggestion: Ensure denominator"));
}

#[test]
fn test_unit_error_dimensions() {
    let error =
        ValidationError::unit_error(Dimension::Length, Dimension::Mass, "Resource conversion");

    let message = format!("{}", error);
    assert!(message.contains("Length"));
    assert!(message.contains("Mass"));
}

#[test]
fn test_scope_error_empty_available() {
    let error = ValidationError::scope_error("unknown_var", vec![], "Policy line 20");

    let message = format!("{}", error);
    assert!(message.contains("Scope error"));
    assert!(message.contains("unknown_var"));
    assert!(!message.contains("Available in:"));
}

#[test]
fn test_error_display_formatting() {
    let errors = vec![
        ValidationError::syntax_error("Missing semicolon", 5, 10),
        ValidationError::type_error("Type mismatch", "line 3"),
        ValidationError::determinism_error("Non-deterministic", "Use fixed values"),
    ];

    for error in errors {
        let message = format!("{}", error);
        assert!(!message.is_empty());
        assert!(message.len() > 10);
    }
}
