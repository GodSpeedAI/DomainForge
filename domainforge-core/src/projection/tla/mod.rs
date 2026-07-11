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
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{sanitize_filename, NameRegistrar};
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
    let ns = model_namespace(graph)?;
    let base = sanitize_filename(&ns);
    let tla_file = format!("{base}.tla");
    let cfg_file = format!("{base}.cfg");

    let flows = collect_flows(graph)?;

    let entities: Vec<String> = sorted_unique_names(graph.all_entities().iter().map(|e| e.name()));
    let resources: Vec<String> =
        sorted_unique_names(graph.all_resources().iter().map(|r| r.name()));

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

fn sorted_unique_names<'a>(names: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut v: Vec<String> = names.map(String::from).collect();
    v.sort();
    v.dedup();
    v
}

/// Build the TLA+ spec body.
///
/// Each flow becomes its own action `Transfer<Resource>_<From>_<To>` (H2):
/// one action per flow, never conjoined, so chained flows (R: A→B, R: B→C)
/// produce two independently-enabled actions rather than a contradictory
/// conjunction. `Next` is the disjunction of all of them.
///
/// `Init` uses a CASE with the correct `CASE r = X -> F1 [] r = Y -> F2 []
/// OTHER -> Fallback` syntax (H1): every arm carries its discriminant, and
/// the default-case keyword is `OTHER` (not `OTHERWISE`).
fn build_tla(
    module: &str,
    entities: &[String],
    resources: &[String],
    flows: &[crate::projection::flows::ResolvedFlow],
    model_ref: &str,
    created_at: &str,
) -> String {
    // Collision-safe identifier registrar: entities/resources with names that
    // collapse to the same identifier get a hash suffix so the TLA+ spec never
    // has duplicate CONSTANTS (M1).
    let mut reg = NameRegistrar::new();
    // Pre-register entities first, then resources, so ident assignment is
    // deterministic and stable across runs.
    let ent_ids: Vec<String> = entities.iter().map(|e| reg.register("ident", e)).collect();
    let res_ids: Vec<String> = resources.iter().map(|r| reg.register("ident", r)).collect();

    let movable: Vec<&str> = {
        let mut seen: Vec<&str> = flows.iter().map(|f| f.resource.as_str()).collect();
        seen.sort();
        seen.dedup();
        seen
    };

    let set_str = |ids: &[String]| -> String {
        if ids.is_empty() {
            "{}".to_string()
        } else {
            format!("{{{}}}", ids.join(", "))
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
    let const_names: Vec<String> = ent_ids.iter().chain(res_ids.iter()).cloned().collect();
    if const_names.is_empty() {
        s.push_str("ModelValue\n");
    } else {
        s.push_str(&const_names.join(", "));
        s.push('\n');
    }
    s.push('\n');

    // Build a lookup from original name → registered id for entities/resources.
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

    s.push_str("(* Entity and resource sets. *)\n");
    s.push_str(&format!("Entities == {}\n", set_str(&ent_ids)));
    s.push_str(&format!("Resources == {}\n\n", set_str(&res_ids)));

    s.push_str("(* holder[r] = the entity currently holding resource r. *)\n");
    s.push_str("VARIABLES holder\n\n");

    // Init: each movable resource starts at its first flow's from-entity.
    // CASE syntax: every arm carries the discriminant `r = X`; the default is
    // `OTHER -> Fallback` (H1: was `= X` missing on arms 2+, and `OTHERWISE`).
    s.push_str("Init ==\n  /\\ holder = [");
    if movable.is_empty() {
        let fallback = entities
            .first()
            .map(|e| id_of(e))
            .unwrap_or_else(|| "ModelValue".into());
        s.push_str("r \\in Resources |-> ");
        s.push_str(&fallback);
    } else {
        s.push_str("r \\in Resources |-> CASE");
        for (i, r) in movable.iter().enumerate() {
            let from = flows
                .iter()
                .find(|f| f.resource == *r)
                .map(|f| f.from.as_str())
                .unwrap_or_else(|| entities.first().map(|e| e.as_str()).unwrap_or("ModelValue"));
            let sep = if i == 0 { " " } else { " [] " };
            // H1 fix: every arm carries the discriminant `r = <Resource>`.
            let r_id = id_of(r);
            let from_id = id_of(from);
            s.push_str(&format!("{sep}r = {r_id} -> {from_id}"));
        }
        let fallback = entities
            .first()
            .map(|e| id_of(e))
            .unwrap_or_else(|| "ModelValue".into());
        // H1 fix: default-case keyword is `OTHER`, not `OTHERWISE`.
        s.push_str(&format!(" [] OTHER -> {fallback}"));
    }
    s.push_str("]\n\n");

    // H2 fix: one action per flow (Transfer<Resource>_<From>_<To>), never
    // conjoined. Chained flows R: A→B and R: B→C produce two independently
    // enabled actions rather than a contradictory conjunction.
    s.push_str("(* One transfer action per declared SEA flow. *)\n");
    if flows.is_empty() {
        s.push_str("(* no flows declared *)\n");
    }
    let mut action_names: Vec<String> = Vec::new();
    for f in flows {
        let r = id_of(&f.resource);
        let from = id_of(&f.from);
        let to = id_of(&f.to);
        let action = format!("Transfer{r}_{from}_{to}");
        action_names.push(action.clone());
        s.push_str(&format!(
            "{action} ==\n  /\\ holder[{r}] = {from}\n  /\\ holder' = [holder EXCEPT ![{r}] = {to}]\n\n"
        ));
    }

    s.push_str("Next == ");
    if action_names.is_empty() {
        s.push_str("FALSE\n\n");
    } else {
        s.push_str(&action_names.join(" \\/ "));
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
    let mut reg = NameRegistrar::new();
    let mut s = String::new();
    // L3 fix: TLC .cfg comments use `\*` (line) and `(* *)` (block), not `#`.
    s.push_str("\\* TLC config projected by DomainForge.\n");
    s.push_str("CONSTANTS\n");
    if entities.is_empty() && resources.is_empty() {
        s.push_str("\\* none\n");
    } else {
        for x in entities.iter().chain(resources.iter()) {
            let i = reg.register("ident", x);
            s.push_str(&format!("{i} = {i}\n"));
        }
    }
    s.push('\n');
    s.push_str("SPECIFICATION Spec\n");
    s.push_str("INVARIANT TypeInvariant\n");
    s
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
        // H2: one action per flow, named Transfer<Resource>_<From>_<To>.
        assert!(body.contains("TransferPurchaseOrder_Buyer_Supplier =="));
        assert!(body.contains("holder[PurchaseOrder] = Buyer"));
        assert!(body.contains("[holder EXCEPT ![PurchaseOrder] = Supplier]"));
        assert!(body.contains("Next == TransferPurchaseOrder_Buyer_Supplier"));
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

    /// H1 regression: the CASE in Init must carry the discriminant `r = X` on
    /// every arm and use `OTHER` (not `OTHERWISE`) for the default. A pre-H1
    /// spec had `[] = PurchaseOrder` (no `r`) and `OTHERWISE`.
    #[test]
    fn init_case_has_discriminant_on_every_arm_and_uses_other() {
        let files = project(SOURCE);
        let body = &files["procurement.tla"];
        // The Init line must contain `CASE r = PurchaseOrder -> Buyer`.
        assert!(
            body.contains("CASE r = PurchaseOrder -> Buyer"),
            "Init CASE missing discriminant on first arm"
        );
        // Must NOT contain the broken `[] = ` (missing discriminant) form.
        assert!(
            !body.contains("[] = "),
            "Init CASE arm missing discriminant (H1 regression)"
        );
        // Default-case keyword must be `OTHER`, not `OTHERWISE`.
        assert!(
            body.contains("[] OTHER -> "),
            "Init CASE must use OTHER as the default-case keyword"
        );
        assert!(
            !body.contains("OTHERWISE"),
            "Init CASE must not use OTHERWISE (H1 regression)"
        );
    }

    /// H2 regression: chained flows (R: A→B and R: B→C) must produce two
    /// separate actions, not a single conjoined `Transfer<R>` that is never
    /// enabled. The pre-H2 code AND-ed both flows' guards + primed
    /// assignments into one action → immediate deadlock.
    #[test]
    fn chained_flows_produce_separate_non_conjoined_actions() {
        let chained = r#"
@namespace "chain"
Entity "A" in chain
Entity "B" in chain
Entity "C" in chain
Resource "R" units in chain

Flow "R" from "A" to "B" quantity 1
Flow "R" from "B" to "C" quantity 1
"#;
        let files = project(chained);
        let body = &files["chain.tla"];
        // Two distinct actions, one per flow.
        assert!(
            body.contains("TransferR_A_B =="),
            "missing action for flow R: A→B"
        );
        assert!(
            body.contains("TransferR_B_C =="),
            "missing action for flow R: B→C"
        );
        // Next must be a disjunction of both.
        assert!(body.contains("Next == TransferR_A_B \\/ TransferR_B_C"));
        // Must NOT have a single conjoined TransferR action with two primed
        // assignments (the H2 bug).
        assert!(
            !body.lines().any(|l| l.starts_with("TransferR ==")),
            "chained flows must not be conjoined into a single TransferR (H2 regression)"
        );
    }

    /// L4 regression: an empty model's Init must not reference undefined
    /// `ChooseEntity`; it must use a valid constant or no CASE at all.
    #[test]
    fn empty_model_init_does_not_reference_undefined_operator() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["default.tla"];
        assert!(
            !body.contains("ChooseEntity"),
            "empty-model Init must not reference undefined ChooseEntity (L4)"
        );
    }
}
