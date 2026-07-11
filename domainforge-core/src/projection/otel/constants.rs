//! Attribute-constant file renderers (Rust / Python / TypeScript).
//!
//! All three read the SAME attribute set from [`TelemetryIR::all_attributes`]
//! (deduplicated, sorted by key) and differ only in surface syntax and identifier
//! casing — there is no per-language business logic. This is the single-producer
//! guarantee: the constant files and the YAML registry are two renderings of one
//! IR, so they can never disagree on the attribute set (asserted by a test).

use super::ir::{Attribute, TelemetryIR};

/// The generated constant identifier for an attribute key:
/// upper-snaked, non-alphanumerics collapsed to `_`, leading digit prefixed.
/// `demo.entity.warehouse.id` → `DEMO_ENTITY_WAREHOUSE_ID`.
fn const_name(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    for ch in key.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "_".to_string();
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// Escape a `//`/`#` doc comment line: strip newlines so one attribute brief is
/// always a single comment line.
fn doc_line(brief: &str) -> String {
    brief.replace(['\n', '\r'], " ")
}

/// Escape a string literal body for Rust/TS double-quoted strings.
fn escape_lit(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render `attributes.rs`: `pub const NAME: &str = "key";` per attribute.
pub fn render_rust(ir: &TelemetryIR, header: &str) -> String {
    let mut s = rust_header(ir, header);
    for a in ir.all_attributes() {
        s.push_str(&emit_rust(a));
    }
    s
}

fn rust_header(ir: &TelemetryIR, header: &str) -> String {
    format!(
        "{}//! OpenTelemetry attribute keys projected from model `{}`.\n\
         //! Model hash: {}. Do not edit by hand.\n\n",
        line_comment(header, "//"),
        escape_lit(&ir.model_ref),
        ir.model_hash
    )
}

fn emit_rust(a: &Attribute) -> String {
    format!(
        "/// {}\npub const {}: &str = \"{}\";\n",
        doc_line(&a.brief),
        const_name(&a.key),
        escape_lit(&a.key)
    )
}

/// Render `attributes.py`: `NAME: str = "key"` per attribute.
pub fn render_python(ir: &TelemetryIR, header: &str) -> String {
    let mut s = format!(
        "{}\"\"\"OpenTelemetry attribute keys projected from model `{}`.\n\n\
         Model hash: {}. Do not edit by hand.\n\"\"\"\n\n",
        line_comment(header, "#"),
        escape_lit(&ir.model_ref),
        ir.model_hash
    );
    for a in ir.all_attributes() {
        s.push_str(&format!(
            "# {}\n{}: str = \"{}\"\n",
            doc_line(&a.brief),
            const_name(&a.key),
            escape_lit(&a.key)
        ));
    }
    s
}

/// Render `attributes.ts`: `export const NAME = "key";` per attribute.
pub fn render_typescript(ir: &TelemetryIR, header: &str) -> String {
    let mut s = format!(
        "{}/**\n * OpenTelemetry attribute keys projected from model `{}`.\n \
         * Model hash: {}. Do not edit by hand.\n */\n\n",
        line_comment(header, "//"),
        escape_lit(&ir.model_ref),
        ir.model_hash
    );
    for a in ir.all_attributes() {
        s.push_str(&format!(
            "/** {} */\nexport const {} = \"{}\";\n",
            doc_line(&a.brief),
            const_name(&a.key),
            escape_lit(&a.key)
        ));
    }
    s
}

/// Re-prefix a provenance header (already `# `-commented) with `marker` so it is
/// a legal comment in the target language, preserving determinism.
fn line_comment(header: &str, marker: &str) -> String {
    let mut out = String::new();
    for line in header.lines() {
        let body = line
            .strip_prefix("# ")
            .or_else(|| line.strip_prefix('#'))
            .unwrap_or(line);
        if body.is_empty() {
            out.push_str(marker);
            out.push('\n');
        } else {
            out.push_str(marker);
            out.push(' ');
            out.push_str(body);
            out.push('\n');
        }
    }
    out
}
