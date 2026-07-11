//! BPMN 2.0 process IR: a graph-derived, renderer-agnostic model of one
//! non-executable process.
//!
//! Mapping (declared v1 subset — see `docs/bpmn-projections.md`):
//! - **process**  — one per model, `isExecutable="false"`.
//! - **task**     — one per entity that participates in a flow (as source or
//!   target); the entity is the actor performing the step.
//! - **sequence flow** — one per declared flow edge, plus start/end wiring.
//! - **gateway**  — a `parallelGateway` is inserted wherever an entity fans out
//!   to (`Diverging`) or in from (`Converging`) more than one flow. Flows carry
//!   no branch predicate, so every split/join is a parallel (AND) gateway;
//!   discriminating exclusive gateways needs condition semantics the model does
//!   not yet express (redesign note in the doc).
//! - **start / end event** — a single start event fans out to every source
//!   entity (no incoming flow); every sink entity (no outgoing flow) feeds a
//!   single end event.
//! - **lane**        — one per role that owns ≥1 participating entity; the
//!   entity's first (sorted) role owns its task.
//! - **data object** — one per declared resource.
//!
//! Every id is minted through [`crate::projection::ids::element_id`] under the
//! `bpmn` family and prefixed with an alpha tag so it is a legal XML `NCName`.
//! All collections are built and emitted in sorted (`BTreeMap`/`BTreeSet`)
//! order, so output is byte-identical run-to-run.

use crate::graph::Graph;
use crate::projection::ids::element_id;
use std::collections::{BTreeMap, BTreeSet};

/// The kind of BPMN flow node, mapped to a concrete element tag by the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    StartEvent,
    EndEvent,
    Task,
    /// `parallelGateway` with `gatewayDirection="Diverging"`.
    GatewayDiverging,
    /// `parallelGateway` with `gatewayDirection="Converging"`.
    GatewayConverging,
}

/// A BPMN flow node (event, task, or gateway). `incoming`/`outgoing` hold the
/// ids of the sequence flows touching this node.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub name: Option<String>,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

/// A directed sequence flow between two flow nodes.
#[derive(Debug, Clone)]
pub struct SequenceFlow {
    pub id: String,
    pub source: String,
    pub target: String,
}

/// A pool lane grouping the tasks performed under one role.
#[derive(Debug, Clone)]
pub struct Lane {
    pub id: String,
    pub name: String,
    pub flow_node_refs: Vec<String>,
}

/// A BPMN data object derived from a declared resource.
#[derive(Debug, Clone)]
pub struct DataObject {
    pub id: String,
    pub name: String,
}

/// The complete process, ready to render. Nodes and sequence flows are sorted
/// by id; lanes and data objects by name/id.
#[derive(Debug, Clone)]
pub struct ProcessIR {
    pub definitions_id: String,
    pub process_id: String,
    pub process_name: String,
    pub lane_set_id: Option<String>,
    pub lanes: Vec<Lane>,
    pub data_objects: Vec<DataObject>,
    pub nodes: Vec<Node>,
    pub sequence_flows: Vec<SequenceFlow>,
}

const FAMILY: &str = "bpmn";

fn id(prefix: &str, parts: &[&str]) -> String {
    format!("{prefix}_{}", element_id(FAMILY, parts))
}

impl ProcessIR {
    /// Build the process IR from `graph`. `model_ref` is a provenance label used
    /// only for the definitions/process display name and their ids.
    pub fn from_graph(graph: &Graph, model_ref: &str) -> Self {
        let definitions_id = id("Definitions", &["definitions", model_ref]);
        let process_id = id("Process", &["process", model_ref]);
        let start_id = id("StartEvent", &["start", &process_id]);
        let end_id = id("EndEvent", &["end", &process_id]);

        // Sorted entities keyed by ConceptId string, so iteration and id minting
        // are deterministic.
        let mut entities: Vec<_> = graph.all_entities();
        entities.sort_by_key(|e| (e.name().to_string(), e.id().to_string()));
        let entity_name: BTreeMap<String, String> = entities
            .iter()
            .map(|e| (e.id().to_string(), e.name().to_string()))
            .collect();

        // Degree counts over resolvable flows (endpoints must be known entities).
        let mut out_deg: BTreeMap<String, usize> = BTreeMap::new();
        let mut in_deg: BTreeMap<String, usize> = BTreeMap::new();
        let mut edges: Vec<(String, String)> = Vec::new();
        let mut participating: BTreeSet<String> = BTreeSet::new();
        for f in graph.all_flows() {
            let from = f.from_id().to_string();
            let to = f.to_id().to_string();
            if !entity_name.contains_key(&from) || !entity_name.contains_key(&to) {
                continue;
            }
            *out_deg.entry(from.clone()).or_default() += 1;
            *in_deg.entry(to.clone()).or_default() += 1;
            participating.insert(from.clone());
            participating.insert(to.clone());
            edges.push((from, to));
        }
        // Deterministic edge order independent of graph insertion order.
        edges.sort();

        // Per-entity node ids and gateway ids.
        let task_id = |eid: &str| id("Task", &["task", eid]);
        let diverge_id = |eid: &str| id("Gateway", &["diverge", eid]);
        let converge_id = |eid: &str| id("Gateway", &["converge", eid]);

        // The node a flow *leaves* an entity through (its diverging gateway when
        // it fans out, else the task itself) and the node a flow *enters* through.
        let exit_of = |eid: &str| {
            if out_deg.get(eid).copied().unwrap_or(0) > 1 {
                diverge_id(eid)
            } else {
                task_id(eid)
            }
        };
        let entry_of = |eid: &str| {
            if in_deg.get(eid).copied().unwrap_or(0) > 1 {
                converge_id(eid)
            } else {
                task_id(eid)
            }
        };

        // Assemble nodes into a map keyed by id.
        let mut nodes: BTreeMap<String, Node> = BTreeMap::new();
        let mut add = |id: String, kind: NodeKind, name: Option<String>| {
            nodes.entry(id.clone()).or_insert(Node {
                id,
                kind,
                name,
                incoming: Vec::new(),
                outgoing: Vec::new(),
            });
        };

        add(start_id.clone(), NodeKind::StartEvent, None);
        add(end_id.clone(), NodeKind::EndEvent, None);
        for eid in &participating {
            add(task_id(eid), NodeKind::Task, Some(entity_name[eid].clone()));
            if out_deg.get(eid).copied().unwrap_or(0) > 1 {
                add(diverge_id(eid), NodeKind::GatewayDiverging, None);
            }
            if in_deg.get(eid).copied().unwrap_or(0) > 1 {
                add(converge_id(eid), NodeKind::GatewayConverging, None);
            }
        }

        // Sequence flows: deterministic id from (source,target). Collected into a
        // map so duplicate (source,target) pairs collapse to one edge.
        let mut flows: BTreeMap<String, SequenceFlow> = BTreeMap::new();
        let mut connect = |source: String, target: String| {
            let fid = id("Flow", &["flow", &source, &target]);
            flows.entry(fid.clone()).or_insert(SequenceFlow {
                id: fid,
                source,
                target,
            });
        };

        if participating.is_empty() {
            // No flows in the model: a minimal, well-formed start→end process.
            connect(start_id.clone(), end_id.clone());
        } else {
            // Internal task↔gateway wiring.
            for eid in &participating {
                if out_deg.get(eid).copied().unwrap_or(0) > 1 {
                    connect(task_id(eid), diverge_id(eid));
                }
                if in_deg.get(eid).copied().unwrap_or(0) > 1 {
                    connect(converge_id(eid), task_id(eid));
                }
            }
            // One sequence flow per declared edge.
            for (from, to) in &edges {
                connect(exit_of(from), entry_of(to));
            }
            // Start fans out to sources; sinks feed the end event.
            for eid in &participating {
                if in_deg.get(eid).copied().unwrap_or(0) == 0 {
                    connect(start_id.clone(), entry_of(eid));
                }
                if out_deg.get(eid).copied().unwrap_or(0) == 0 {
                    connect(exit_of(eid), end_id.clone());
                }
            }
        }

        // Populate incoming/outgoing from the sequence flows.
        for f in flows.values() {
            if let Some(n) = nodes.get_mut(&f.source) {
                n.outgoing.push(f.id.clone());
            }
            if let Some(n) = nodes.get_mut(&f.target) {
                n.incoming.push(f.id.clone());
            }
        }
        for n in nodes.values_mut() {
            n.incoming.sort();
            n.outgoing.sort();
        }

        // Lanes: one swimlane per declared role. `flowNodeRef`s are populated
        // from any entity→role bindings the graph carries — each participating
        // entity's first (sorted) role owns its task. The current `.sea` surface
        // declares roles but does not expose syntax to bind them to entities, so
        // refs are typically empty; the graph itself supports binding, so the
        // wiring is future-proof (see the doc's limitations note).
        let mut lane_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for eid in &participating {
            let cid = entities
                .iter()
                .find(|e| e.id().to_string() == *eid)
                .map(|e| e.id().clone());
            let Some(cid) = cid else { continue };
            let mut role_names = graph.role_names_for_entity(&cid);
            role_names.sort();
            if let Some(role) = role_names.first() {
                lane_refs
                    .entry(role.clone())
                    .or_default()
                    .insert(task_id(eid));
            }
        }
        let mut roles: Vec<_> = graph.all_roles();
        roles.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        let lanes: Vec<Lane> = roles
            .iter()
            .map(|r| {
                let name = r.name().to_string();
                let refs = lane_refs.remove(&name).unwrap_or_default();
                Lane {
                    id: id("Lane", &["lane", r.namespace(), &name]),
                    name,
                    flow_node_refs: refs.into_iter().collect(),
                }
            })
            .collect();
        let lane_set_id = (!lanes.is_empty()).then(|| id("LaneSet", &["laneset", &process_id]));

        // Data objects: one per resource, sorted by (name, id).
        let mut resources: Vec<_> = graph.all_resources();
        resources.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        let data_objects: Vec<DataObject> = resources
            .iter()
            .map(|r| DataObject {
                id: id("DataObject", &["data", r.namespace(), r.name()]),
                name: r.name().to_string(),
            })
            .collect();

        ProcessIR {
            definitions_id,
            process_id,
            process_name: model_ref.to_string(),
            lane_set_id,
            lanes,
            data_objects,
            nodes: nodes.into_values().collect(),
            sequence_flows: flows.into_values().collect(),
        }
    }

    /// Structural reference-integrity check — the teeth-check the BPMN XSD cannot
    /// perform (sequence-flow `incoming`/`outgoing`/`sourceRef`/`targetRef` are
    /// `xsd:QName`, which XSD validation does not resolve against document ids).
    ///
    /// Verifies every node `incoming`/`outgoing` names a declared sequence flow,
    /// every sequence-flow endpoint names a declared node, and every lane
    /// `flowNodeRef` names a declared node. Returns the first dangling reference.
    pub fn validate_references(&self) -> Result<(), String> {
        let node_ids: BTreeSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        let flow_ids: BTreeSet<&str> = self.sequence_flows.iter().map(|f| f.id.as_str()).collect();

        for f in &self.sequence_flows {
            if !node_ids.contains(f.source.as_str()) {
                return Err(format!(
                    "sequence flow {} has sourceRef {} with no matching flow node",
                    f.id, f.source
                ));
            }
            if !node_ids.contains(f.target.as_str()) {
                return Err(format!(
                    "sequence flow {} has targetRef {} with no matching flow node",
                    f.id, f.target
                ));
            }
        }
        for n in &self.nodes {
            for r in n.incoming.iter().chain(&n.outgoing) {
                if !flow_ids.contains(r.as_str()) {
                    return Err(format!(
                        "flow node {} references sequence flow {} which does not exist",
                        n.id, r
                    ));
                }
            }
        }
        for lane in &self.lanes {
            for r in &lane.flow_node_refs {
                if !node_ids.contains(r.as_str()) {
                    return Err(format!(
                        "lane {} references flow node {} which does not exist",
                        lane.id, r
                    ));
                }
            }
        }
        Ok(())
    }
}
