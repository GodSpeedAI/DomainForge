/// Error handling and diagnostics module
///
/// This module provides comprehensive error handling infrastructure including:
/// - Error codes for all validation errors
/// - Source position and range tracking
/// - Fuzzy string matching for "did you mean?" suggestions
/// - Multiple diagnostic output formatters (JSON, Human-readable, LSP)
pub mod diagnostics;
pub mod fuzzy;

// Re-export commonly used types
pub use crate::validation_error::{ErrorCode, Position, SourceRange, ValidationError};
pub use diagnostics::{DiagnosticFormatter, HumanFormatter, JsonFormatter, LspFormatter};
