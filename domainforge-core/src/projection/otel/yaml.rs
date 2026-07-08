//! OpenTelemetry semantic-convention registry renderer.
//!
//! Emits a deterministic YAML registry (`registry/telemetry.yaml`) describing
//! the attribute groups and span-kind suggestions in a [`TelemetryIR`]. The
//! shape is a small, fixed subset of the SemConv registry schema (groups with
//! `type: attribute_group` carrying attribute definitions, and groups with
//! `type: span` referencing them), validated in CI against
//! `schemas/otel/semconv_registry.schema.json`.
//!
//! No YAML crate is pulled in: the shape is small and fixed, so this
//! hand-rolled emitter (which quotes every scalar and iterates the IR's already
//! sorted collections) is both smaller and guaranteed byte-deterministic.

use super::ir::TelemetryIR;

/// Render the registry YAML for `ir`. `header` is a provenance comment block.
pub fn render(ir: &TelemetryIR, header: &str) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("groups:\n");

    // Attribute-definition groups first (already sorted by id in the IR).
    for g in &ir.groups {
        s.push_str(&format!("  - id: {}\n", quote(&g.id)));
        s.push_str("    type: attribute_group\n");
        s.push_str(&format!("    brief: {}\n", quote(&g.brief)));
        s.push_str("    attributes:\n");
        for a in &g.attributes {
            s.push_str(&format!("      - id: {}\n", quote(&a.key)));
            s.push_str(&format!("        type: {}\n", a.r#type.as_str()));
            s.push_str("        stability: development\n");
            s.push_str(&format!("        brief: {}\n", quote(&a.brief)));
        }
    }

    // Span-kind suggestion groups (referencing the attributes above).
    for span in &ir.spans {
        s.push_str(&format!("  - id: {}\n", quote(&span.id)));
        s.push_str("    type: span\n");
        s.push_str(&format!("    span_kind: {}\n", span.span_kind.as_str()));
        s.push_str(&format!("    brief: {}\n", quote(&span.brief)));
        s.push_str("    attributes:\n");
        for r in &span.attribute_refs {
            s.push_str(&format!("      - ref: {}\n", quote(r)));
        }
    }

    s
}

/// Double-quote a YAML scalar, escaping `\` and `"`. The IR's strings never
/// contain newlines, so no block-scalar handling is needed.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}
