/// TypeScript-native error handling for sea_dsl
///
/// This module provides error conversion that creates native JavaScript/TypeScript
/// Error objects with typed properties. TypeScript users should never see Rust errors.
use crate::validation_error::ValidationError;
use napi::bindgen_prelude::*;

/// Convert ValidationError to napi Error with custom properties
pub fn to_napi_error(err: ValidationError) -> Error {
    let code = err.error_code().as_str().to_string();
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
    let mut error = Error::from_reason(message);

    // Add error-specific properties based on variant
    match err {
        ValidationError::SyntaxError {
            line,
            column,
            end_line,
            end_column,
            ..
        } => {
            // Store as JSON string since napi Error doesn't support arbitrary properties directly
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "line": line,
                "column": column,
                "endLine": end_line,
                "endColumn": end_column,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::TypeError {
            expected_type,
            found_type,
            suggestion,
            ..
        } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "expected": expected_type,
                "found": found_type,
                "suggestion": suggestion,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::UnitError {
            expected,
            found,
            suggestion,
            ..
        } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "expectedDimension": format!("{}", expected),
                "foundDimension": format!("{}", found),
                "suggestion": suggestion,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::UndefinedReference {
            reference_type,
            name,
            suggestion,
            ..
        } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "referenceType": reference_type.to_string(),
                "name": name,
                "suggestion": suggestion,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::ScopeError {
            variable,
            available_in,
            suggestion,
            ..
        } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "variable": variable,
                "availableIn": available_in,
                "suggestion": suggestion,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::DuplicateDeclaration {
            name,
            first_location,
            second_location,
        } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "name": name,
                "firstLocation": first_location,
                "secondLocation": second_location,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::DeterminismError { hint, .. } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "hint": hint,
            });
            attach_metadata(&mut error, metadata);
        }
        ValidationError::InvalidExpression { suggestion, .. } => {
            let metadata = serde_json::json!({
                "code": code,
                "errorType": error_type,
                "suggestion": suggestion,
            });
            attach_metadata(&mut error, metadata);
        }
    }

    error
}

fn attach_metadata(error: &mut Error, metadata: serde_json::Value) {
    error.reason = format!(
        "{}\n__metadata__: {}",
        error.reason,
        serde_json::to_string(&metadata).unwrap_or_else(|e| format!("serialization_error: {}", e))
    );
}
