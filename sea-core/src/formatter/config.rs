//! Formatter configuration options.

use serde::{Deserialize, Serialize};

/// Style of indentation to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IndentStyle {
    /// Use spaces for indentation (default).
    #[default]
    Spaces,
    /// Use tabs for indentation.
    Tabs,
}

/// Configuration options for the SEA code formatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Style of indentation (spaces or tabs).
    pub indent_style: IndentStyle,
    /// Number of spaces per indentation level (ignored if using tabs).
    pub indent_width: usize,
    /// Maximum line width before wrapping (advisory).
    pub max_line_width: usize,
    /// Whether to ensure file ends with a newline.
    pub trailing_newline: bool,
    /// Whether to preserve comments in output.
    pub preserve_comments: bool,
    /// Whether to sort imports alphabetically.
    pub sort_imports: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces,
            indent_width: 4,
            max_line_width: 100,
            trailing_newline: true,
            preserve_comments: true,
            sort_imports: true,
        }
    }
}

impl FormatConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the indent style.
    pub fn with_indent_style(mut self, style: IndentStyle) -> Self {
        self.indent_style = style;
        self
    }

    /// Set the indent width (for spaces).
    pub fn with_indent_width(mut self, width: usize) -> Self {
        self.indent_width = width;
        self
    }

    /// Set whether to use tabs.
    pub fn with_tabs(mut self) -> Self {
        self.indent_style = IndentStyle::Tabs;
        self
    }

    /// Get the string to use for one level of indentation.
    pub fn indent_string(&self) -> String {
        match self.indent_style {
            IndentStyle::Spaces => " ".repeat(self.indent_width),
            IndentStyle::Tabs => "\t".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FormatConfig::default();
        assert_eq!(config.indent_style, IndentStyle::Spaces);
        assert_eq!(config.indent_width, 4);
        assert_eq!(config.max_line_width, 100);
        assert!(config.trailing_newline);
        assert!(config.preserve_comments);
        assert!(config.sort_imports);
    }

    #[test]
    fn test_indent_string_spaces() {
        let config = FormatConfig::default().with_indent_width(2);
        assert_eq!(config.indent_string(), "  ");
    }

    #[test]
    fn test_indent_string_tabs() {
        let config = FormatConfig::default().with_tabs();
        assert_eq!(config.indent_string(), "\t");
    }

    #[test]
    fn test_builder_pattern() {
        let config = FormatConfig::new()
            .with_indent_style(IndentStyle::Tabs)
            .with_indent_width(8);

        assert_eq!(config.indent_style, IndentStyle::Tabs);
        assert_eq!(config.indent_width, 8);
    }
}
