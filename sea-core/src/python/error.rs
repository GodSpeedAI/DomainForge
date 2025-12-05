/// Python-native error types for sea_dsl
///
/// This module provides Python exception conversion that feels native to Python users.
/// Users should never see Rust error types when using the Python bindings.
use crate::validation_error::ValidationError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

/// Convert ValidationError to appropriate Python exception with custom attributes
pub fn to_python_exception(py: Python, err: ValidationError) -> PyErr {
    let code = err.error_code().as_str().to_string();

    match err {
        ValidationError::SyntaxError {
            message,
            line,
            column,
            ..
        } => {
            let exc = PyException::new_err(message);
            let exc_obj = exc.value(py);

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("line", line) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("column", column) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "SyntaxError") {
                return e;
            }

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

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "TypeError") {
                return e;
            }
            if let Some(exp) = expected_type {
                if let Err(e) = exc_obj.setattr("expected", exp) {
                    return e;
                }
            }
            if let Some(fnd) = found_type {
                if let Err(e) = exc_obj.setattr("found", fnd) {
                    return e;
                }
            }
            if let Some(sug) = suggestion {
                if let Err(e) = exc_obj.setattr("suggestion", sug) {
                    return e;
                }
            }

            exc
        }
        ValidationError::UnitError {
            expected,
            found,
            suggestion,
            ..
        } => {
            let message = format!("Unit mismatch: expected {:?}, found {:?}", expected, found);
            let exc = PyException::new_err(message);
            let exc_obj = exc.value(py);

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "UnitError") {
                return e;
            }
            if let Err(e) = exc_obj.setattr("expected_dimension", format!("{:?}", expected)) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("found_dimension", format!("{:?}", found)) {
                return e;
            }
            if let Some(sug) = suggestion {
                if let Err(e) = exc_obj.setattr("suggestion", sug) {
                    return e;
                }
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

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "ReferenceError") {
                return e;
            }
            if let Err(e) = exc_obj.setattr("reference_type", reference_type.to_string()) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("name", name) {
                return e;
            }
            if let Some(sug) = suggestion {
                if let Err(e) = exc_obj.setattr("suggestion", sug) {
                    return e;
                }
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

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "ScopeError") {
                return e;
            }
            if let Err(e) = exc_obj.setattr("variable", variable) {
                return e;
            }
            if let Some(sug) = suggestion {
                if let Err(e) = exc_obj.setattr("suggestion", sug) {
                    return e;
                }
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

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "DuplicateDeclaration") {
                return e;
            }
            if let Err(e) = exc_obj.setattr("name", name) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("first_location", first_location) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("second_location", second_location) {
                return e;
            }

            exc
        }
        ValidationError::DeterminismError { message, hint } => {
            let exc = PyException::new_err(message);
            let exc_obj = exc.value(py);

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "DeterminismError") {
                return e;
            }
            if let Err(e) = exc_obj.setattr("hint", hint) {
                return e;
            }

            exc
        }
        ValidationError::InvalidExpression {
            message,
            suggestion,
            ..
        } => {
            let exc = PyException::new_err(message);
            let exc_obj = exc.value(py);

            if let Err(e) = exc_obj.setattr("code", code) {
                return e;
            }
            if let Err(e) = exc_obj.setattr("error_type", "InvalidExpression") {
                return e;
            }
            if let Some(sug) = suggestion {
                if let Err(e) = exc_obj.setattr("suggestion", sug) {
                    return e;
                }
            }

            exc
        }
    }
}
