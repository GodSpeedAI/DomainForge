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
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{sanitize_filename, NameRegistrar};
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
    let ns = model_namespace(graph)?;
    let file = format!("{}.als", sanitize_filename(&ns));
    let body = build_model(graph, &ns, model_ref, &created_at)?;
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
fn build_model(
    graph: &Graph,
    ns: &str,
    model_ref: &str,
    created_at: &str,
) -> Result<String, String> {
    let mut reg = NameRegistrar::new();
    // M2: sanitize the namespace for the module header — Alloy requires a
    // valid module name (identifier), so `module supply chain` must become
    // `module supply_chain`.
    let ns_ident = reg.register("ident", ns);

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

    let flows = collect_flows(graph)?; // M4: loud dangling-ref policy

    // Collision-safe sig names (M1): entities/resources that collide after
    // sanitization get a hash suffix.
    let ent_ids: Vec<String> = entities.iter().map(|e| reg.register("ident", e)).collect();
    let res_ids: Vec<String> = resources.iter().map(|r| reg.register("ident", r)).collect();

    let ent_lookup: std::collections::BTreeMap<&str, &str> = entities
        .iter()
        .zip(ent_ids.iter())
        .map(|(e, id)| (e.as_str(), id.as_str()))
        .collect();
    let res_lookup: std::collections::BTreeMap<&str, &str> = resources
        .iter()
        .zip(res_ids.iter())
        .map(|(r, id)| (r.as_str(), id.as_str()))
        .collect();
    let mut id_of = |name: &str| -> String {
        ent_lookup
            .get(name)
            .or_else(|| res_lookup.get(name))
            .map(|s| s.to_string())
            .unwrap_or_else(|| reg.register("ident", name))
    };

    let mut s = String::new();
    s.push_str(&format!(
        "// Alloy model projected by DomainForge from {model_ref} at {created_at}.\n\
         // Analyze with the Alloy Analyzer (https://alloytools.org/).\n\
         module {ns_ident}\n\n"
    ));

    s.push_str("// Principals (SEA entities)\n");
    s.push_str("abstract sig Principal {}\n");
    if ent_ids.is_empty() {
        s.push_str("// no entities declared\n");
    } else {
        s.push_str(&format!(
            "sig {} extends Principal {{}}\n\n",
            ent_ids.join(", ")
        ));
    }

    s.push_str("// Resources that move through the cell (SEA resources)\n");
    s.push_str("abstract sig Resource {}\n");
    if res_ids.is_empty() {
        s.push_str("// no resources declared\n");
    } else {
        s.push_str(&format!(
            "sig {} extends Resource {{}}\n\n",
            res_ids.join(", ")
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
    for f in &flows {
        let r = id_of(&f.resource);
        let from = id_of(&f.from);
        let to = id_of(&f.to);
        let fact_name = format!("flow_{}_{}_{}", r, from, to);
        s.push_str(&format!(
            "fact {fact_name} {{\n  some f: Flow | f.resource in {r} and f.from in {from} and f.to in {to}\n}}\n"
        ));
    }
    // M3: scope the `run` command to accommodate models with >3 flows. Each
    // flow fact may require a distinct Flow atom; capping at 3 makes models
    // with ≥4 flows report "no instance found" (false inconsistency). Scale
    // the scope to max(3, flow_count).
    let scope = flows.len().max(3);
    s.push_str(&format!("\nrun {{}} for {scope}\n"));
    Ok(s)
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

    /// M3 regression: a model with >3 flows must scale the run scope so the
    /// facts remain satisfiable. Capping at 3 makes a 4-flow model report
    /// "no instance found" (false inconsistency).
    #[test]
    fn run_scope_scales_with_flow_count() {
        let many_flows = r#"
@namespace "many"
Entity "A" in many
Entity "B" in many
Entity "C" in many
Entity "D" in many
Entity "E" in many
Resource "R1" units in many
Resource "R2" units in many
Resource "R3" units in many
Resource "R4" units in many
Flow "R1" from "A" to "B" quantity 1
Flow "R2" from "B" to "C" quantity 1
Flow "R3" from "C" to "D" quantity 1
Flow "R4" from "D" to "E" quantity 1
"#;
        let files = project(many_flows);
        let body = &files["many.als"];
        // 4 flows → scope must be at least 4, not capped at 3.
        assert!(
            body.contains("run {} for 4"),
            "run scope must scale to flow count (M3): {}",
            body.lines().find(|l| l.starts_with("run")).unwrap_or("")
        );
    }

    /// M2 regression: a namespace with spaces must be sanitized in the module
    /// header (`module supply_chain`, not `module supply chain`).
    #[test]
    fn module_header_sanitizes_namespace() {
        // Entity uses the default @namespace (no `in` clause needed).
        let files = project("@namespace \"supply chain\"\nEntity \"E\"\n");
        let body = &files["supply_chain.als"];
        assert!(
            body.contains("module supply_chain"),
            "module header must sanitize the namespace (M2)"
        );
        assert!(
            !body.contains("module supply chain"),
            "module header must not contain raw namespace with spaces (M2)"
        );
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
