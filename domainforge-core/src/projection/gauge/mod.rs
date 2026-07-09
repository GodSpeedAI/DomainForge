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
    let ns = model_namespace(graph);
    let file = format!("{}.spec", sanitize_filename(&ns));
    let body = build_spec(graph, &ns, model_ref, &created_at);
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
fn build_spec(graph: &Graph, ns: &str, model_ref: &str, created_at: &str) -> String {
    let entity_name: BTreeMap<String, String> = graph
        .all_entities()
        .iter()
        .map(|e| (e.id().to_string(), e.name().to_string()))
        .collect();
    let resource_name: BTreeMap<String, String> = graph
        .all_resources()
        .iter()
        .map(|r| (r.id().to_string(), r.name().to_string()))
        .collect();

    let mut flows: Vec<(String, String, String, String)> = Vec::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            continue;
        };
        flows.push((
            resource.clone(),
            from.clone(),
            to.clone(),
            f.quantity().to_string(),
        ));
    }
    flows.sort();

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
    for (resource, from, to, qty) in &flows {
        s.push_str(&format!(
            "## Issue {resource} from {from} to {to}\n\n\
             * Given a \"{from}\" entity exists\n\
             * When \"{from}\" issues \"{resource}\" (quantity \"{qty}\") to \"{to}\"\n\
             * Then \"{to}\" should hold \"{qty}\" \"{resource}\"\n\n"
        ));
    }
    s
}

/// Derive a single namespace: first entity's namespace (sorted by name), else
/// first resource's, else `"default"`.
fn model_namespace(graph: &Graph) -> String {
    let mut ents = graph.all_entities();
    ents.sort_by_key(|e| e.name().to_string());
    if let Some(e) = ents.first() {
        return e.namespace().to_string();
    }
    let mut res = graph.all_resources();
    res.sort_by_key(|r| r.name().to_string());
    if let Some(r) = res.first() {
        return r.namespace().to_string();
    }
    "default".to_string()
}

/// Sanitize a namespace into a filesystem-safe spec basename: keep
/// alphanumerics and `-`, `_`, `.`; replace everything else with `_`.
fn sanitize_filename(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("default");
    }
    out
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
}
