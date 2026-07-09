//! Alloy formal-modeling projection: a SEA model becomes an [Alloy] model
//! (`.als`) capturing the architecture's entities, resources, and flows as
//! signatures and relational facts, so the structure can be analyzed with the
//! Alloy Analyzer (instance enumeration, assertion checking).
//!
//! Mapping:
//! - model namespace → `module <ns>`
//! - `Entity` → `sig <E> extends Principal {}`
//! - `Resource` → `sig <R> extends Resource {}`
//! - each `Flow` (`<R> from <E1> to <E2>`) → a fact asserting a `Flow` atom
//!   exists with `resource in <R>`, `from in <E1>`, `to in <E2>`.
//!
//! Output is a single `<namespace>.als`. Signatures are sorted; flows are
//! sorted by (resource, from, to), so output is byte-identical run-to-run.
//!
//! [Alloy]: https://alloytools.org/

use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use std::collections::BTreeMap;

/// Emit the Alloy model into `sink`; returns the emitted relative path.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ns = model_namespace(graph);
    let file = format!("{}.als", sanitize_filename(&ns));
    let body = build_model(graph, &ns, model_ref, &created_at);
    sink.write(&file, &body)?;
    Ok(vec![file])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_alloy_in_memory(
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

/// Build the Alloy model body.
fn build_model(graph: &Graph, ns: &str, model_ref: &str, created_at: &str) -> String {
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

    let mut entities: Vec<String> = graph
        .all_entities()
        .iter()
        .map(|e| e.name().to_string())
        .collect();
    entities.sort();
    entities.dedup();
    let mut resources: Vec<String> = graph
        .all_resources()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    resources.sort();
    resources.dedup();

    let mut flows: Vec<(String, String, String)> = Vec::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            continue;
        };
        flows.push((resource.clone(), from.clone(), to.clone()));
    }
    flows.sort();

    let mut s = String::new();
    s.push_str(&format!(
        "// Alloy model projected by DomainForge from {model_ref} at {created_at}.\n\
         // Analyze with the Alloy Analyzer (https://alloytools.org/).\n\
         module {ns}\n\n"
    ));

    s.push_str("// Principals (SEA entities)\n");
    s.push_str("abstract sig Principal {}\n");
    if entities.is_empty() {
        s.push_str("// no entities declared\n");
    } else {
        let names: Vec<String> = entities.iter().map(|e| ident(e)).collect();
        s.push_str(&format!(
            "sig {} extends Principal {{}}\n\n",
            names.join(", ")
        ));
    }

    s.push_str("// Resources that move through the cell (SEA resources)\n");
    s.push_str("abstract sig Resource {}\n");
    if resources.is_empty() {
        s.push_str("// no resources declared\n");
    } else {
        let names: Vec<String> = resources.iter().map(|r| ident(r)).collect();
        s.push_str(&format!(
            "sig {} extends Resource {{}}\n\n",
            names.join(", ")
        ));
    }

    s.push_str(
        "// A flow moves a resource from one principal to another.\n\
         sig Flow {\n  resource: one Resource,\n  from, to: one Principal\n}\n\n",
    );

    s.push_str("// Declared SEA flows (one fact each).\n");
    if flows.is_empty() {
        s.push_str("// no flows declared\n");
    }
    for (resource, from, to) in &flows {
        s.push_str(&format!(
            "fact flow_{}_{}_{} {{\n  some f: Flow | f.resource in {} and f.from in {} and f.to in {}\n}}\n",
            ident(resource),
            ident(from),
            ident(to),
            ident(resource),
            ident(from),
            ident(to)
        ));
    }
    s.push_str("\nrun {} for 3\n");
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

/// Sanitize a name into a valid Alloy identifier: leading letter/underscore,
/// alphanumerics + underscore otherwise. Names not starting with a letter get
/// an `E`/`R` safe prefix is avoided (kept simple); non-alnum → `_`.
fn ident(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if i == 0 && c.is_ascii_digit() {
                out.push('_');
            }
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("Unknown");
    }
    out
}

/// Sanitize a namespace into a filesystem-safe basename.
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
        project_alloy_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_single_namespaced_als_file() {
        let files = project(SOURCE);
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["procurement.als"]);
    }

    #[test]
    fn model_has_signatures_and_one_fact_per_flow() {
        let files = project(SOURCE);
        let body = &files["procurement.als"];
        assert!(body.starts_with("// Alloy model projected"));
        assert!(body.contains("module procurement"));
        assert!(body.contains("abstract sig Principal {}"));
        assert!(body.contains("abstract sig Resource {}"));
        assert!(body.contains("sig Buyer, Supplier extends Principal {}"));
        assert!(body.contains("sig PurchaseOrder extends Resource {}"));
        assert!(body.contains("sig Flow {"));
        assert!(body.contains("resource: one Resource"));
        assert!(body.contains("from, to: one Principal"));
        assert_eq!(body.matches("fact flow_").count(), 1);
        assert!(body.contains(
            "some f: Flow | f.resource in PurchaseOrder and f.from in Buyer and f.to in Supplier"
        ));
        assert!(body.contains("run {} for 3"));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_structure_without_signatures_or_facts() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["default.als"];
        assert!(body.contains("module default"));
        assert!(body.contains("abstract sig Principal {}"));
        assert!(body.contains("no entities declared"));
        assert!(body.contains("no resources declared"));
        assert!(body.contains("no flows declared"));
        assert_eq!(body.matches("fact flow_").count(), 0);
    }
}
