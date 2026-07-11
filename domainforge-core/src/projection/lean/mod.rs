//! Lean 4 projection: emits a self-contained, zero-dependency Lake package
//! from a DomainForge graph.
//!
//! Two-layer proof discipline: the `DomainForge` library must always compile
//! with zero `sorry` (every theorem discharged by `decide`); properties that
//! cannot be auto-grounded become documented `sorry` stubs in the separate
//! `Obligations` library. CI gates the first layer only.
//!
//! Output is byte-deterministic for a fixed `created_at`: all graph
//! collections are iterated in sorted order and no wall-clock or random data
//! enters file contents.

pub mod expr;

use crate::graph::Graph;
use crate::policy::Policy;
use crate::projection::sink::ArtifactSink;
use expr::{lower_policy, scaled, unique_idents, GroundCtx, Lowered};
use std::collections::BTreeMap;

/// Pinned Lean toolchain for emitted packages. Bump procedure: change this
/// constant, then confirm the `verify-lean` CI job (real `lake build`) is green.
pub const LEAN_TOOLCHAIN: &str = "leanprover/lean4:v4.15.0";

/// Emit the Lean package into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let model = build_model(graph)?;
    let header = header(model_ref, &created_at);

    let files: Vec<(&str, String)> = vec![
        ("lean-toolchain", format!("{LEAN_TOOLCHAIN}\n")),
        ("lakefile.toml", lakefile()),
        ("README.md", readme(model_ref, &created_at)),
        ("DomainForge.lean", root_module(&header)),
        ("DomainForge/Types.lean", types_module(&header, &model)),
        ("DomainForge/Model.lean", model_module(&header, &model)),
        (
            "DomainForge/Policies.lean",
            policies_module(&header, &model),
        ),
        ("Obligations.lean", obligations_root(&header)),
        (
            "Obligations/Stubs.lean",
            obligations_module(&header, &model),
        ),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (rel, content) in files {
        sink.write(rel, &content)?;
        written.push(rel.to_string());
    }
    Ok(written)
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_lean_in_memory(
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

struct FlowLit {
    resource: String,
    source: String,
    target: String,
    quantity: i128,
}

struct RelLit {
    subject: String,
    predicate: String,
    object: String,
}

enum PolicyLean {
    Checked {
        def_name: String,
        doc: String,
        lean: String,
        holds: bool,
    },
    Obligation {
        def_name: String,
        doc: String,
    },
}

struct Model {
    entities: Vec<String>,
    roles: Vec<String>,
    resources: Vec<String>,
    flows: Vec<FlowLit>,
    relations: Vec<RelLit>,
    scale: u32,
    policies: Vec<PolicyLean>,
}

impl Model {
    fn has_flow_types(&self) -> bool {
        !self.entities.is_empty() && !self.resources.is_empty()
    }
}

fn build_model(graph: &Graph) -> Result<Model, String> {
    // Sorted names → deterministic constructor order and id→ident maps.
    let mut entities: Vec<_> = graph.all_entities();
    entities.sort_by_key(|e| (e.name().to_string(), e.id().to_string()));
    let mut roles: Vec<_> = graph.all_roles();
    roles.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
    let mut resources: Vec<_> = graph.all_resources();
    resources.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));

    let entity_idents = unique_idents(entities.iter().map(|e| e.name()));
    let role_idents = unique_idents(roles.iter().map(|r| r.name()));
    let resource_idents = unique_idents(resources.iter().map(|r| r.name()));

    let entity_by_id: BTreeMap<String, &String> = entities
        .iter()
        .zip(&entity_idents)
        .map(|(e, ident)| (e.id().to_string(), ident))
        .collect();
    let role_by_id: BTreeMap<String, &String> = roles
        .iter()
        .zip(&role_idents)
        .map(|(r, ident)| (r.id().to_string(), ident))
        .collect();
    let resource_by_id: BTreeMap<String, &String> = resources
        .iter()
        .zip(&resource_idents)
        .map(|(r, ident)| (r.id().to_string(), ident))
        .collect();

    let mut policies: Vec<&Policy> = graph.all_policies();
    policies.sort_by_key(|p| (p.name.clone(), p.id.to_string()));

    // Shared decimal scale over every quantity in the model and its policies.
    let mut decimals: Vec<rust_decimal::Decimal> =
        graph.all_flows().iter().map(|f| f.quantity()).collect();
    for p in &policies {
        expr::collect_decimals(p.expression(), &mut decimals);
    }
    let scale = decimals
        .iter()
        .map(|d| d.normalize().scale())
        .max()
        .unwrap_or(0);

    let mut flows = Vec::new();
    for f in graph.all_flows() {
        let (Some(resource), Some(source), Some(target)) = (
            resource_by_id.get(&f.resource_id().to_string()),
            entity_by_id.get(&f.from_id().to_string()),
            entity_by_id.get(&f.to_id().to_string()),
        ) else {
            return Err(format!(
                "flow {} references an unknown entity or resource",
                f.id()
            ));
        };
        flows.push(FlowLit {
            resource: (*resource).clone(),
            source: (*source).clone(),
            target: (*target).clone(),
            quantity: scaled(f.quantity(), scale)?,
        });
    }
    flows.sort_by(|a, b| {
        (&a.resource, &a.source, &a.target, a.quantity).cmp(&(
            &b.resource,
            &b.source,
            &b.target,
            b.quantity,
        ))
    });

    let mut relations = Vec::new();
    for r in graph.all_relations() {
        let (Some(subject), Some(object)) = (
            role_by_id.get(&r.subject_role().to_string()),
            role_by_id.get(&r.object_role().to_string()),
        ) else {
            return Err(format!("relation {} references an unknown role", r.name()));
        };
        relations.push(RelLit {
            subject: (*subject).clone(),
            predicate: r.predicate().to_string(),
            object: (*object).clone(),
        });
    }
    relations.sort_by(|a, b| {
        (&a.subject, &a.predicate, &a.object).cmp(&(&b.subject, &b.predicate, &b.object))
    });

    let has_flow_types = !entity_idents.is_empty() && !resource_idents.is_empty();
    let ctx = GroundCtx {
        scale,
        flow_quantities: has_flow_types.then(|| flows.iter().map(|f| f.quantity).collect()),
    };

    let policy_idents = unique_idents(policies.iter().map(|p| p.name.as_str()));
    let lean_policies = policies
        .iter()
        .zip(&policy_idents)
        .map(|(p, ident)| {
            let doc = policy_doc(p);
            match lower_policy(p.expression(), &ctx) {
                Lowered::Groundable { lean, holds } => PolicyLean::Checked {
                    def_name: format!("policy_{ident}"),
                    doc,
                    lean,
                    holds,
                },
                Lowered::Deferred { reason } => PolicyLean::Obligation {
                    def_name: format!("obligation_{ident}"),
                    doc: obligation_doc(p, &doc, &reason),
                },
            }
        })
        .collect();

    Ok(Model {
        entities: entity_idents,
        roles: role_idents,
        resources: resource_idents,
        flows,
        relations,
        scale,
        policies: lean_policies,
    })
}

/// Doc-comment-safe text: a `-/` in user content must not close the comment.
fn doc_safe(s: &str) -> String {
    s.replace("-/", "- /")
}

fn policy_doc(p: &Policy) -> String {
    let mut doc = format!(
        "Policy `{}` ({} · {:?}/{:?}, priority {})",
        doc_safe(&p.name),
        doc_safe(&p.namespace),
        p.kind,
        p.modality,
        p.priority
    );
    if let Some(rationale) = &p.rationale {
        doc.push_str(&format!("\n    Rationale: {}", doc_safe(rationale)));
    }
    if !p.tags.is_empty() {
        doc.push_str(&format!("\n    Tags: {}", doc_safe(&p.tags.join(", "))));
    }
    doc
}

fn obligation_doc(p: &Policy, base_doc: &str, reason: &str) -> String {
    let ast = serde_json::to_string(p.expression()).unwrap_or_else(|e| format!("<{e}>"));
    format!(
        "{base_doc}\n    Not auto-groundable: {}.\n    Expression (JSON AST): {}\n    \
         Formalize the statement (replace `True`) and prove it.",
        doc_safe(reason),
        doc_safe(&ast)
    )
}

fn header(model_ref: &str, created_at: &str) -> String {
    format!(
        "/-\nGenerated by DomainForge (`--format lean`) from {}.\nCreated: {}. Do not edit by hand.\n-/\n",
        doc_safe(model_ref),
        doc_safe(created_at)
    )
}

fn lakefile() -> String {
    "name = \"domainforge\"\ndefaultTargets = [\"DomainForge\"]\n\n\
     [[lean_lib]]\nname = \"DomainForge\"\n\n\
     [[lean_lib]]\nname = \"Obligations\"\n"
        .to_string()
}

fn readme(model_ref: &str, created_at: &str) -> String {
    format!(
        "# DomainForge Lean 4 package\n\n\
         Generated from `{model_ref}` at {created_at}.\n\n\
         - `DomainForge/` — types, model facts, and machine-checked policy theorems.\n  \
         Compiles **sorry-free**: every proof is discharged by `decide` against the\n  \
         declared model. `lake build` re-verifies everything.\n\
         - `Obligations/` — proof obligations that could not be auto-grounded.\n  \
         Each `sorry` stub documents the original policy; strengthen them over time.\n\n\
         ## Check the proofs\n\n\
         ```bash\n\
         lake build            # checks the sorry-free DomainForge library\n\
         lake build Obligations # elaborates the obligation stubs (sorry warnings expected)\n\
         ```\n\n\
         The toolchain is pinned in `lean-toolchain`; `elan` fetches it automatically.\n\
         The package has zero external dependencies (no Mathlib) — builds are hermetic.\n\n\
         Quantities are exact scaled integers (`value × 10^-quantityScale`); units are\n\
         recorded in the source model and not checked here.\n"
    )
}

fn root_module(header: &str) -> String {
    format!(
        "{header}import DomainForge.Types\nimport DomainForge.Model\nimport DomainForge.Policies\n"
    )
}

fn inductive(name: &str, doc: &str, ctors: &[String]) -> String {
    let mut s = format!("/-- {doc} -/\ninductive {name} where\n");
    for c in ctors {
        s.push_str(&format!("  | {c}\n"));
    }
    s.push_str("  deriving DecidableEq, Repr\n");
    s
}

fn types_module(header: &str, m: &Model) -> String {
    let mut s = format!("{header}\nnamespace DomainForge\n\n");
    for (name, plural, ctors) in [
        ("Entity", "entities", &m.entities),
        ("Role", "roles", &m.roles),
        ("Resource", "resources", &m.resources),
    ] {
        if ctors.is_empty() {
            s.push_str(&format!("-- No {plural} declared in the model.\n\n"));
        } else {
            let doc = format!("Closed domain of {plural} declared in the model.");
            s.push_str(&inductive(name, &doc, ctors));
            s.push('\n');
        }
    }
    s.push_str("end DomainForge\n");
    s
}

fn model_module(header: &str, m: &Model) -> String {
    let mut s = format!("{header}import DomainForge.Types\n\nnamespace DomainForge\n\n");
    s.push_str(&format!(
        "/-- All quantities are exact scaled integers: `value × 10^(-quantityScale)`. -/\n\
         def quantityScale : Nat := {}\n\n",
        m.scale
    ));
    if m.has_flow_types() {
        s.push_str(
            "structure Flow where\n  resource : Resource\n  source : Entity\n  \
             target : Entity\n  quantity : Int\n  deriving DecidableEq, Repr\n\n",
        );
        s.push_str("/-- Flows declared in the model (quantities scaled by `quantityScale`). -/\n");
        if m.flows.is_empty() {
            s.push_str("def flows : List Flow := []\n\n");
        } else {
            s.push_str("def flows : List Flow := [\n");
            let rows: Vec<String> = m
                .flows
                .iter()
                .map(|f| {
                    format!(
                        "  {{ resource := .{}, source := .{}, target := .{}, quantity := {} }}",
                        f.resource, f.source, f.target, f.quantity
                    )
                })
                .collect();
            s.push_str(&rows.join(",\n"));
            s.push_str("\n]\n\n");
        }
    } else {
        s.push_str("-- No flows: the model declares no entity/resource domains.\n\n");
    }
    if !m.roles.is_empty() {
        s.push_str(
            "structure Rel where\n  subject : Role\n  predicate : String\n  object : Role\n  \
             deriving DecidableEq, Repr\n\n",
        );
        s.push_str("/-- Role-to-role relations declared in the model. -/\n");
        if m.relations.is_empty() {
            s.push_str("def relations : List Rel := []\n\n");
        } else {
            s.push_str("def relations : List Rel := [\n");
            let rows: Vec<String> = m
                .relations
                .iter()
                .map(|r| {
                    format!(
                        "  {{ subject := .{}, predicate := \"{}\", object := .{} }}",
                        r.subject,
                        r.predicate.replace('\\', "\\\\").replace('"', "\\\""),
                        r.object
                    )
                })
                .collect();
            s.push_str(&rows.join(",\n"));
            s.push_str("\n]\n\n");
        }
    }
    s.push_str("end DomainForge\n");
    s
}

fn policies_module(header: &str, m: &Model) -> String {
    let mut s = format!("{header}import DomainForge.Model\n\nnamespace DomainForge\n\n");
    let mut held: Vec<&str> = Vec::new();
    let mut any = false;
    for p in &m.policies {
        let PolicyLean::Checked {
            def_name,
            doc,
            lean,
            holds,
        } = p
        else {
            continue;
        };
        any = true;
        // `abbrev` keeps the definition reducible so `decide` sees through it.
        s.push_str(&format!(
            "/-- {doc} -/\nabbrev {def_name} : Prop := {lean}\n\n"
        ));
        if *holds {
            s.push_str(&format!(
                "/-- Machine-checked: the declared model satisfies this policy. -/\n\
                 theorem {def_name}_holds : {def_name} := by decide\n\n"
            ));
            held.push(def_name);
        } else {
            s.push_str(&format!(
                "/-- ⚠ Machine-checked: this policy is VIOLATED by the declared model. -/\n\
                 theorem {def_name}_violated : ¬ {def_name} := by decide\n\n"
            ));
        }
    }
    if held.len() >= 2 {
        s.push_str(&format!(
            "/-- The declared model jointly satisfies every machine-checked policy —\n    \
             a satisfiability witness that no two checked policies contradict. -/\n\
             theorem checked_policies_consistent : {} :=\n  ⟨{}⟩\n\n",
            held.join(" ∧ "),
            held.iter()
                .map(|h| format!("{h}_holds"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !any {
        s.push_str("-- No policies were auto-groundable; see Obligations/Stubs.lean.\n\n");
    }
    s.push_str("end DomainForge\n");
    s
}

fn obligations_root(header: &str) -> String {
    format!("{header}import Obligations.Stubs\n")
}

fn obligations_module(header: &str, m: &Model) -> String {
    let mut s = format!("{header}import DomainForge.Model\n\nnamespace DomainForge\n\n");
    let mut any = false;
    for p in &m.policies {
        let PolicyLean::Obligation { def_name, doc } = p else {
            continue;
        };
        any = true;
        s.push_str(&format!(
            "/-- {doc} -/\ntheorem {def_name} : True := by sorry\n\n"
        ));
    }
    if !any {
        s.push_str("-- All policies were machine-checked; no outstanding obligations.\n\n");
    }
    s.push_str("end DomainForge\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_to_graph;

    const FIXED_TS: &str = "2026-07-02T00:00:00+00:00";

    const SOURCE: &str = r#"
@namespace "demo"

Entity "Warehouse" in demo
Entity "Factory" in demo

Role "Operator" in demo
Role "Supervisor" in demo

Resource "CameraUnits" units in demo

Flow "CameraUnits" from "Warehouse" to "Factory" quantity 100

Relation "Oversight"
  subject: "Supervisor"
  predicate: "supervises"
  object: "Operator"

Policy positive_flow as: Flow.quantity > 0
Policy capacity_limit as: Flow.quantity < 500
Policy overload as: Flow.quantity > 1000
Policy named_entities as: Entity.name != ""
"#;

    fn project(source: &str) -> BTreeMap<String, String> {
        let graph = parse_to_graph(source).expect("fixture parses");
        project_lean_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_complete_package() {
        let files = project(SOURCE);
        let expected = [
            "DomainForge.lean",
            "DomainForge/Model.lean",
            "DomainForge/Policies.lean",
            "DomainForge/Types.lean",
            "Obligations.lean",
            "Obligations/Stubs.lean",
            "README.md",
            "lakefile.toml",
            "lean-toolchain",
        ];
        assert_eq!(files.keys().collect::<Vec<_>>(), expected.to_vec());
        assert_eq!(files["lean-toolchain"].trim(), LEAN_TOOLCHAIN);
    }

    #[test]
    fn types_and_model_are_generated() {
        let files = project(SOURCE);
        let types = &files["DomainForge/Types.lean"];
        assert!(types.contains("inductive Entity where\n  | Factory\n  | Warehouse"));
        assert!(types.contains("| Supervisor"));
        assert!(types.contains("| CameraUnits"));
        let model = &files["DomainForge/Model.lean"];
        assert!(model.contains("def quantityScale : Nat := 0"));
        assert!(model.contains(
            "{ resource := .CameraUnits, source := .Warehouse, target := .Factory, quantity := 100 }"
        ));
        assert!(model.contains(
            "{ subject := .Supervisor, predicate := \"supervises\", object := .Operator }"
        ));
    }

    #[test]
    fn groundable_policies_get_decide_proofs() {
        let files = project(SOURCE);
        let policies = &files["DomainForge/Policies.lean"];
        assert!(policies
            .contains("abbrev policy_positive_flow : Prop := ∀ f ∈ flows, (f.quantity > 0)"));
        assert!(policies
            .contains("theorem policy_positive_flow_holds : policy_positive_flow := by decide"));
        assert!(
            policies.contains("theorem policy_overload_violated : ¬ policy_overload := by decide")
        );
        assert!(policies.contains(
            "theorem checked_policies_consistent : policy_capacity_limit ∧ policy_positive_flow"
        ));
    }

    #[test]
    fn deferred_policies_become_obligations_and_checked_layer_is_sorry_free() {
        let files = project(SOURCE);
        let stubs = &files["Obligations/Stubs.lean"];
        assert!(stubs.contains("theorem obligation_named_entities : True := by sorry"));
        assert!(stubs.contains("Not auto-groundable"));
        for (path, content) in &files {
            if path.starts_with("DomainForge") {
                assert!(!content.contains("sorry"), "{path} must be sorry-free");
            }
        }
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_valid_package() {
        let files = project("@namespace \"empty\"\n");
        assert!(files["DomainForge/Types.lean"].contains("-- No entities declared"));
        assert!(files["DomainForge/Model.lean"].contains("-- No flows"));
        assert!(files["Obligations/Stubs.lean"].contains("no outstanding obligations"));
    }
}
