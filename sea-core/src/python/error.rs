/// Python-native error types for sea_dsl
/// 
/// This module provides Python exception conversion that feels native to Python users.
/// Users should never see Rust error types when using the Python bindings.

use crate::validation_error::ValidationError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

/// Convert ValidationError to appropriate Python exception with custom attributes
pub fn to_python_exception(err: ValidationError) -> PyErr {
    let code = err.error_code().as_str().to_string();

    Python::with_gil(|py| {
        match err {
            ValidationError::SyntaxError {
                message,
                line,
                column,
                ..
            } => {
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                // Set custom attributes
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("line", line);
                let _ = exc_obj.setattr("column", column);
                let _ = exc_obj.setattr("error_type", "SyntaxError");
                
                exc
            }
            ValidationError::TypeError {
                message,
                expected_type,
                found_type,
                suggestion,
                ..
            } => {
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "TypeError");
                if let Some(exp) = expected_type {
                    let _ = exc_obj.setattr("expected", exp);
                }
                if let Some(fnd) = found_type {
                    let _ = exc_obj.setattr("found", fnd);
                }
                if let Some(sug) = suggestion {
                    let _ = exc_obj.setattr("suggestion", sug);
                }
                
                exc
            }
            ValidationError::UnitError {
                expected,
                found,
                suggestion,
                ..
            } => {
                let message = format!(
                    "Unit mismatch: expected {:?}, found {:?}",
                    expected, found
                );
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "UnitError");
                let _ = exc_obj.setattr("expected_dimension", format!("{:?}", expected));
                let _ = exc_obj.setattr("found_dimension", format!("{:?}", found));
                if let Some(sug) = suggestion {
                    let _ = exc_obj.setattr("suggestion", sug);
                }
                
                exc
            }
            ValidationError::UndefinedReference {
                reference_type,
                name,
                suggestion,
                ..
            } => {
                let message = format!("Undefined {}: '{}'", reference_type, name);
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "ReferenceError");
                let _ = exc_obj.setattr("reference_type", reference_type);
                let _ = exc_obj.setattr("name", name);
                if let Some(sug) = suggestion {
                    let _ = exc_obj.setattr("suggestion", sug);
                }
                
                exc
            }
            ValidationError::ScopeError {
                variable,
                suggestion,
                ..
            } => {
                let message = format!("Variable '{}' not in scope", variable);
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "ScopeError");
                let _ = exc_obj.setattr("variable", variable);
                if let Some(sug) = suggestion {
                    let _ = exc_obj.setattr("suggestion", sug);
                }
                
                exc
            }
            ValidationError::DuplicateDeclaration {
                name,
                first_location,
                second_location,
            } => {
                let message = format!(
                    "Duplicate declaration of '{}': first at {}, duplicate at {}",
                    name, first_location, second_location
                );
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "DuplicateDeclaration");
                let _ = exc_obj.setattr("name", name);
                let _ = exc_obj.setattr("first_location", first_location);
                let _ = exc_obj.setattr("second_location", second_location);
                
                exc
            }
            ValidationError::DeterminismError { message, hint } => {
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "DeterminismError");
                let _ = exc_obj.setattr("hint", hint);
                
                exc
            }
            ValidationError::InvalidExpression {
                message,
                suggestion,
                ..
            } => {
                let exc = PyException::new_err(message);
                let exc_obj = exc.value(py);
                
                let _ = exc_obj.setattr("code", code);
                let _ = exc_obj.setattr("error_type", "InvalidExpression");
                if let Some(sug) = suggestion {
                    let _ = exc_obj.setattr("suggestion", sug);
                }
                
                exc
            }
        }
    })
}
