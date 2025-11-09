use crate::units::Dimension;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ValidationError {
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
        end_line: Option<usize>,
        end_column: Option<usize>,
    },
    TypeError {
        message: String,
        location: String,
        expected_type: Option<String>,
        found_type: Option<String>,
        suggestion: Option<String>,
    },
    UnitError {
        expected: Dimension,
        found: Dimension,
        location: String,
        suggestion: Option<String>,
    },
    ScopeError {
        variable: String,
        available_in: Vec<String>,
        location: String,
        suggestion: Option<String>,
    },
    DeterminismError {
        message: String,
        hint: String,
    },
    UndefinedReference {
        reference_type: String,
        name: String,
        location: String,
        suggestion: Option<String>,
    },
    DuplicateDeclaration {
        name: String,
        first_location: String,
        second_location: String,
    },
    InvalidExpression {
        message: String,
        location: String,
        suggestion: Option<String>,
    },
}

impl ValidationError {
    pub fn syntax_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::SyntaxError {
            message: message.into(),
            line,
            column,
            end_line: None,
            end_column: None,
        }
    }

    pub fn syntax_error_with_range(
        message: impl Into<String>,
        line: usize,
        column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Self::SyntaxError {
            message: message.into(),
            line,
            column,
            end_line: Some(end_line),
            end_column: Some(end_column),
        }
    }

    pub fn type_error(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
            location: location.into(),
            expected_type: None,
            found_type: None,
            suggestion: None,
        }
    }

    pub fn unit_error(expected: Dimension, found: Dimension, location: impl Into<String>) -> Self {
        Self::UnitError {
            expected,
            found,
            location: location.into(),
            suggestion: None,
        }
    }

    pub fn scope_error(
        variable: impl Into<String>,
        available_in: Vec<String>,
        location: impl Into<String>,
    ) -> Self {
        Self::ScopeError {
            variable: variable.into(),
            available_in,
            location: location.into(),
            suggestion: None,
        }
    }

    pub fn determinism_error(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::DeterminismError {
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub fn undefined_reference(
        reference_type: impl Into<String>,
        name: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self::UndefinedReference {
            reference_type: reference_type.into(),
            name: name.into(),
            location: location.into(),
            suggestion: None,
        }
    }

    pub fn duplicate_declaration(
        name: impl Into<String>,
        first_location: impl Into<String>,
        second_location: impl Into<String>,
    ) -> Self {
        Self::DuplicateDeclaration {
            name: name.into(),
            first_location: first_location.into(),
            second_location: second_location.into(),
        }
    }

    pub fn invalid_expression(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::InvalidExpression {
            message: message.into(),
            location: location.into(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        match &mut self {
            ValidationError::UnitError { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            ValidationError::TypeError { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            ValidationError::ScopeError { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            ValidationError::UndefinedReference { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            ValidationError::InvalidExpression { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            _ => {}
        }
        self
    }

    pub fn with_types(
        mut self,
        expected_type: impl Into<String>,
        found_type: impl Into<String>,
    ) -> Self {
        if let ValidationError::TypeError {
            expected_type: e,
            found_type: f,
            ..
        } = &mut self
        {
            *e = Some(expected_type.into());
            *f = Some(found_type.into());
        }
        self
    }

    // Phase 15A: Additional convenience constructors for common error patterns

    /// Create an error for an undefined Entity with a helpful suggestion
    pub fn undefined_entity(name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        Self::UndefinedReference {
            reference_type: "Entity".to_string(),
            name: name.clone(),
            location: location.into(),
            suggestion: Some(format!("Did you mean to define 'Entity \"{}\"'?", name)),
        }
    }

    /// Create an error for an undefined Resource with a helpful suggestion
    pub fn undefined_resource(name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        Self::UndefinedReference {
            reference_type: "Resource".to_string(),
            name: name.clone(),
            location: location.into(),
            suggestion: Some(format!("Did you mean to define 'Resource \"{}\"'?", name)),
        }
    }

    /// Create an error for an undefined Flow with a helpful suggestion
    pub fn undefined_flow(name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        Self::UndefinedReference {
            reference_type: "Flow".to_string(),
            name: name.clone(),
            location: location.into(),
            suggestion: Some(format!(
                "Did you mean to define a Flow involving '{}'?",
                name
            )),
        }
    }

    /// Create a unit mismatch error with automatic suggestion
    pub fn unit_mismatch(
        expected: Dimension,
        found: Dimension,
        location: impl Into<String>,
    ) -> Self {
        let suggestion = Some(format!(
            "Expected dimension {:?} but found {:?}. Consider using unit conversion or checking your unit definitions.",
            expected, found
        ));
        Self::UnitError {
            expected,
            found,
            location: location.into(),
            suggestion,
        }
    }

    /// Create a type mismatch error with types and suggestion
    pub fn type_mismatch(
        expected: impl Into<String>,
        found: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        let expected = expected.into();
        let found = found.into();
        Self::TypeError {
            message: format!("Type mismatch: expected {}, found {}", expected, found),
            location: location.into(),
            expected_type: Some(expected.clone()),
            found_type: Some(found.clone()),
            suggestion: Some(format!(
                "Convert {} to {} or adjust the expression",
                found, expected
            )),
        }
    }

    /// Create a scope error with available variables listed
    pub fn variable_not_in_scope(
        variable: impl Into<String>,
        available: Vec<String>,
        location: impl Into<String>,
    ) -> Self {
        let variable = variable.into();
        let suggestion = if !available.is_empty() {
            Some(format!(
                "Available variables: {}. Did you mean one of these?",
                available.join(", ")
            ))
        } else {
            Some("No variables are currently in scope.".to_string())
        };

        Self::ScopeError {
            variable,
            available_in: available,
            location: location.into(),
            suggestion,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::SyntaxError {
                message,
                line,
                column,
                end_line,
                end_column,
            } => {
                if let (Some(el), Some(ec)) = (end_line, end_column) {
                    write!(
                        f,
                        "Syntax error at {}:{} to {}:{}: {}",
                        line, column, el, ec, message
                    )
                } else {
                    write!(f, "Syntax error at {}:{}: {}", line, column, message)
                }
            }
            ValidationError::TypeError {
                message,
                location,
                expected_type,
                found_type,
                suggestion,
            } => {
                write!(f, "Type error at {}: {}", location, message)?;
                if let (Some(exp), Some(fnd)) = (expected_type, found_type) {
                    write!(f, " (expected {}, found {})", exp, fnd)?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n  Suggestion: {}", sug)?;
                }
                Ok(())
            }
            ValidationError::UnitError {
                expected,
                found,
                location,
                suggestion,
            } => {
                write!(
                    f,
                    "Unit error at {}: incompatible dimensions (expected {:?}, found {:?})",
                    location, expected, found
                )?;
                if let Some(sug) = suggestion {
                    write!(f, "\n  Suggestion: {}", sug)?;
                }
                Ok(())
            }
            ValidationError::ScopeError {
                variable,
                available_in,
                location,
                suggestion,
            } => {
                write!(
                    f,
                    "Scope error at {}: variable '{}' not in scope",
                    location, variable
                )?;
                if !available_in.is_empty() {
                    write!(f, "\n  Available in: {}", available_in.join(", "))?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\n  Suggestion: {}", sug)?;
                }
                Ok(())
            }
            ValidationError::DeterminismError { message, hint } => {
                write!(f, "Determinism error: {}", message)?;
                write!(f, "\n  Hint: {}", hint)
            }
            ValidationError::UndefinedReference {
                reference_type,
                name,
                location,
                suggestion,
            } => {
                write!(f, "Undefined {} '{}' at {}", reference_type, name, location)?;
                if let Some(sug) = suggestion {
                    write!(f, "\n  Suggestion: {}", sug)?;
                }
                Ok(())
            }
            ValidationError::DuplicateDeclaration {
                name,
                first_location,
                second_location,
            } => {
                write!(
                    f,
                    "Duplicate declaration of '{}': first at {}, duplicate at {}",
                    name, first_location, second_location
                )
            }
            ValidationError::InvalidExpression {
                message,
                location,
                suggestion,
            } => {
                write!(f, "Invalid expression at {}: {}", location, message)?;
                if let Some(sug) = suggestion {
                    write!(f, "\n  Suggestion: {}", sug)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ValidationError {}
