use crate::parser::Rule;
use pest::error::Error as PestError;
use std::fmt;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },
    UnsupportedExpression {
        kind: String,
        span: Option<String>,
    },
    GrammarError(String),
    UndefinedEntity {
        name: String,
        line: usize,
        column: usize,
    },
    UndefinedResource {
        name: String,
        line: usize,
        column: usize,
    },
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },
    DuplicateDeclaration {
        name: String,
        line: usize,
        column: usize,
    },
    TypeError {
        message: String,
        location: String,
    },
    InvalidExpression(String),
    InvalidQuantity(String),
    Validation(String),

    // E500-E599: Namespace and module errors
    /// E500: Referenced namespace does not exist
    NamespaceNotFound {
        namespace: String,
        line: usize,
        column: usize,
        suggestion: Option<String>,
    },
    /// E503: Referenced module file could not be found
    ModuleNotFound {
        module_path: String,
        line: usize,
        column: usize,
    },
    /// E504: Imported symbol is not exported by the target module
    SymbolNotExported {
        symbol: String,
        module: String,
        line: usize,
        column: usize,
        available_exports: Vec<String>,
    },
    /// E505: Circular dependency detected between modules
    CircularDependency {
        cycle: Vec<String>,
    },
}

impl ParseError {
    pub fn from_pest(err: PestError<Rule>) -> Self {
        let (line, column) = match err.line_col {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), _) => (l, c),
        };

        ParseError::SyntaxError {
            message: err.variant.message().to_string(),
            line,
            column,
        }
    }

    pub fn syntax_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        ParseError::SyntaxError {
            message: message.into(),
            line,
            column,
        }
    }

    pub fn undefined_entity(name: impl Into<String>, line: usize, column: usize) -> Self {
        ParseError::UndefinedEntity {
            name: name.into(),
            line,
            column,
        }
    }

    /// Creates an UndefinedEntity error without location information (uses 0:0)
    pub fn undefined_entity_no_loc(name: impl Into<String>) -> Self {
        Self::undefined_entity(name, 0, 0)
    }

    pub fn undefined_resource(name: impl Into<String>, line: usize, column: usize) -> Self {
        ParseError::UndefinedResource {
            name: name.into(),
            line,
            column,
        }
    }

    /// Creates an UndefinedResource error without location information (uses 0:0)
    pub fn undefined_resource_no_loc(name: impl Into<String>) -> Self {
        Self::undefined_resource(name, 0, 0)
    }

    pub fn undefined_variable(name: impl Into<String>, line: usize, column: usize) -> Self {
        ParseError::UndefinedVariable {
            name: name.into(),
            line,
            column,
        }
    }

    pub fn duplicate_declaration(name: impl Into<String>, line: usize, column: usize) -> Self {
        ParseError::DuplicateDeclaration {
            name: name.into(),
            line,
            column,
        }
    }

    /// Creates a DuplicateDeclaration error without location information (uses 0:0)
    pub fn duplicate_declaration_no_loc(name: impl Into<String>) -> Self {
        Self::duplicate_declaration(name, 0, 0)
    }

    pub fn type_error(message: impl Into<String>, location: impl Into<String>) -> Self {
        ParseError::TypeError {
            message: message.into(),
            location: location.into(),
        }
    }

    /// E500: Namespace not found error
    pub fn namespace_not_found(
        namespace: impl Into<String>,
        line: usize,
        column: usize,
        suggestion: Option<String>,
    ) -> Self {
        ParseError::NamespaceNotFound {
            namespace: namespace.into(),
            line,
            column,
            suggestion,
        }
    }

    /// E503: Module not found error
    pub fn module_not_found(module_path: impl Into<String>, line: usize, column: usize) -> Self {
        ParseError::ModuleNotFound {
            module_path: module_path.into(),
            line,
            column,
        }
    }

    /// E504: Symbol not exported error
    pub fn symbol_not_exported(
        symbol: impl Into<String>,
        module: impl Into<String>,
        line: usize,
        column: usize,
        available_exports: Vec<String>,
    ) -> Self {
        ParseError::SymbolNotExported {
            symbol: symbol.into(),
            module: module.into(),
            line,
            column,
            available_exports,
        }
    }

    /// E505: Circular dependency error
    pub fn circular_dependency(cycle: Vec<String>) -> Self {
        ParseError::CircularDependency { cycle }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError {
                message,
                line,
                column,
            } => {
                write!(f, "Syntax error at {}:{}: {}", line, column, message)
            }
            ParseError::UnsupportedExpression { kind, span } => {
                if let Some(span) = span {
                    write!(f, "Unsupported expression '{}' at {}", kind, span)
                } else {
                    write!(f, "Unsupported expression '{}'", kind)
                }
            }
            ParseError::GrammarError(msg) => write!(f, "Grammar error: {}", msg),
            ParseError::UndefinedEntity { name, line, column } => {
                write!(f, "Undefined entity: {} at {}:{}", name, line, column)
            }
            ParseError::UndefinedResource { name, line, column } => {
                write!(f, "Undefined resource: {} at {}:{}", name, line, column)
            }
            ParseError::UndefinedVariable { name, line, column } => {
                write!(f, "Undefined variable: {} at {}:{}", name, line, column)
            }
            ParseError::DuplicateDeclaration { name, line, column } => {
                write!(f, "Duplicate declaration: {} at {}:{}", name, line, column)
            }
            ParseError::TypeError { message, location } => {
                write!(f, "Type error at {}: {}", location, message)
            }
            ParseError::InvalidExpression(msg) => write!(f, "Invalid expression: {}", msg),
            ParseError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {}", msg),
            ParseError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ParseError::NamespaceNotFound {
                namespace,
                line,
                column,
                suggestion,
            } => {
                write!(
                    f,
                    "Namespace '{}' not found at {}:{}",
                    namespace, line, column
                )?;
                if let Some(sug) = suggestion {
                    write!(f, ". Did you mean '{}'?", sug)?;
                }
                Ok(())
            }
            ParseError::ModuleNotFound {
                module_path,
                line,
                column,
            } => {
                write!(
                    f,
                    "Module '{}' not found at {}:{}",
                    module_path, line, column
                )
            }
            ParseError::SymbolNotExported {
                symbol,
                module,
                line,
                column,
                available_exports,
            } => {
                write!(
                    f,
                    "Symbol '{}' is not exported by module '{}' at {}:{}",
                    symbol, module, line, column
                )?;
                if !available_exports.is_empty() {
                    write!(f, ". Available exports: {}", available_exports.join(", "))?;
                }
                Ok(())
            }
            ParseError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {}", cycle.join(" -> "))
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl From<PestError<Rule>> for ParseError {
    fn from(err: PestError<Rule>) -> Self {
        ParseError::from_pest(err)
    }
}
