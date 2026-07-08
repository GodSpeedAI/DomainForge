//! ArchiMate 3.0 architecture IR: a graph-derived, renderer-agnostic model of a
//! single business-layer architecture, plus its motivation drivers.
//!
//! Mapping (declared v1 subset — see `docs/archimate-projections.md`):
//! - **BusinessRole**    — one per declared role.
//! - **BusinessObject**  — one per declared entity *and* one per declared
//!   resource (both are passive business-layer objects).
//! - **BusinessProcess** — one per declared flow (the activity that moves a
//!   resource from one entity to another).
//! - **Requirement**     — one per authority policy (a motivation-layer driver
//!   the architecture must satisfy).
//!
//! Relations are emitted **only** where the static [`ALLOWED_RELATIONS`] table
//! says the `(source-kind, relation-kind, target-kind)` triple is legal ArchiMate.
//! [`Relation::build`] consults that table and returns `Err` on any illegal
//! triple, so the IR *cannot* hold an out-of-spec relation — validation by
//! construction, the property this projection exists to prove. Emitted relations:
//! - **Assignment**  BusinessRole → BusinessProcess (a role performs a flow's
//!   process) when the graph binds the role to the flow's source entity.
//! - **Access**      BusinessProcess → BusinessObject (a flow's process accesses
//!   the objects for its source and target entities).
//! - **Triggering**  BusinessProcess → BusinessProcess (a flow whose target
//!   entity is another flow's source entity triggers that next flow).
//! - **Association** Requirement → BusinessObject (a policy is associated with
//!   every object whose name its condition expression references).
//!
//! Every id is minted through [`crate::projection::ids::element_id`] under the
//! `archimate` family and prefixed with an alpha tag so it is a legal XML
//! `NCName` / `xsd:ID`. All collections are built and emitted in sorted
//! (`BTreeMap`/`Vec::sort`) order, so output is byte-identical run-to-run.

use crate::graph::Graph;
use crate::projection::ids::element_id;
use std::collections::{BTreeMap, BTreeSet};

/// An ArchiMate element type. The [`ElemKind::archimate_type`] string is the
/// exact `xsi:type` value the renderer stamps on the `<element>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElemKind {
    BusinessRole,
    BusinessObject,
    BusinessProcess,
    BusinessService,
    Requirement,
}

impl ElemKind {
    /// The `xsi:type` value (an ArchiMate element type name).
    pub fn archimate_type(self) -> &'static str {
        match self {
            ElemKind::BusinessRole => "BusinessRole",
            ElemKind::BusinessObject => "BusinessObject",
            ElemKind::BusinessProcess => "BusinessProcess",
            ElemKind::BusinessService => "BusinessService",
            ElemKind::Requirement => "Requirement",
        }
    }

    /// Whether this kind belongs to the business layer (motivation elements such
    /// as [`ElemKind::Requirement`] do not; they are excluded from the
    /// business-layer view).
    pub fn is_business_layer(self) -> bool {
        !matches!(self, ElemKind::Requirement)
    }
}

/// An ArchiMate relationship type. The [`RelKind::archimate_type`] string is the
/// exact `xsi:type` value the renderer stamps on the `<relationship>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelKind {
    Composition,
    Aggregation,
    Assignment,
    Realization,
    Serving,
    Access,
    Triggering,
    Flow,
    Specialization,
    Association,
    Influence,
}

impl RelKind {
    /// The `xsi:type` value (an ArchiMate relationship type name).
    pub fn archimate_type(self) -> &'static str {
        match self {
            RelKind::Composition => "Composition",
            RelKind::Aggregation => "Aggregation",
            RelKind::Assignment => "Assignment",
            RelKind::Realization => "Realization",
            RelKind::Serving => "Serving",
            RelKind::Access => "Access",
            RelKind::Triggering => "Triggering",
            RelKind::Flow => "Flow",
            RelKind::Specialization => "Specialization",
            RelKind::Association => "Association",
            RelKind::Influence => "Influence",
        }
    }
}

/// The curated legal `(source-kind, relation-kind, target-kind)` triples for the
/// element and relation kinds this projection uses. It is a **subset** of the
/// full ArchiMate 3.1 relationship matrix — only the kinds we mint elements for
/// appear — but every entry is genuinely spec-legal, and no triple absent from
/// this table can ever be built (see [`Relation::build`]). Sorted at rest is not
/// required (lookup is a linear scan), but the table is grouped by relation for
/// readability.
pub const ALLOWED_RELATIONS: &[(ElemKind, RelKind, ElemKind)] = &[
    // Assignment: an active-structure element performs a behavior element.
    (
        ElemKind::BusinessRole,
        RelKind::Assignment,
        ElemKind::BusinessProcess,
    ),
    (
        ElemKind::BusinessRole,
        RelKind::Assignment,
        ElemKind::BusinessService,
    ),
    // Access: a behavior element accesses a passive-structure element.
    (
        ElemKind::BusinessProcess,
        RelKind::Access,
        ElemKind::BusinessObject,
    ),
    (
        ElemKind::BusinessService,
        RelKind::Access,
        ElemKind::BusinessObject,
    ),
    // Triggering: temporal/causal ordering between behavior elements.
    (
        ElemKind::BusinessProcess,
        RelKind::Triggering,
        ElemKind::BusinessProcess,
    ),
    (
        ElemKind::BusinessProcess,
        RelKind::Triggering,
        ElemKind::BusinessService,
    ),
    (
        ElemKind::BusinessService,
        RelKind::Triggering,
        ElemKind::BusinessProcess,
    ),
    // Flow: transfer between behavior elements.
    (
        ElemKind::BusinessProcess,
        RelKind::Flow,
        ElemKind::BusinessProcess,
    ),
    // Serving: a behavior element serves an active-structure or behavior element.
    (
        ElemKind::BusinessService,
        RelKind::Serving,
        ElemKind::BusinessRole,
    ),
    (
        ElemKind::BusinessService,
        RelKind::Serving,
        ElemKind::BusinessProcess,
    ),
    (
        ElemKind::BusinessProcess,
        RelKind::Serving,
        ElemKind::BusinessRole,
    ),
    // Realization: a more concrete element realizes a more abstract one.
    (
        ElemKind::BusinessProcess,
        RelKind::Realization,
        ElemKind::BusinessService,
    ),
    (
        ElemKind::BusinessProcess,
        RelKind::Realization,
        ElemKind::Requirement,
    ),
    (
        ElemKind::BusinessService,
        RelKind::Realization,
        ElemKind::Requirement,
    ),
    (
        ElemKind::BusinessObject,
        RelKind::Realization,
        ElemKind::Requirement,
    ),
    // Association: the universal, weakly-typed relationship.
    (
        ElemKind::Requirement,
        RelKind::Association,
        ElemKind::BusinessObject,
    ),
    (
        ElemKind::Requirement,
        RelKind::Association,
        ElemKind::BusinessProcess,
    ),
    (
        ElemKind::Requirement,
        RelKind::Association,
        ElemKind::BusinessRole,
    ),
    (
        ElemKind::BusinessObject,
        RelKind::Association,
        ElemKind::BusinessObject,
    ),
    // Influence: one motivation element influences another.
    (
        ElemKind::Requirement,
        RelKind::Influence,
        ElemKind::Requirement,
    ),
    // Specialization: same-type generalization.
    (
        ElemKind::BusinessRole,
        RelKind::Specialization,
        ElemKind::BusinessRole,
    ),
    (
        ElemKind::BusinessObject,
        RelKind::Specialization,
        ElemKind::BusinessObject,
    ),
    (
        ElemKind::BusinessProcess,
        RelKind::Specialization,
        ElemKind::BusinessProcess,
    ),
    (
        ElemKind::BusinessService,
        RelKind::Specialization,
        ElemKind::BusinessService,
    ),
    (
        ElemKind::Requirement,
        RelKind::Specialization,
        ElemKind::Requirement,
    ),
    // Composition / Aggregation: whole-part between compatible elements.
    (
        ElemKind::BusinessObject,
        RelKind::Composition,
        ElemKind::BusinessObject,
    ),
    (
        ElemKind::BusinessObject,
        RelKind::Aggregation,
        ElemKind::BusinessObject,
    ),
    (
        ElemKind::BusinessProcess,
        RelKind::Composition,
        ElemKind::BusinessProcess,
    ),
    (
        ElemKind::Requirement,
        RelKind::Aggregation,
        ElemKind::Requirement,
    ),
];

/// Whether `(source, rel, target)` is a legal ArchiMate triple per
/// [`ALLOWED_RELATIONS`].
pub fn relation_allowed(source: ElemKind, rel: RelKind, target: ElemKind) -> bool {
    ALLOWED_RELATIONS
        .iter()
        .any(|&(s, r, t)| s == source && r == rel && t == target)
}

/// An ArchiMate element (a node in the model).
#[derive(Debug, Clone)]
pub struct Element {
    pub id: String,
    pub kind: ElemKind,
    pub name: String,
}

/// An ArchiMate relationship (a typed, directed edge). Constructed only through
/// [`Relation::build`], which rejects any triple absent from [`ALLOWED_RELATIONS`].
#[derive(Debug, Clone)]
pub struct Relation {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: RelKind,
    pub source_kind: ElemKind,
    pub target_kind: ElemKind,
}

impl Relation {
    /// Build a relation, enforcing the ArchiMate relation matrix by construction.
    /// Returns `Err` (naming the offending triple) when the
    /// `(source-kind, relation-kind, target-kind)` triple is not in
    /// [`ALLOWED_RELATIONS`] — the teeth-check the whole projection turns on.
    pub fn build(
        id: String,
        source: String,
        source_kind: ElemKind,
        kind: RelKind,
        target: String,
        target_kind: ElemKind,
    ) -> Result<Self, String> {
        if !relation_allowed(source_kind, kind, target_kind) {
            return Err(format!(
                "illegal ArchiMate relation: {} --{}--> {} is not in the allowed-relations matrix",
                source_kind.archimate_type(),
                kind.archimate_type(),
                target_kind.archimate_type()
            ));
        }
        Ok(Relation {
            id,
            source,
            target,
            kind,
            source_kind,
            target_kind,
        })
    }
}

/// A node inside the auto-generated view: it references an [`Element`] and
/// carries a deterministic grid position (visual layout is index-derived, never
/// authored, so output stays byte-identical).
#[derive(Debug, Clone)]
pub struct ViewNode {
    pub id: String,
    pub element_ref: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// A single ArchiMate view (`xsi:type="Diagram"`) listing element references.
#[derive(Debug, Clone)]
pub struct View {
    pub id: String,
    pub name: String,
    pub nodes: Vec<ViewNode>,
}

/// The complete architecture, ready to render. Every collection is sorted so
/// output is byte-identical run-to-run.
#[derive(Debug, Clone)]
pub struct ArchitectureIR {
    pub model_id: String,
    pub model_name: String,
    pub elements: Vec<Element>,
    pub relations: Vec<Relation>,
    pub views: Vec<View>,
}

const FAMILY: &str = "archimate";

fn id(prefix: &str, parts: &[&str]) -> String {
    format!("{prefix}_{}", element_id(FAMILY, parts))
}

/// Split `expr` into identifier tokens (`[A-Za-z0-9_]+` runs) for policy→object
/// association discovery.
fn identifier_tokens(expr: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut cur = String::new();
    for ch in expr.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.insert(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.insert(cur);
    }
    out
}

impl ArchitectureIR {
    /// Build the architecture IR from `graph`. `model_ref` is a provenance label
    /// used for the model display name and its id. Returns `Err` only if a
    /// relation would violate the ArchiMate matrix — which, by construction of
    /// this builder, it never does; the fallibility is the guarantee.
    pub fn from_graph(graph: &Graph, model_ref: &str) -> Result<Self, String> {
        let model_id = id("Model", &["model", model_ref]);

        // --- Elements -------------------------------------------------------
        // Roles → BusinessRole.
        let mut roles: Vec<_> = graph.all_roles();
        roles.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        let mut role_elem_by_name: BTreeMap<String, String> = BTreeMap::new();
        let mut elements: Vec<Element> = Vec::new();
        for r in &roles {
            let eid = id("Element", &["role", r.namespace(), r.name()]);
            role_elem_by_name.insert(r.name().to_string(), eid.clone());
            elements.push(Element {
                id: eid,
                kind: ElemKind::BusinessRole,
                name: r.name().to_string(),
            });
        }

        // Entities → BusinessObject. Track element id + ConceptId string so
        // flows can look up their endpoints.
        let mut entities: Vec<_> = graph.all_entities();
        entities.sort_by_key(|e| (e.name().to_string(), e.id().to_string()));
        // ConceptId string → (element id, entity ConceptId for role lookup).
        let mut object_by_entity: BTreeMap<String, String> = BTreeMap::new();
        let mut entity_cid: BTreeMap<String, crate::ConceptId> = BTreeMap::new();
        for e in &entities {
            let eid = id("Element", &["entity", e.namespace(), e.name()]);
            object_by_entity.insert(e.id().to_string(), eid.clone());
            entity_cid.insert(e.id().to_string(), e.id().clone());
            elements.push(Element {
                id: eid,
                kind: ElemKind::BusinessObject,
                name: e.name().to_string(),
            });
        }

        // Resources → BusinessObject. Named objects for policy association.
        let mut resources: Vec<_> = graph.all_resources();
        resources.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        // Object name → element id (entities and resources), for policy tokens.
        let mut object_by_name: BTreeMap<String, String> = BTreeMap::new();
        for e in &entities {
            object_by_name
                .entry(e.name().to_string())
                .or_insert_with(|| object_by_entity[&e.id().to_string()].clone());
        }
        for r in &resources {
            let eid = id("Element", &["resource", r.namespace(), r.name()]);
            object_by_name
                .entry(r.name().to_string())
                .or_insert_with(|| eid.clone());
            elements.push(Element {
                id: eid,
                kind: ElemKind::BusinessObject,
                name: r.name().to_string(),
            });
        }

        // Flows → BusinessProcess. Flow ids are random UUIDs, so the process id
        // is derived from the (deterministic) resource/from/to ConceptIds.
        // Only flows whose endpoints resolve to known entities are projected.
        let mut flow_keys: Vec<(String, String, String)> = Vec::new();
        for f in graph.all_flows() {
            let from = f.from_id().to_string();
            let to = f.to_id().to_string();
            if !object_by_entity.contains_key(&from) || !object_by_entity.contains_key(&to) {
                continue;
            }
            flow_keys.push((from, to, f.resource_id().to_string()));
        }
        flow_keys.sort();
        flow_keys.dedup();
        // (from, to) → process element id, for triggering chains.
        let mut process_by_key: BTreeMap<(String, String, String), String> = BTreeMap::new();
        let mut processes_from_entity: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (from, to, res) in &flow_keys {
            let pid = id("Element", &["process", from, to, res]);
            let from_name = entities
                .iter()
                .find(|e| e.id().to_string() == *from)
                .map(|e| e.name().to_string())
                .unwrap_or_default();
            let to_name = entities
                .iter()
                .find(|e| e.id().to_string() == *to)
                .map(|e| e.name().to_string())
                .unwrap_or_default();
            elements.push(Element {
                id: pid.clone(),
                kind: ElemKind::BusinessProcess,
                name: format!("{from_name} to {to_name}"),
            });
            process_by_key.insert((from.clone(), to.clone(), res.clone()), pid.clone());
            processes_from_entity
                .entry(from.clone())
                .or_default()
                .push(pid);
        }

        // Policies → Requirement.
        let mut policies: Vec<_> = graph.all_policies();
        policies.sort_by_key(|p| (p.name.clone(), p.id.to_string()));
        let mut requirement_of_policy: Vec<(String, String)> = Vec::new(); // (req id, expr)
        for p in &policies {
            let rid = id("Element", &["requirement", &p.namespace, &p.name]);
            elements.push(Element {
                id: rid.clone(),
                kind: ElemKind::Requirement,
                name: p.name.clone(),
            });
            requirement_of_policy.push((rid, p.expression().to_string()));
        }

        // Elements sorted for byte-determinism (kind, then name, then id).
        elements.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.id.cmp(&b.id))
        });

        // --- Relations (validation by construction) ------------------------
        // Collected into a map keyed by relation id so duplicates collapse and
        // ordering is deterministic.
        let mut relations: BTreeMap<String, Relation> = BTreeMap::new();
        let mut add = |rid: String,
                       src: String,
                       sk: ElemKind,
                       rk: RelKind,
                       tgt: String,
                       tk: ElemKind|
         -> Result<(), String> {
            let rel = Relation::build(rid.clone(), src, sk, rk, tgt, tk)?;
            relations.entry(rid).or_insert(rel);
            Ok(())
        };

        for (from, to, res) in &flow_keys {
            let pid = &process_by_key[&(from.clone(), to.clone(), res.clone())];
            let from_obj = &object_by_entity[from];
            let to_obj = &object_by_entity[to];
            // Access: the process reads/writes the two endpoint objects.
            add(
                id("Relation", &["access", pid, from_obj]),
                pid.clone(),
                ElemKind::BusinessProcess,
                RelKind::Access,
                from_obj.clone(),
                ElemKind::BusinessObject,
            )?;
            add(
                id("Relation", &["access", pid, to_obj]),
                pid.clone(),
                ElemKind::BusinessProcess,
                RelKind::Access,
                to_obj.clone(),
                ElemKind::BusinessObject,
            )?;
            // Assignment: roles bound to the source entity perform the process.
            if let Some(cid) = entity_cid.get(from) {
                let mut role_names = graph.role_names_for_entity(cid);
                role_names.sort();
                for rn in &role_names {
                    if let Some(role_elem) = role_elem_by_name.get(rn) {
                        add(
                            id("Relation", &["assign", role_elem, pid]),
                            role_elem.clone(),
                            ElemKind::BusinessRole,
                            RelKind::Assignment,
                            pid.clone(),
                            ElemKind::BusinessProcess,
                        )?;
                    }
                }
            }
        }

        // Triggering: a flow whose target entity is another flow's source entity
        // triggers that next flow's process.
        for (from, to, res) in &flow_keys {
            let pid = &process_by_key[&(from.clone(), to.clone(), res.clone())];
            if let Some(next_processes) = processes_from_entity.get(to) {
                for next_pid in next_processes {
                    if next_pid == pid {
                        continue;
                    }
                    add(
                        id("Relation", &["trigger", pid, next_pid]),
                        pid.clone(),
                        ElemKind::BusinessProcess,
                        RelKind::Triggering,
                        next_pid.clone(),
                        ElemKind::BusinessProcess,
                    )?;
                }
            }
        }

        // Association: a requirement is associated with every object whose name
        // its condition expression references (token match).
        for (rid, expr) in &requirement_of_policy {
            let tokens = identifier_tokens(expr);
            for (obj_name, obj_id) in &object_by_name {
                let referenced = tokens.contains(obj_name)
                    || (obj_name.contains(' ') && expr.contains(obj_name.as_str()));
                if referenced {
                    add(
                        id("Relation", &["assoc", rid, obj_id]),
                        rid.clone(),
                        ElemKind::Requirement,
                        RelKind::Association,
                        obj_id.clone(),
                        ElemKind::BusinessObject,
                    )?;
                }
            }
        }

        let relations: Vec<Relation> = relations.into_values().collect();

        // --- View: one auto-generated business-layer view -------------------
        // Lists every business-layer element (roles, objects, processes) via a
        // node with a deterministic grid position. Motivation requirements are a
        // different layer and are excluded (per-stakeholder views are v2).
        let business: Vec<&Element> = elements
            .iter()
            .filter(|e| e.kind.is_business_layer())
            .collect();
        let cols = 5;
        let (cell_w, cell_h, node_w, node_h) = (185, 100, 160, 80);
        let nodes: Vec<ViewNode> = business
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let col = (i % cols) as i32;
                let row = (i / cols) as i32;
                ViewNode {
                    id: id("Node", &["node", &e.id]),
                    element_ref: e.id.clone(),
                    x: 12 + col * cell_w,
                    y: 12 + row * cell_h,
                    w: node_w,
                    h: node_h,
                }
            })
            .collect();
        let views = vec![View {
            id: id("View", &["view", "business", model_ref]),
            name: "Business Layer".to_string(),
            nodes,
        }];

        Ok(ArchitectureIR {
            model_id,
            model_name: model_ref.to_string(),
            elements,
            relations,
            views,
        })
    }

    /// Structural reference-integrity check — the teeth-check XSD `xsd:IDREF`
    /// validation cannot fully perform against minted ids. Verifies every
    /// relationship endpoint, every view `elementRef`, and asserts every held
    /// relation is still spec-legal (defence in depth against a hand-mutated IR).
    /// Returns the first problem found.
    pub fn validate_references(&self) -> Result<(), String> {
        let elem_ids: BTreeSet<&str> = self.elements.iter().map(|e| e.id.as_str()).collect();
        let elem_kind: BTreeMap<&str, ElemKind> = self
            .elements
            .iter()
            .map(|e| (e.id.as_str(), e.kind))
            .collect();

        for r in &self.relations {
            if !elem_ids.contains(r.source.as_str()) {
                return Err(format!(
                    "relationship {} has source {} with no matching element",
                    r.id, r.source
                ));
            }
            if !elem_ids.contains(r.target.as_str()) {
                return Err(format!(
                    "relationship {} has target {} with no matching element",
                    r.id, r.target
                ));
            }
            if !relation_allowed(r.source_kind, r.kind, r.target_kind) {
                return Err(format!(
                    "relationship {} is an illegal ArchiMate triple: {} --{}--> {}",
                    r.id,
                    r.source_kind.archimate_type(),
                    r.kind.archimate_type(),
                    r.target_kind.archimate_type()
                ));
            }
            // Endpoint kinds must match the declared source/target kinds.
            if elem_kind.get(r.source.as_str()) != Some(&r.source_kind) {
                return Err(format!(
                    "relationship {} source-kind disagrees with element {}",
                    r.id, r.source
                ));
            }
            if elem_kind.get(r.target.as_str()) != Some(&r.target_kind) {
                return Err(format!(
                    "relationship {} target-kind disagrees with element {}",
                    r.id, r.target
                ));
            }
        }
        for v in &self.views {
            for n in &v.nodes {
                if !elem_ids.contains(n.element_ref.as_str()) {
                    return Err(format!(
                        "view {} node {} references element {} which does not exist",
                        v.id, n.id, n.element_ref
                    ));
                }
            }
        }
        Ok(())
    }
}
