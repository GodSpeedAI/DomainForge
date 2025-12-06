use crate::units::Dimension;
#[cfg(feature = "python")]
use pyo3::{IntoPyObject, PyAny, Python};
use std::fmt;

/// Threshold for fuzzy matching suggestions (Levenshtein distance)
pub const FUZZY_MATCH_THRESHOLD: usize = 2;

/// Position in source code (line and column, 1-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Result<Self, String> {
        if line == 0 || column == 0 {
            Err(format!(
                "Position line and column must be >= 1. Got line={}, column={}",
                line, column
            ))
        } else {
            Ok(Self { line, column })
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Source range with start and end positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: Position,
    pub end: Position,
}

impl SourceRange {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_line_col(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            start: Position::new(start_line, start_col)?,
            end: Position::new(end_line, end_col)?,
        })
    }

    pub fn single_position(line: usize, column: usize) -> Result<Self, String> {
        let pos = Position::new(line, column)?;
        Ok(Self {
            start: pos,
            end: pos,
        })
    }
}

impl fmt::Display for SourceRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(f, "{} to {}", self.start, self.end)
        }
    }
}

/// Error codes for all validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ErrorCode {
    // E001-E099: Syntax and parsing errors
    E001_UndefinedEntity,
    E002_UndefinedResource,
    E003_UnitMismatch,
    E004_TypeMismatch,
    E005_SyntaxError,
    E006_InvalidExpression,
    E007_DuplicateDeclaration,
    E008_UndefinedVariable,
    E009_InvalidQuantity,
    E010_InvalidIdentifier,

    // E100-E199: Type system errors
    E100_IncompatibleTypes,
    E101_InvalidTypeConversion,
    E102_TypeInferenceFailed,
    E103_InvalidOperandType,
    E104_InvalidComparisonType,

    // E200-E299: Unit and dimension errors
    E200_DimensionMismatch,
    E201_InvalidUnit,
    E202_UnitConversionFailed,
    E203_IncompatibleDimensions,

    // E300-E399: Scope and reference errors
    E300_VariableNotInScope,
    E301_UndefinedReference,
    E302_CircularReference,
    E303_InvalidReference,

    // E400-E499: Policy validation errors
    E400_PolicyEvaluationFailed,
    E401_InvalidPolicyExpression,
    E402_DeterminismViolation,
    E403_InvalidModality,

    // E500-E599: Namespace and module errors
    E500_NamespaceNotFound,
    E501_AmbiguousNamespace,
    E502_InvalidNamespace,
    E503_ModuleNotFound,
    E504_SymbolNotExported,
    E505_CircularDependency,
    E506_AmbiguousImport,
    E507_InvalidExport,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Syntax and parsing
            ErrorCode::E001_UndefinedEntity => "E001",
            ErrorCode::E002_UndefinedResource => "E002",
            ErrorCode::E003_UnitMismatch => "E003",
            ErrorCode::E004_TypeMismatch => "E004",
            ErrorCode::E005_SyntaxError => "E005",
            ErrorCode::E006_InvalidExpression => "E006",
            ErrorCode::E007_DuplicateDeclaration => "E007",
            ErrorCode::E008_UndefinedVariable => "E008",
            ErrorCode::E009_InvalidQuantity => "E009",
            ErrorCode::E010_InvalidIdentifier => "E010",

            // Type system
            ErrorCode::E100_IncompatibleTypes => "E100",
            ErrorCode::E101_InvalidTypeConversion => "E101",
            ErrorCode::E102_TypeInferenceFailed => "E102",
            ErrorCode::E103_InvalidOperandType => "E103",
            ErrorCode::E104_InvalidComparisonType => "E104",

            // Units and dimensions
            ErrorCode::E200_DimensionMismatch => "E200",
            ErrorCode::E201_InvalidUnit => "E201",
            ErrorCode::E202_UnitConversionFailed => "E202",
            ErrorCode::E203_IncompatibleDimensions => "E203",

            // Scope and references
            ErrorCode::E300_VariableNotInScope => "E300",
            ErrorCode::E301_UndefinedReference => "E301",
            ErrorCode::E302_CircularReference => "E302",
            ErrorCode::E303_InvalidReference => "E303",

            // Policy validation
            ErrorCode::E400_PolicyEvaluationFailed => "E400",
            ErrorCode::E401_InvalidPolicyExpression => "E401",
            ErrorCode::E402_DeterminismViolation => "E402",
            ErrorCode::E403_InvalidModality => "E403",

            // Namespace and modules
            ErrorCode::E500_NamespaceNotFound => "E500",
            ErrorCode::E501_AmbiguousNamespace => "E501",
            ErrorCode::E502_InvalidNamespace => "E502",
            ErrorCode::E503_ModuleNotFound => "E503",
            ErrorCode::E504_SymbolNotExported => "E504",
            ErrorCode::E505_CircularDependency => "E505",
            ErrorCode::E506_AmbiguousImport => "E506",
            ErrorCode::E507_InvalidExport => "E507",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::E001_UndefinedEntity => "Undefined entity",
            ErrorCode::E002_UndefinedResource => "Undefined resource",
            ErrorCode::E003_UnitMismatch => "Unit mismatch",
            ErrorCode::E004_TypeMismatch => "Type mismatch",
            ErrorCode::E005_SyntaxError => "Syntax error",
            ErrorCode::E006_InvalidExpression => "Invalid expression",
            ErrorCode::E007_DuplicateDeclaration => "Duplicate declaration",
            ErrorCode::E008_UndefinedVariable => "Undefined variable",
            ErrorCode::E009_InvalidQuantity => "Invalid quantity",
            ErrorCode::E010_InvalidIdentifier => "Invalid identifier",
            ErrorCode::E100_IncompatibleTypes => "Incompatible types",
            ErrorCode::E101_InvalidTypeConversion => "Invalid type conversion",
            ErrorCode::E102_TypeInferenceFailed => "Type inference failed",
            ErrorCode::E103_InvalidOperandType => "Invalid operand type",
            ErrorCode::E104_InvalidComparisonType => "Invalid comparison type",
            ErrorCode::E200_DimensionMismatch => "Dimension mismatch",
            ErrorCode::E201_InvalidUnit => "Invalid unit",
            ErrorCode::E202_UnitConversionFailed => "Unit conversion failed",
            ErrorCode::E203_IncompatibleDimensions => "Incompatible dimensions",
            ErrorCode::E300_VariableNotInScope => "Variable not in scope",
            ErrorCode::E301_UndefinedReference => "Undefined reference",
            ErrorCode::E302_CircularReference => "Circular reference",
            ErrorCode::E303_InvalidReference => "Invalid reference",
            ErrorCode::E400_PolicyEvaluationFailed => "Policy evaluation failed",
            ErrorCode::E401_InvalidPolicyExpression => "Invalid policy expression",
            ErrorCode::E402_DeterminismViolation => "Determinism violation",
            ErrorCode::E403_InvalidModality => "Invalid modality",
            ErrorCode::E500_NamespaceNotFound => "Namespace not found",
            ErrorCode::E501_AmbiguousNamespace => "Ambiguous namespace",
            ErrorCode::E502_InvalidNamespace => "Invalid namespace",
            ErrorCode::E503_ModuleNotFound => "Module not found",
            ErrorCode::E504_SymbolNotExported => "Imported symbol is not exported",
            ErrorCode::E505_CircularDependency => "Circular dependency detected",
            ErrorCode::E506_AmbiguousImport => "Ambiguous import",
            ErrorCode::E507_InvalidExport => "Invalid export",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ReferenceType {
    Entity,
    Resource,
    Variable,
    Flow,
    Other(String),
}

impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferenceType::Entity => write!(f, "Entity"),
            ReferenceType::Resource => write!(f, "Resource"),
            ReferenceType::Variable => write!(f, "Variable"),
            ReferenceType::Flow => write!(f, "Flow"),
            ReferenceType::Other(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(feature = "python")]
impl<'py> IntoPyObject<'py> for ReferenceType {
    type Target = PyAny;
    type Output = pyo3::Bound<'py, PyAny>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> pyo3::PyResult<Self::Output> {
        Ok(pyo3::types::PyString::new(py, &self.to_string()).into_any())
    }
}

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
        reference_type: ReferenceType,
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
    /// Get the error code for this validation error
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ValidationError::SyntaxError { .. } => ErrorCode::E005_SyntaxError,
            ValidationError::TypeError { .. } => ErrorCode::E004_TypeMismatch,
            ValidationError::UnitError { .. } => ErrorCode::E003_UnitMismatch,
            ValidationError::ScopeError { .. } => ErrorCode::E300_VariableNotInScope,
            ValidationError::DeterminismError { .. } => ErrorCode::E402_DeterminismViolation,
            ValidationError::UndefinedReference { reference_type, .. } => match reference_type {
                ReferenceType::Entity => ErrorCode::E001_UndefinedEntity,
                ReferenceType::Resource => ErrorCode::E002_UndefinedResource,
                ReferenceType::Variable => ErrorCode::E008_UndefinedVariable,
                _ => ErrorCode::E301_UndefinedReference,
            },
            ValidationError::DuplicateDeclaration { .. } => ErrorCode::E007_DuplicateDeclaration,
            ValidationError::InvalidExpression { .. } => ErrorCode::E006_InvalidExpression,
        }
    }

    /// Get the source range for this error (if available)
    pub fn range(&self) -> Option<SourceRange> {
        match self {
            ValidationError::SyntaxError {
                line,
                column,
                end_line,
                end_column,
                ..
            } => {
                // We ignore errors here as this is just for reporting
                let start =
                    Position::new(*line, *column).unwrap_or_else(|_| Position::new(1, 1).unwrap());
                let end = match (end_line, end_column) {
                    (Some(el), Some(ec)) => Position::new(*el, *ec).unwrap_or(start),
                    _ => start,
                };
                Some(SourceRange::new(start, end))
            }
            _ => None, // Other variants use string locations for now
        }
    }

    /// Get a user-friendly location string
    pub fn location_string(&self) -> Option<String> {
        match self {
            ValidationError::SyntaxError { line, column, .. } => {
                Some(format!("{}:{}", line, column))
            }
            ValidationError::TypeError { location, .. }
            | ValidationError::UnitError { location, .. }
            | ValidationError::ScopeError { location, .. }
            | ValidationError::UndefinedReference { location, .. }
            | ValidationError::InvalidExpression { location, .. } => Some(location.clone()),
            ValidationError::DuplicateDeclaration {
                second_location, ..
            } => Some(second_location.clone()),
            ValidationError::DeterminismError { .. } => None,
        }
    }

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
            reference_type: ReferenceType::Other(reference_type.into()),
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

    // Additional convenience constructors for common error patterns

    /// Create an error for an undefined Entity with a helpful suggestion
    pub fn undefined_entity(name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        Self::UndefinedReference {
            reference_type: ReferenceType::Entity,
            name: name.clone(),
            location: location.into(),
            suggestion: Some(format!("Did you mean to define 'Entity \"{}\"'?", name)),
        }
    }

    /// Create an error for an undefined Resource with a helpful suggestion
    pub fn undefined_resource(name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        Self::UndefinedReference {
            reference_type: ReferenceType::Resource,
            name: name.clone(),
            location: location.into(),
            suggestion: Some(format!("Did you mean to define 'Resource \"{}\"'?", name)),
        }
    }

    /// Create an error for an undefined Flow with a helpful suggestion
    pub fn undefined_flow(name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        Self::UndefinedReference {
            reference_type: ReferenceType::Flow,
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

    /// Create an undefined entity error with fuzzy matching suggestions
    ///
    /// # Arguments
    /// * `name` - The undefined entity name
    /// * `location` - Source location of the error
    /// * `candidates` - Available entity names to suggest
    fn undefined_reference_with_candidates(
        reference_type: ReferenceType,
        name: String,
        location: String,
        candidates: &[String],
    ) -> Self {
        use crate::error::fuzzy::find_best_match;

        let suggestion = find_best_match(&name, candidates, FUZZY_MATCH_THRESHOLD)
            .map(|match_name| format!("Did you mean '{}'?", match_name))
            .or_else(|| {
                Some(format!(
                    "Did you mean to define '{} \"{}\"'?",
                    reference_type, name
                ))
            });

        Self::UndefinedReference {
            reference_type,
            name,
            location,
            suggestion,
        }
    }

    /// Create an undefined entity error with fuzzy matching suggestions
    ///
    /// # Arguments
    /// * `name` - The undefined entity name
    /// * `location` - Source location of the error
    /// * `candidates` - Available entity names to suggest
    pub fn undefined_entity_with_candidates(
        name: impl Into<String>,
        location: impl Into<String>,
        candidates: &[String],
    ) -> Self {
        Self::undefined_reference_with_candidates(
            ReferenceType::Entity,
            name.into(),
            location.into(),
            candidates,
        )
    }

    /// Create an undefined resource error with fuzzy matching suggestions
    pub fn undefined_resource_with_candidates(
        name: impl Into<String>,
        location: impl Into<String>,
        candidates: &[String],
    ) -> Self {
        Self::undefined_reference_with_candidates(
            ReferenceType::Resource,
            name.into(),
            location.into(),
            candidates,
        )
    }

    /// Create an undefined variable error with fuzzy matching suggestions
    pub fn undefined_variable_with_candidates(
        name: impl Into<String>,
        location: impl Into<String>,
        candidates: &[String],
    ) -> Self {
        use crate::error::fuzzy::suggest_similar;

        let name = name.into();
        let matches = suggest_similar(&name, candidates, FUZZY_MATCH_THRESHOLD);
        let suggestion = if !matches.is_empty() {
            let quoted: Vec<String> = matches.iter().map(|m| format!("'{}'", m)).collect();
            Some(format!("Did you mean {}?", quoted.join(", ")))
        } else {
            Some("No similar variables found in scope.".to_string())
        };

        Self::UndefinedReference {
            reference_type: ReferenceType::Variable,
            name,
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
