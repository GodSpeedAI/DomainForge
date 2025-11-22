/// WASM-native error handling for sea_dsl
///
/// This module provides error conversion that creates error messages with embedded
/// metadata for WASM bindings. JavaScript users get structured error information.
use crate::validation_error::ValidationError;
use wasm_bindgen::prelude::*;

/// Convert ValidationError to JsValue error string with embedded metadata
///
/// The error message includes JSON metadata that JavaScript can parse to get
/// structured error information including error codes, types, and suggestions.
pub fn to_js_error(err: ValidationError) -> JsValue {
    let code = err.error_code().as_str();
    let error_type = match &err {
        ValidationError::SyntaxError { .. } => "SyntaxError",
        ValidationError::TypeError { .. } => "TypeError",
        ValidationError::UnitError { .. } => "UnitError",
        ValidationError::UndefinedReference { .. } => "ReferenceError",
        ValidationError::ScopeError { .. } => "ScopeError",
        ValidationError::DuplicateDeclaration { .. } => "DuplicateDeclaration",
        ValidationError::DeterminismError { .. } => "DeterminismError",
        ValidationError::InvalidExpression { .. } => "InvalidExpression",
    };

    let message = err.to_string();

    // Create metadata JSON based on error variant
    let metadata = match err {
        ValidationError::SyntaxError {
            line,
            column,
            end_line,
            end_column,
            ..
        } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "line": line,
                "column": column,
                "endLine": end_line,
                "endColumn": end_column,
            })
        }
        ValidationError::TypeError {
            expected_type,
            found_type,
            suggestion,
            ..
        } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "expected": expected_type,
                "found": found_type,
                "suggestion": suggestion,
            })
        }
        ValidationError::UnitError {
            expected,
            found,
            suggestion,
            ..
        } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "expectedDimension": format!("{}", expected),
                "foundDimension": format!("{}", found),
                "suggestion": suggestion,
            })
        }
        ValidationError::UndefinedReference {
            reference_type,
            name,
            suggestion,
            ..
        } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "referenceType": reference_type,
                "name": name,
                "suggestion": suggestion,
            })
        }
        ValidationError::ScopeError {
            variable,
            available_in,
            suggestion,
            ..
        } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "variable": variable,
                "availableIn": available_in,
                "suggestion": suggestion,
            })
        }
        ValidationError::DuplicateDeclaration {
            name,
            first_location,
            second_location,
        } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "name": name,
                "firstLocation": first_location,
                "secondLocation": second_location,
            })
        }
        ValidationError::DeterminismError { hint, .. } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "hint": hint,
            })
        }
        ValidationError::InvalidExpression { suggestion, .. } => {
            serde_json::json!({
                "code": code,
                "errorType": error_type,
                "suggestion": suggestion,
            })
        }
    };

    // Create error message with embedded metadata
    // Create error object with message and metadata
    let error_object = serde_json::json!({
        "message": message,
        "metadata": metadata
    });

    // Convert to JsValue using serde-wasm-bindgen
    match serde_wasm_bindgen::to_value(&error_object) {
        Ok(val) => val,
        Err(_) => {
            // Fallback to string if serialization fails
            JsValue::from_str(&message)
        }
    }
}
