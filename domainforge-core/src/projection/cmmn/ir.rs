//! CMMN 1.1 case IR: a graph-derived, renderer-agnostic model of one case.
//!
//! Mapping (declared v1 subset — see `docs/cmmn-projections.md`):
//! - **case**              — one per model.
//! - **case file item**    — one per declared resource, each backed by a
//!   `caseFileItemDefinition` at the `definitions` level.
//! - **case role**         — one per declared role; referenced by a human task's
//!   `performerRef` when the graph binds the role to that task's entity.
//! - **human task**        — one per entity (the work performed in the case);
//!   `performerRef` names the entity's first (sorted) bound role, if any.
//! - **milestone**         — one per authority policy: the achievement
//!   "policy satisfied".
//! - **sentry / entry criterion** — one per policy *condition*. Each milestone's
//!   plan item carries an `entryCriterion` referencing a `sentry` whose `ifPart`
//!   condition is the policy's expression rendered as text. This is the
//!   policy→sentry lowering the projection exists to prove.
//! - **plan item**         — one per human task and one per milestone, wiring the
//!   plan-item definitions into the case plan and (for milestones) holding the
//!   entry criterion.
//!
//! **Non-goal (v1):** event listeners. CMMN `userEventListener` /
//! `timerEventListener` model *runtime* events, and the canonical model carries
//! no runtime-event source, so none are emitted (see the doc).
//!
//! Every id is minted through [`crate::projection::ids::element_id`] under the
//! `cmmn` family and prefixed with an alpha tag so it is a legal XML `NCName`.
//! All collections are built and emitted in sorted (`BTreeMap`/`BTreeSet`)
//! order, so output is byte-identical run-to-run.

use crate::graph::Graph;
use crate::projection::ids::element_id;
use std::collections::BTreeMap;

/// A `caseFileItemDefinition` (definitions-level, reusable) derived from a
/// resource. Referenced by a [`CaseFileItem`] via `definitionRef`.
#[derive(Debug, Clone)]
pub struct CaseFileItemDef {
    pub id: String,
    pub name: String,
}

/// A `caseFileItem` inside the case file, derived from a declared resource.
#[derive(Debug, Clone)]
pub struct CaseFileItem {
    pub id: String,
    pub name: String,
    /// NCName of the backing [`CaseFileItemDef`] (an unprefixed `xsd:QName`).
    pub definition_ref: String,
}

/// A CMMN case role derived from a declared role.
#[derive(Debug, Clone)]
pub struct Role {
    pub id: String,
    pub name: String,
}

/// A `humanTask` plan-item definition derived from an entity.
#[derive(Debug, Clone)]
pub struct HumanTask {
    pub id: String,
    pub name: String,
    /// Id of the [`Role`] that performs this task, when the graph binds one.
    pub performer_ref: Option<String>,
}

/// A `milestone` plan-item definition derived from an authority policy.
#[derive(Debug, Clone)]
pub struct Milestone {
    pub id: String,
    pub name: String,
}

/// A `sentry` whose `ifPart` condition lowers an authority policy's expression.
#[derive(Debug, Clone)]
pub struct Sentry {
    pub id: String,
    /// The policy expression, rendered as text into the `ifPart` `condition`.
    pub condition: String,
}

/// The kind of plan-item definition a [`PlanItem`] instantiates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanItemKind {
    HumanTask,
    Milestone,
}

/// A `planItem` in the case plan: it references a plan-item definition and, for
/// milestones, carries the `entryCriterion` naming the guarding sentry.
#[derive(Debug, Clone)]
pub struct PlanItem {
    pub id: String,
    pub name: String,
    pub kind: PlanItemKind,
    /// `definitionRef` — id of the referenced [`HumanTask`] or [`Milestone`].
    pub definition_ref: String,
    /// For milestone plan items: the `entryCriterion` id and the sentry it
    /// references (`sentryRef`). `None` for human tasks.
    pub entry_criterion: Option<(String, String)>,
}

/// The complete case, ready to render. Every collection is sorted by id/name so
/// output is byte-identical run-to-run.
#[derive(Debug, Clone)]
pub struct CaseIR {
    pub definitions_id: String,
    pub case_id: String,
    pub case_name: String,
    pub plan_model_id: String,
    pub case_file_item_defs: Vec<CaseFileItemDef>,
    pub case_file_items: Vec<CaseFileItem>,
    pub roles: Vec<Role>,
    pub human_tasks: Vec<HumanTask>,
    pub milestones: Vec<Milestone>,
    pub sentries: Vec<Sentry>,
    pub plan_items: Vec<PlanItem>,
}

const FAMILY: &str = "cmmn";

fn id(prefix: &str, parts: &[&str]) -> String {
    format!("{prefix}_{}", element_id(FAMILY, parts))
}

impl CaseIR {
    /// Build the case IR from `graph`. `model_ref` is a provenance label used
    /// for the definitions/case display name and their ids.
    pub fn from_graph(graph: &Graph, model_ref: &str) -> Self {
        let definitions_id = id("Definitions", &["definitions", model_ref]);
        let case_id = id("Case", &["case", model_ref]);
        let plan_model_id = id("CasePlan", &["caseplan", &case_id]);

        // Roles: one CMMN case role per declared role, sorted by (name, id).
        let mut roles_src: Vec<_> = graph.all_roles();
        roles_src.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        // Map role name → CMMN role id, for performer resolution.
        let mut role_id_by_name: BTreeMap<String, String> = BTreeMap::new();
        let roles: Vec<Role> = roles_src
            .iter()
            .map(|r| {
                let rid = id("Role", &["role", r.namespace(), r.name()]);
                role_id_by_name.insert(r.name().to_string(), rid.clone());
                Role {
                    id: rid,
                    name: r.name().to_string(),
                }
            })
            .collect();

        // Case file items: one per resource, sorted by (name, id). Each is
        // backed by a caseFileItemDefinition so the reference is well-formed.
        let mut resources: Vec<_> = graph.all_resources();
        resources.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        let mut case_file_item_defs = Vec::new();
        let mut case_file_items = Vec::new();
        for r in &resources {
            let def_id = id("CfiDef", &["cfidef", r.namespace(), r.name()]);
            case_file_item_defs.push(CaseFileItemDef {
                id: def_id.clone(),
                name: r.name().to_string(),
            });
            case_file_items.push(CaseFileItem {
                id: id("Cfi", &["cfi", r.namespace(), r.name()]),
                name: r.name().to_string(),
                definition_ref: def_id,
            });
        }

        // Human tasks: one per entity, sorted by (name, id). performerRef names
        // the entity's first (sorted) bound role when the graph carries a
        // binding (the `.sea` surface declares roles but has no bind syntax, so
        // this is typically empty; the wiring is future-proof — see the doc).
        let mut entities: Vec<_> = graph.all_entities();
        entities.sort_by_key(|e| (e.name().to_string(), e.id().to_string()));
        let mut human_tasks = Vec::new();
        for e in &entities {
            let mut role_names = graph.role_names_for_entity(e.id());
            role_names.sort();
            let performer_ref = role_names
                .first()
                .and_then(|n| role_id_by_name.get(n).cloned());
            human_tasks.push(HumanTask {
                id: id("Task", &["task", e.namespace(), e.name()]),
                name: e.name().to_string(),
                performer_ref,
            });
        }

        // Milestones + sentries: one per authority policy, sorted by (name, id).
        // Each policy's expression lowers to a sentry ifPart condition, and the
        // milestone's plan item is gated by an entryCriterion referencing it.
        let mut policies: Vec<_> = graph.all_policies();
        policies.sort_by_key(|p| (p.name.clone(), p.id.to_string()));
        let mut milestones = Vec::new();
        let mut sentries = Vec::new();
        let mut milestone_plan_items = Vec::new();
        for p in &policies {
            let pid = p.id.to_string();
            let milestone_id = id("Milestone", &["milestone", &p.namespace, &p.name]);
            let sentry_id = id("Sentry", &["sentry", &p.namespace, &p.name]);
            let entry_id = id("EntryCriterion", &["entry", &p.namespace, &p.name]);
            let plan_item_id = id("PlanItem", &["pi-milestone", &pid]);

            milestones.push(Milestone {
                id: milestone_id.clone(),
                name: format!("{} satisfied", p.name),
            });
            sentries.push(Sentry {
                id: sentry_id.clone(),
                condition: p.expression().to_string(),
            });
            milestone_plan_items.push(PlanItem {
                id: plan_item_id,
                name: format!("{} satisfied", p.name),
                kind: PlanItemKind::Milestone,
                definition_ref: milestone_id,
                entry_criterion: Some((entry_id, sentry_id)),
            });
        }

        // Plan items: human-task plan items first (sorted with their tasks),
        // then milestone plan items. definitionRef wires each to its definition.
        let mut plan_items: Vec<PlanItem> = human_tasks
            .iter()
            .map(|t| PlanItem {
                id: id("PlanItem", &["pi-task", &t.id]),
                name: t.name.clone(),
                kind: PlanItemKind::HumanTask,
                definition_ref: t.id.clone(),
                entry_criterion: None,
            })
            .collect();
        plan_items.extend(milestone_plan_items);

        CaseIR {
            definitions_id,
            case_id,
            case_name: model_ref.to_string(),
            plan_model_id,
            case_file_item_defs,
            case_file_items,
            roles,
            human_tasks,
            milestones,
            sentries,
            plan_items,
        }
    }

    /// Structural reference-integrity check — the teeth-check the CMMN XSD cannot
    /// perform (`definitionRef`/`sentryRef`/`performerRef` are `xsd:IDREF`, whose
    /// referential integrity XSD validation does not resolve against document
    /// ids). Returns the first dangling reference.
    pub fn validate_references(&self) -> Result<(), String> {
        use std::collections::BTreeSet;
        let def_ids: BTreeSet<&str> = self
            .human_tasks
            .iter()
            .map(|t| t.id.as_str())
            .chain(self.milestones.iter().map(|m| m.id.as_str()))
            .collect();
        let sentry_ids: BTreeSet<&str> = self.sentries.iter().map(|s| s.id.as_str()).collect();
        let role_ids: BTreeSet<&str> = self.roles.iter().map(|r| r.id.as_str()).collect();
        let cfi_def_ids: BTreeSet<&str> = self
            .case_file_item_defs
            .iter()
            .map(|d| d.id.as_str())
            .collect();

        for pi in &self.plan_items {
            if !def_ids.contains(pi.definition_ref.as_str()) {
                return Err(format!(
                    "plan item {} has definitionRef {} with no matching plan item definition",
                    pi.id, pi.definition_ref
                ));
            }
            if let Some((_, sentry_ref)) = &pi.entry_criterion {
                if !sentry_ids.contains(sentry_ref.as_str()) {
                    return Err(format!(
                        "plan item {} has entryCriterion sentryRef {} with no matching sentry",
                        pi.id, sentry_ref
                    ));
                }
            }
        }
        for t in &self.human_tasks {
            if let Some(pr) = &t.performer_ref {
                if !role_ids.contains(pr.as_str()) {
                    return Err(format!(
                        "human task {} has performerRef {} with no matching role",
                        t.id, pr
                    ));
                }
            }
        }
        for cfi in &self.case_file_items {
            if !cfi_def_ids.contains(cfi.definition_ref.as_str()) {
                return Err(format!(
                    "case file item {} has definitionRef {} with no matching definition",
                    cfi.id, cfi.definition_ref
                ));
            }
        }
        Ok(())
    }
}
