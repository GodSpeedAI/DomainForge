//! TLA+ formal-specification projection: a SEA model becomes a [TLA+] spec
//! describing the architecture's flows as a state machine, so the behavior
//! can be model-checked with TLC (deadlock/invariant checking).
//!
//! Mapping:
//! - model namespace → `MODULE <ns>`
//! - each `Entity` + `Resource` → a `CONSTANT` (the actors and the movable
//!   resources)
//! - `VARIABLES holder` — a function from each flow-bearing resource to its
//!   current holder
//! - `Init` — every flow-bearing resource starts held by its first flow's
//!   source entity
//! - each `Flow` (`<R> from <E1> to <E2>`) → a `Transfer<R>` action guarding
//!   `holder[R] = E1` and setting `holder' = [holder EXCEPT ![R] = E2]`
//! - `Next` — disjunction of all transfer actions; `Spec == Init /\ [][Next]_holder`
//! - `TypeInvariant` — holder always maps resources into the entity set
//!
//! Output is two files: `<ns>.tla` (the spec) and `<ns>.cfg` (TLC config
//! assigning each CONSTANT a model value and binding SPECIFICATION + INVARIANT).
//! Sorted for byte-identical output.
//!
//! [TLA+]: https://lamport.org/tla/tla.html

use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use std::collections::BTreeMap;

/// Emit the TLA+ spec + config into `sink`; returns emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ns = model_namespace(graph);
    let base = sanitize_filename(&ns);
    let tla_file = format!("{base}.tla");
    let cfg_file = format!("{base}.cfg");

    let (entities, resources, flows) = collect(graph);

    let tla = build_tla(&base, &entities, &resources, &flows, model_ref, &created_at);
    let cfg = build_cfg(&entities, &resources);

    sink.write(&tla_file, &tla)?;
    sink.write(&cfg_file, &cfg)?;
    Ok(vec![tla_file, cfg_file])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_tla_in_memory(
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

type Flow = (String, String, String); // (resource, from, to)

fn collect(graph: &Graph) -> (Vec<String>, Vec<String>, Vec<Flow>) {
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

    let mut flows: Vec<Flow> = Vec::new();
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
    (entities, resources, flows)
}

/// The set of resources that appear in at least one flow (movable resources).
fn movable_resources(flows: &[Flow]) -> Vec<String> {
    let mut v: Vec<String> = flows.iter().map(|(r, _, _)| r.clone()).collect();
    v.sort();
    v.dedup();
    v
}

fn build_tla(
    module: &str,
    entities: &[String],
    resources: &[String],
    flows: &[Flow],
    model_ref: &str,
    created_at: &str,
) -> String {
    let movable = movable_resources(flows);
    let eid = |s: &str| -> String { ident(s) };
    let set = |xs: &[String]| -> String {
        if xs.is_empty() {
            "{}".to_string()
        } else {
            format!(
                "{{{}}}",
                xs.iter().map(|x| eid(x)).collect::<Vec<_>>().join(", ")
            )
        }
    };

    let mut s = String::new();
    s.push_str(&format!(
        "(* TLA+ spec projected by DomainForge from {model_ref} at {created_at}. *)\n\
         (* Model-check with TLC: tlc {module}.tla. *)\n\
         ---------------------------- MODULE {module} ----------------------------\n\
         EXTENDS Naturals, Sequences, TLC\n\n"
    ));

    s.push_str("(* SEA entities + resources, bound in the .cfg. *)\nCONSTANTS ");
    let const_names: Vec<String> = entities
        .iter()
        .chain(resources.iter())
        .map(|x| eid(x))
        .collect();
    if const_names.is_empty() {
        s.push_str("None\n");
    } else {
        s.push_str(&const_names.join(", "));
        s.push('\n');
    }
    s.push('\n');

    s.push_str("(* Entity and resource sets. *)\n");
    s.push_str(&format!("Entities == {}\n", set(entities)));
    s.push_str(&format!("Resources == {}\n\n", set(resources)));

    s.push_str("(* holder[r] = the entity currently holding resource r. *)\n");
    s.push_str("VARIABLES holder\n\n");

    // Init: each movable resource starts at its first flow's from-entity.
    s.push_str("Init ==\n  /\\ holder = [");
    if movable.is_empty() {
        s.push_str("r \\in {} |-> ChooseEntity");
    } else {
        s.push_str("r \\in Resources |-> CASE r");
        for (i, r) in movable.iter().enumerate() {
            let from = flows
                .iter()
                .find(|(fr, _, _)| fr == r)
                .map(|(_, f, _)| f.clone())
                .unwrap_or_else(|| entities.first().cloned().unwrap_or_else(|| "None".into()));
            let sep = if i == 0 { " " } else { " [] " };
            s.push_str(&format!("{sep}= {} -> {}", eid(r), eid(&from)));
        }
        // ponytail: CASE without exhaustive arms; non-movable resources keep an
        // arbitrary entity via OTHERWISE — fine since no action touches them.
        let fallback = entities.first().cloned().unwrap_or_else(|| "None".into());
        s.push_str(&format!(" [] OTHERWISE {}", eid(&fallback)));
    }
    s.push_str("]\n\n");

    // One transfer action per declared flow.
    s.push_str("(* One transfer action per declared SEA flow. *)\n");
    if flows.is_empty() {
        s.push_str("(* no flows declared *)\n");
    }
    let mut by_resource: BTreeMap<String, Vec<&Flow>> = BTreeMap::new();
    for f in flows {
        by_resource.entry(f.0.clone()).or_default().push(f);
    }
    for (resource, group) in &by_resource {
        s.push_str(&format!("Transfer{} ==\n", eid(resource)));
        for (_, from, to) in group {
            s.push_str(&format!(
                "  /\\ holder[{}] = {}\n  /\\ holder' = [holder EXCEPT ![{}] = {}]\n",
                eid(resource),
                eid(from),
                eid(resource),
                eid(to)
            ));
        }
        s.push('\n');
    }

    s.push_str("Next == ");
    if flows.is_empty() {
        s.push_str("FALSE\n\n");
    } else {
        let transfers: Vec<String> = by_resource
            .keys()
            .map(|r| format!("Transfer{}", eid(r)))
            .collect();
        s.push_str(&transfers.join(" \\/ "));
        s.push('\n');
    }
    s.push('\n');

    s.push_str("Spec == Init /\\ [][Next]_holder\n\n");
    s.push_str("(* Type invariant: holder always maps resources into Entities. *)\n");
    s.push_str("TypeInvariant == holder \\in [Resources -> Entities]\n\n");
    s.push_str("=============================================================================\n");
    s
}

fn build_cfg(entities: &[String], resources: &[String]) -> String {
    let mut s = String::new();
    s.push_str("# TLC config projected by DomainForge.\n");
    s.push_str("CONSTANTS\n");
    if entities.is_empty() && resources.is_empty() {
        s.push_str("# none\n");
    } else {
        for x in entities.iter().chain(resources.iter()) {
            let i = ident(x);
            s.push_str(&format!("{i} = {i}\n"));
        }
    }
    s.push('\n');
    s.push_str("SPECIFICATION Spec\n");
    s.push_str("INVARIANT TypeInvariant\n");
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

/// Sanitize a name into a valid TLA+ identifier: alphanumerics + underscore;
/// non-alnum -> `_`. Leading digit -> prefix `_`.
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
        project_tla_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_tla_and_cfg() {
        let files = project(SOURCE);
        assert_eq!(
            files.keys().collect::<Vec<_>>(),
            vec!["procurement.cfg", "procurement.tla"]
        );
    }

    #[test]
    fn spec_has_module_init_next_and_one_action_per_flow() {
        let files = project(SOURCE);
        let body = &files["procurement.tla"];
        assert!(body.contains("MODULE procurement"));
        assert!(body.contains("EXTENDS Naturals, Sequences, TLC"));
        assert!(body.contains("CONSTANTS Buyer, Supplier, PurchaseOrder"));
        assert!(body.contains("VARIABLES holder"));
        assert!(body.contains("Init =="));
        assert!(body.contains("TransferPurchaseOrder =="));
        assert!(body.contains("holder[PurchaseOrder] = Buyer"));
        assert!(body.contains("[holder EXCEPT ![PurchaseOrder] = Supplier]"));
        assert!(body.contains("Next == TransferPurchaseOrder"));
        assert!(body.contains("Spec == Init /\\ [][Next]_holder"));
        assert!(body.contains("TypeInvariant =="));
        assert!(body.contains("====="));
    }

    #[test]
    fn cfg_binds_constants_and_spec() {
        let files = project(SOURCE);
        let cfg = &files["procurement.cfg"];
        assert!(cfg.contains("CONSTANTS"));
        assert!(cfg.contains("Buyer = Buyer"));
        assert!(cfg.contains("PurchaseOrder = PurchaseOrder"));
        assert!(cfg.contains("SPECIFICATION Spec"));
        assert!(cfg.contains("INVARIANT TypeInvariant"));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_valid_structure() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["default.tla"];
        assert!(body.contains("MODULE default"));
        assert!(body.contains("Next == FALSE"));
        assert!(body.contains("Init =="));
        assert!(body.contains("Spec =="));
    }
}
