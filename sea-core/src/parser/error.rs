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
    UndefinedEntity(String),
    UndefinedResource(String),
    UndefinedVariable(String),
    DuplicateDeclaration(String),
    TypeError {
        message: String,
        location: String,
    },
    InvalidExpression(String),
    InvalidQuantity(String),
    Validation(String),
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

    pub fn undefined_entity(name: impl Into<String>) -> Self {
        ParseError::UndefinedEntity(name.into())
    }

    pub fn undefined_resource(name: impl Into<String>) -> Self {
        ParseError::UndefinedResource(name.into())
    }

    pub fn undefined_variable(name: impl Into<String>) -> Self {
        ParseError::UndefinedVariable(name.into())
    }

    pub fn duplicate_declaration(name: impl Into<String>) -> Self {
        ParseError::DuplicateDeclaration(name.into())
    }

    pub fn type_error(message: impl Into<String>, location: impl Into<String>) -> Self {
        ParseError::TypeError {
            message: message.into(),
            location: location.into(),
        }
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
            ParseError::UndefinedEntity(name) => {
                write!(f, "Undefined entity: {}", name)
            }
            ParseError::UndefinedResource(name) => {
                write!(f, "Undefined resource: {}", name)
            }
            ParseError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            }
            ParseError::DuplicateDeclaration(name) => {
                write!(f, "Duplicate declaration: {}", name)
            }
            ParseError::TypeError { message, location } => {
                write!(f, "Type error at {}: {}", location, message)
            }
            ParseError::InvalidExpression(msg) => write!(f, "Invalid expression: {}", msg),
            ParseError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {}", msg),
            ParseError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<PestError<Rule>> for ParseError {
    fn from(err: PestError<Rule>) -> Self {
        ParseError::from_pest(err)
    }
}
