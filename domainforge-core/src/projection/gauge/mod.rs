//! Gauge verification projection: a SEA model becomes a Gauge `.spec`
//! (Markdown) so the architecture's flows are expressed as executable test
//! scenarios that a test author wires step implementations to.
//!
//! Mapping:
//! - model namespace → the spec name (`# <Namespace> flows`)
//! - each `Flow` (`<Resource> from <From> to <To>`) → a `## Issue <Resource>
//!   from <From> to <To>` scenario with Given/When/Then steps whose parameters
//!   are quoted (Gauge reserves `"`, `<`, `>` — this projection never emits
//!   raw `<`/`>` in step text).
//!
//! Output is a single `<namespace>.spec` file. Flows are sorted by
//! (resource, from, to), so output is byte-identical run-to-run.
//!
//! [Gauge]: https://docs.gauge.org/writing-specifications/

use crate::graph::Graph;
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{sanitize_filename, NameRegistrar};
use crate::projection::sink::ArtifactSink;
use std::collections::BTreeMap;

/// Emit the Gauge spec into `sink`; returns the emitted relative path.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ns = model_namespace(graph)?;
    let file = format!("{}.spec", sanitize_filename(&ns));
    let body = build_spec(graph, &ns, model_ref, &created_at)?;
    sink.write(&file, &body)?;
    Ok(vec![file])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_gauge_in_memory(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(graph, model_ref, created_at, &mut sink)?;
    Ok(map)
}

/// Build the Gauge spec body.
fn build_spec(
    graph: &Graph,
    ns: &str,
    model_ref: &str,
    created_at: &str,
) -> Result<String, String> {
    let flows = collect_flows(graph)?; // M4: loud dangling-ref policy

    let mut s = String::new();
    s.push_str(&format!(
        "# {ns} flows\n\n\
         // Gauge spec projected by DomainForge from {model_ref} at {created_at}.\n\
         // One scenario per SEA Flow. Wire step implementations to these step\n\
         // texts; parameters are quoted.\n\n"
    ));
    if flows.is_empty() {
        s.push_str("// No flows declared — nothing to verify.\n");
    }
    // Collision-safe scenario headings: two flows that produce the same heading
    // `## Issue R from A to B` get a hash suffix (Gauge requires unique scenario
    // names per spec file) — M1.
    let mut reg = NameRegistrar::new();
    for f in &flows {
        // M2: escape names embedded in step text so `"` in a name doesn't
        // corrupt step-parameter quoting, and `<`/`>` never appear raw.
        let heading = format!(
            "Issue {} from {} to {}",
            gauge_escape(&f.resource),
            gauge_escape(&f.from),
            gauge_escape(&f.to)
        );
        let unique_heading = reg.register("slug", &heading);
        let h_resource = gauge_escape(&f.resource);
        let h_from = gauge_escape(&f.from);
        let h_to = gauge_escape(&f.to);
        let h_qty = gauge_escape(&f.quantity);
        // Strip any hash suffix from the display (the suffix is part of the
        // registered slug, which lowercases — but the heading must preserve
        // the readable form). We keep the readable form and append a numeric
        // disambiguator only when needed.
        let display = if unique_heading != slug(&heading) {
            // Collision occurred: append the suffix to the readable heading.
            let suffix = unique_heading.strip_prefix(&slug(&heading)).unwrap_or("");
            format!("{heading}{suffix}")
        } else {
            heading
        };
        s.push_str(&format!(
            "## {display}\n\n\
             * Given a \"{h_from}\" entity exists\n\
             * When \"{h_from}\" issues \"{h_resource}\" (quantity \"{h_qty}\") to \"{h_to}\"\n\
             * Then \"{h_to}\" should hold \"{h_qty}\" \"{h_resource}\"\n\n"
        ));
    }
    Ok(s)
}

/// Escape a name for safe embedding in Gauge step text: replace `"` (corrupts
/// parameter quoting) and `<`/`>` (reserved by Gauge for dynamic params) with
/// `_`. M2.
fn gauge_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' | '<' | '>' => '_',
            _ => c,
        })
        .collect()
}

/// Lowercase slug (re-exported kernel helper for the heading dedup).
fn slug(s: &str) -> String {
    crate::projection::ids::slug(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_to_graph;

    const FIXED_TS: &str = "2026-07-02T00:00:00+00:00";

    const SOURCE: &str = r#"
@namespace "procurement"

Entity "Buyer" in procurement
Entity "Supplier" in procurement
Resource "PurchaseOrder" units in procurement

Flow "PurchaseOrder" from "Buyer" to "Supplier" quantity 1
"#;

    fn project(source: &str) -> BTreeMap<String, String> {
        let graph = parse_to_graph(source).expect("fixture parses");
        project_gauge_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_single_namespaced_spec_file() {
        let files = project(SOURCE);
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["procurement.spec"]);
    }

    #[test]
    fn spec_has_heading_and_one_scenario_per_flow() {
        let files = project(SOURCE);
        let body = &files["procurement.spec"];
        assert!(body.starts_with("# procurement flows\n"));
        assert_eq!(body.matches("## ").count(), 1, "one scenario per flow");
        assert!(body.contains("## Issue PurchaseOrder from Buyer to Supplier"));
        assert!(body.contains("* Given a \"Buyer\" entity exists"));
        assert!(body.contains(
            "* When \"Buyer\" issues \"PurchaseOrder\" (quantity \"1\") to \"Supplier\""
        ));
        assert!(body.contains("* Then \"Supplier\" should hold \"1\" \"PurchaseOrder\""));
    }

    #[test]
    fn step_text_uses_no_reserved_angle_brackets() {
        let files = project(SOURCE);
        let body = &files["procurement.spec"];
        // Gauge reserves '<' and '>' for dynamic params; steps must not contain them raw.
        for line in body.lines().filter(|l| l.starts_with("* ")) {
            assert!(
                !line.contains('<') && !line.contains('>'),
                "reserved char in step: {line}"
            );
        }
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_no_scenarios() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["default.spec"];
        assert!(body.starts_with("# default flows\n"));
        assert_eq!(body.matches("## ").count(), 0);
        assert!(body.contains("No flows declared"));
    }

    /// M2 regression: names with `<`/`>`/`"` must be sanitized in step text.
    /// Gauge reserves `<`/`>` for dynamic params; `"` corrupts param quoting.
    #[test]
    fn hostile_names_sanitized_in_step_text() {
        let hostile = r#"
@namespace "h"
Entity "x<y>" in h
Entity "a\"b" in h
Resource "R" units in h
Flow "R" from "x<y>" to "a\"b" quantity 1
"#;
        let files = project(hostile);
        let body = &files["h.spec"];
        for line in body.lines().filter(|l| l.starts_with("* ")) {
            assert!(
                !line.contains('<') && !line.contains('>'),
                "reserved angle bracket in step (M2): {line}"
            );
        }
        // The `"` in entity name must be replaced with `_` (sanitized).
        assert!(
            !body.contains("a\"b"),
            "raw quote must not appear in step text (M2)"
        );
    }
}
