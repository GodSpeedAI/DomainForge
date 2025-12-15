//! WASM bindings for the SEA formatter.

use crate::formatter::{format, FormatConfig, IndentStyle};
use wasm_bindgen::prelude::*;

/// Format SEA-DSL source code.
///
/// @param source - The SEA-DSL source code to format
/// @param indentWidth - Number of spaces per indent level (default: 4)
/// @param useTabs - Use tabs instead of spaces for indentation (default: false)
/// @param preserveComments - Preserve comments in output (default: true)
/// @param sortImports - Sort imports alphabetically (default: true)
/// @returns The formatted source code
/// @throws Error if the source code cannot be parsed
///
/// @example
/// ```javascript
/// import { formatSource } from '@domainforge/sea-wasm';
///
/// const formatted = formatSource('Entity   "Foo"  in    bar');
/// console.log(formatted); // Entity "Foo" in bar
/// ```
#[wasm_bindgen(js_name = formatSource)]
pub fn format_source(
    source: &str,
    indent_width: Option<u32>,
    use_tabs: Option<bool>,
    preserve_comments: Option<bool>,
    sort_imports: Option<bool>,
) -> Result<String, JsValue> {
    let config = FormatConfig {
        indent_width: indent_width.unwrap_or(4) as usize,
        indent_style: if use_tabs.unwrap_or(false) {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces
        },
        preserve_comments: preserve_comments.unwrap_or(true),
        sort_imports: sort_imports.unwrap_or(true),
        ..Default::default()
    };

    format(source, config).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Check if SEA-DSL source code is already formatted.
///
/// @param source - The SEA-DSL source code to check
/// @param indentWidth - Number of spaces per indent level (default: 4)
/// @param useTabs - Use tabs instead of spaces for indentation (default: false)
/// @returns True if the source is already formatted, false otherwise
/// @throws Error if the source code cannot be parsed
///
/// @example
/// ```javascript
/// import { checkFormat } from '@domainforge/sea-wasm';
///
/// console.log(checkFormat('Entity "Foo" in bar\n')); // true
/// console.log(checkFormat('Entity   "Foo"  in    bar')); // false
/// ```
#[wasm_bindgen(js_name = checkFormat)]
pub fn check_format(
    source: &str,
    indent_width: Option<u32>,
    use_tabs: Option<bool>,
) -> Result<bool, JsValue> {
    let formatted = format_source(source, indent_width, use_tabs, Some(true), Some(true))?;
    Ok(source == formatted)
}
