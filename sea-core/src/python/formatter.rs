//! Python bindings for the SEA formatter.

use crate::formatter::{format, FormatConfig, IndentStyle};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Format SEA-DSL source code.
///
/// Args:
///     source: The SEA-DSL source code to format
///     indent_width: Number of spaces per indent level (default: 4)
///     use_tabs: Use tabs instead of spaces for indentation (default: false)
///     preserve_comments: Preserve comments in output (default: true)
///     sort_imports: Sort imports alphabetically (default: true)
///
/// Returns:
///     The formatted source code
///
/// Raises:
///     ValueError: If the source code cannot be parsed
///
/// Example:
///     >>> from sea_dsl import format_source
///     >>> formatted = format_source('Entity   "Foo"  in    bar')
///     >>> print(formatted)
///     Entity "Foo" in bar
#[pyfunction]
#[pyo3(signature = (source, indent_width=4, use_tabs=false, preserve_comments=true, sort_imports=true))]
pub fn format_source(
    source: &str,
    indent_width: usize,
    use_tabs: bool,
    preserve_comments: bool,
    sort_imports: bool,
) -> PyResult<String> {
    let config = FormatConfig {
        indent_width,
        indent_style: if use_tabs {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces
        },
        preserve_comments,
        sort_imports,
        ..Default::default()
    };

    format(source, config).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Check if SEA-DSL source code is already formatted.
///
/// Args:
///     source: The SEA-DSL source code to check
///     indent_width: Number of spaces per indent level (default: 4)
///     use_tabs: Use tabs instead of spaces for indentation (default: false)
///
/// Returns:
///     True if the source is already formatted, False otherwise
///
/// Raises:
///     ValueError: If the source code cannot be parsed
///
/// Example:
///     >>> from sea_dsl import check_format
///     >>> check_format('Entity "Foo" in bar\\n')
///     True
///     >>> check_format('Entity   "Foo"  in    bar')
///     False
#[pyfunction]
#[pyo3(signature = (source, indent_width=4, use_tabs=false))]
pub fn check_format(source: &str, indent_width: usize, use_tabs: bool) -> PyResult<bool> {
    let config = FormatConfig {
        indent_width,
        indent_style: if use_tabs {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces
        },
        ..Default::default()
    };

    let formatted = format(source, config).map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(source == formatted)
}
