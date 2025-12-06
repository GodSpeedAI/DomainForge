/// Diagnostic formatters for ValidationError
///
/// Provides multiple output formats for error diagnostics:
/// - JSON: Machine-readable format for CI/CD tools
/// - Human: Color-coded format with source snippets for developers
/// - LSP: Language Server Protocol compatible format for IDEs
use crate::validation_error::{SourceRange, ValidationError};
use serde::{Deserialize, Serialize};

/// Trait for formatting validation errors
pub trait DiagnosticFormatter {
    /// Format a validation error with optional source code
    fn format(&self, error: &ValidationError, source: Option<&str>) -> String;
}

/// JSON diagnostic format for machine parsing (CI/CD tools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDiagnostic {
    pub code: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<JsonRange>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRange {
    pub start: JsonPosition,
    pub end: JsonPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPosition {
    pub line: usize,
    pub column: usize,
}

impl From<SourceRange> for JsonRange {
    fn from(range: SourceRange) -> Self {
        JsonRange {
            start: JsonPosition {
                line: range.start.line,
                column: range.start.column,
            },
            end: JsonPosition {
                line: range.end.line,
                column: range.end.column,
            },
        }
    }
}

impl JsonDiagnostic {
    pub fn from_validation_error(error: &ValidationError) -> Self {
        let code = error.error_code();
        let severity = Self::determine_severity(error);
        let range = error.range().map(JsonRange::from);
        let message = error.to_string();
        let hint = Self::extract_hint(error);

        JsonDiagnostic {
            code: code.as_str().to_string(),
            severity,
            range,
            message,
            hint,
        }
    }

    fn determine_severity(error: &ValidationError) -> String {
        match error {
            ValidationError::SyntaxError { .. }
            | ValidationError::TypeError { .. }
            | ValidationError::UnitError { .. }
            | ValidationError::UndefinedReference { .. }
            | ValidationError::DuplicateDeclaration { .. } => "error".to_string(),
            ValidationError::DeterminismError { .. } => "warning".to_string(),
            ValidationError::ScopeError { .. } | ValidationError::InvalidExpression { .. } => {
                "error".to_string()
            }
        }
    }

    fn extract_hint(error: &ValidationError) -> Option<String> {
        match error {
            ValidationError::TypeError { suggestion, .. }
            | ValidationError::UnitError { suggestion, .. }
            | ValidationError::ScopeError { suggestion, .. }
            | ValidationError::UndefinedReference { suggestion, .. }
            | ValidationError::InvalidExpression { suggestion, .. } => suggestion.clone(),
            ValidationError::DeterminismError { hint, .. } => Some(hint.clone()),
            _ => None,
        }
    }
}

/// JSON formatter implementation
pub struct JsonFormatter;

impl DiagnosticFormatter for JsonFormatter {
    fn format(&self, error: &ValidationError, _source: Option<&str>) -> String {
        let diagnostic = JsonDiagnostic::from_validation_error(error);
        serde_json::to_string_pretty(&diagnostic).unwrap_or_else(|_| {
            // Fallback: create a minimal JSON object manually but safely
            let fallback = serde_json::json!({
                "code": error.error_code().as_str(),
                "severity": "error",
                "message": error.to_string()
            });
            serde_json::to_string(&fallback).unwrap_or_else(|_| {
                r#"{"code": "UNKNOWN", "severity": "error", "message": "Failed to serialize error"}"#.to_string()
            })
        })
    }
}

/// Format multiple errors as a JSON array
pub fn format_errors_json(errors: &[ValidationError]) -> String {
    let diagnostics: Vec<JsonDiagnostic> = errors
        .iter()
        .map(JsonDiagnostic::from_validation_error)
        .collect();

    serde_json::to_string_pretty(&diagnostics).unwrap_or_else(|_| "[]".to_string())
}

/// Human-readable diagnostic formatter with color support
pub struct HumanFormatter {
    pub use_color: bool,
    pub show_source: bool,
}

impl Default for HumanFormatter {
    fn default() -> Self {
        Self {
            use_color: true,
            show_source: true,
        }
    }
}

impl HumanFormatter {
    pub fn new(use_color: bool, show_source: bool) -> Self {
        Self {
            use_color,
            show_source,
        }
    }

    fn colorize(&self, text: &str, color: &str) -> String {
        if !self.use_color {
            return text.to_string();
        }

        let color_code = match color {
            "red" => "\x1b[31m",
            "yellow" => "\x1b[33m",
            "blue" => "\x1b[34m",
            "cyan" => "\x1b[36m",
            "bold" => "\x1b[1m",
            _ => "",
        };

        format!("{}{}\x1b[0m", color_code, text)
    }

    fn format_source_snippet(&self, source: &str, range: SourceRange) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let start_line = range.start.line.saturating_sub(1); // Convert to 0-indexed
        let end_line = range.end.line.saturating_sub(1);

        if start_line >= lines.len() {
            return String::new();
        }

        let mut output = String::new();
        let line_num_width = (end_line + 1).to_string().len();

        // Show one line before if available
        if start_line > 0 {
            output.push_str(&format!(
                "{:>width$} | {}\n",
                start_line,
                lines[start_line - 1],
                width = line_num_width
            ));
        }

        let last_line = end_line.min(lines.len().saturating_sub(1));
        if last_line < start_line {
            return output;
        }

        for (line_idx, line) in lines
            .iter()
            .enumerate()
            .skip(start_line)
            .take(last_line.saturating_sub(start_line) + 1)
        {
            let line_num = line_idx + 1;
            output.push_str(&format!(
                "{} | {}\n",
                self.colorize(
                    &format!("{:>width$}", line_num, width = line_num_width),
                    "blue"
                ),
                line
            ));

            // Add caret indicators for the error range
            if line_idx == start_line {
                let padding = " ".repeat(line_num_width + 3);
                let start_col = range.start.column.saturating_sub(1);
                let end_col = if line_idx == end_line {
                    range.end.column.saturating_sub(1)
                } else {
                    line.len()
                };

                let caret_padding = " ".repeat(start_col);
                let carets = "^".repeat(end_col.saturating_sub(start_col).max(1));
                output.push_str(&format!(
                    "{}{}{}\n",
                    padding,
                    caret_padding,
                    self.colorize(&carets, "red")
                ));
            }
        }

        // Show one line after if available
        if end_line + 1 < lines.len() {
            output.push_str(&format!(
                "{:>width$} | {}\n",
                end_line + 2,
                lines[end_line + 1],
                width = line_num_width
            ));
        }

        output
    }
}

impl DiagnosticFormatter for HumanFormatter {
    fn format(&self, error: &ValidationError, source: Option<&str>) -> String {
        let code = error.error_code();
        let severity = match error {
            ValidationError::DeterminismError { .. } => "warning",
            _ => "error",
        };

        let severity_colored = match severity {
            "error" => self.colorize("error", "red"),
            "warning" => self.colorize("warning", "yellow"),
            _ => severity.to_string(),
        };

        let mut output = format!(
            "{}[{}]: {}\n",
            severity_colored,
            self.colorize(code.as_str(), "bold"),
            error
        );

        // Add source snippet if available
        if self.show_source {
            if let (Some(src), Some(range)) = (source, error.range()) {
                output.push_str(&format!("  {} {}\n", self.colorize("-->", "blue"), range));
                output.push_str(&self.format_source_snippet(src, range));
            } else if let Some(location) = error.location_string() {
                output.push_str(&format!(
                    "  {} {}\n",
                    self.colorize("-->", "blue"),
                    location
                ));
            }
        }

        // Add hint if available
        if let Some(hint) = JsonDiagnostic::extract_hint(error) {
            output.push_str(&format!("  {} {}\n", self.colorize("hint:", "cyan"), hint));
        }

        output
    }
}

/// LSP (Language Server Protocol) compatible formatter
pub struct LspFormatter;

impl DiagnosticFormatter for LspFormatter {
    fn format(&self, error: &ValidationError, _source: Option<&str>) -> String {
        let code = error.error_code();
        let severity = match error {
            ValidationError::DeterminismError { .. } => 2, // Warning
            _ => 1,                                        // Error
        };

        let range = error.range();
        let message = error.to_string();
        let hint = JsonDiagnostic::extract_hint(error);

        let full_message = if let Some(h) = hint {
            format!("{}\n\n{}", message, h)
        } else {
            message
        };

        let range_value = if let Some(r) = range {
            serde_json::json!({
                "start": {
                    "line": r.start.line.saturating_sub(1),
                    "character": r.start.column.saturating_sub(1)
                },
                "end": {
                    "line": r.end.line.saturating_sub(1),
                    "character": r.end.column.saturating_sub(1)
                }
            })
        } else {
            serde_json::json!({
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 0 }
            })
        };

        let diagnostic = serde_json::json!({
            "range": range_value,
            "severity": severity,
            "code": code.as_str(),
            "source": "sea-dsl",
            "message": full_message
        });

        serde_json::to_string(&diagnostic).unwrap_or_else(|_| {
            r#"{"severity": 1, "message": "Failed to serialize diagnostic"}"#.to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_formatter() {
        let error = ValidationError::syntax_error("unexpected token", 10, 5);
        let formatter = JsonFormatter;
        let output = formatter.format(&error, None);

        assert!(output.contains("E005"));
        assert!(output.contains("error"));
        assert!(output.contains("unexpected token"));
    }

    #[test]
    fn test_json_formatter_with_range() {
        let error = ValidationError::syntax_error_with_range("test", 10, 5, 10, 15);
        let formatter = JsonFormatter;
        let output = formatter.format(&error, None);

        assert!(output.contains(r#""line": 10"#));
        assert!(output.contains(r#""column": 5"#));
    }

    #[test]
    fn test_json_formatter_with_hint() {
        let candidates = vec!["Warehouse".to_string()];
        let error =
            ValidationError::undefined_entity_with_candidates("Warehous", "line 10", &candidates);
        let formatter = JsonFormatter;
        let output = formatter.format(&error, None);

        assert!(output.contains("E001"));
        assert!(output.contains("Warehouse"));
    }

    #[test]
    fn test_human_formatter_no_color() {
        let error = ValidationError::syntax_error("test error", 1, 1);
        let formatter = HumanFormatter::new(false, false);
        let output = formatter.format(&error, None);

        assert!(output.contains("error[E005]"));
        assert!(output.contains("test error"));
        assert!(!output.contains("\x1b[")); // No ANSI codes
    }

    #[test]
    fn test_human_formatter_with_color() {
        let error = ValidationError::syntax_error("test error", 1, 1);
        let formatter = HumanFormatter::new(true, false);
        let output = formatter.format(&error, None);

        assert!(output.contains("\x1b[")); // Has ANSI codes
    }

    #[test]
    fn test_lsp_formatter() {
        let error = ValidationError::syntax_error_with_range("test", 10, 5, 10, 15);
        let formatter = LspFormatter;
        let output = formatter.format(&error, None);

        let json: serde_json::Value = serde_json::from_str(&output).expect("should be valid json");
        assert_eq!(json["severity"], 1);
        assert_eq!(json["code"], "E005");
        assert_eq!(json["source"], "sea-dsl");
        // LSP uses 0-indexed lines
        assert_eq!(json["range"]["start"]["line"], 9);
    }

    #[test]
    fn test_format_multiple_errors() {
        let errors = vec![
            ValidationError::syntax_error("error 1", 1, 1),
            ValidationError::undefined_entity("Test", "line 5"),
        ];

        let output = format_errors_json(&errors);
        assert!(output.contains("E005"));
        assert!(output.contains("E001"));
    }
}
