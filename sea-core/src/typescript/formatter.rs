//! TypeScript bindings for the SEA formatter.

use crate::formatter::{format, FormatConfig, IndentStyle};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Configuration options for the formatter.
#[napi(object)]
pub struct FormatOptions {
    /// Number of spaces per indent level (default: 4)
    pub indent_width: Option<u32>,
    /// Use tabs instead of spaces for indentation (default: false)
    pub use_tabs: Option<bool>,
    /// Preserve comments in output (default: true)
    pub preserve_comments: Option<bool>,
    /// Sort imports alphabetically (default: true)
    pub sort_imports: Option<bool>,
}

/// Format SEA-DSL source code.
///
/// @param source - The SEA-DSL source code to format
/// @param options - Optional formatting configuration
/// @returns The formatted source code
/// @throws Error if the source code cannot be parsed
///
/// @example
/// ```typescript
/// import { formatSource } from '@domainforge/sea';
///
/// const formatted = formatSource('Entity   "Foo"  in    bar');
/// console.log(formatted); // Entity "Foo" in bar
/// ```
#[napi]
pub fn format_source(source: String, options: Option<FormatOptions>) -> Result<String> {
    let opts = options.unwrap_or(FormatOptions {
        indent_width: None,
        use_tabs: None,
        preserve_comments: None,
        sort_imports: None,
    });

    let config = FormatConfig {
        indent_width: opts.indent_width.unwrap_or(4) as usize,
        indent_style: if opts.use_tabs.unwrap_or(false) {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces
        },
        preserve_comments: opts.preserve_comments.unwrap_or(true),
        sort_imports: opts.sort_imports.unwrap_or(true),
        ..Default::default()
    };

    format(&source, config).map_err(|e| Error::from_reason(e.to_string()))
}

/// Check if SEA-DSL source code is already formatted.
///
/// @param source - The SEA-DSL source code to check
/// @param options - Optional formatting configuration
/// @returns True if the source is already formatted, false otherwise
/// @throws Error if the source code cannot be parsed
///
/// @example
/// ```typescript
/// import { checkFormat } from '@domainforge/sea';
///
/// console.log(checkFormat('Entity "Foo" in bar\n')); // true
/// console.log(checkFormat('Entity   "Foo"  in    bar')); // false
/// ```
#[napi]
pub fn check_format(source: String, options: Option<FormatOptions>) -> Result<bool> {
    let formatted = format_source(source.clone(), options)?;
    Ok(source == formatted)
}
