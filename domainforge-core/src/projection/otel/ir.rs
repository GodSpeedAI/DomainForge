//! OpenTelemetry semantic-convention IR: a graph-derived, renderer-agnostic
//! model of the telemetry a system built from this domain model should emit.
//!
//! Mapping (declared v1 subset — see `docs/otel-projections.md`):
//! - **resource attribute group** — carries the correlation keys
//!   `domainforge.model.ref`, `domainforge.model.hash`, and
//!   `domainforge.element.id`. These are the traceability keystone: a runtime
//!   span/metric stamped with these values points back at the exact projected
//!   model and the exact domain element it describes.
//! - **entity attribute group**  — one per declared entity; attributes for the
//!   entity's deterministic element id and human name.
//! - **flow attribute group**    — one per declared flow; resource / source /
//!   target / quantity attributes plus an INTERNAL span-kind suggestion.
//! - **policy attribute group**  — one per authority policy (a policy-decision
//!   point); attributes for the emitted decision and the policy's modality.
//!
//! **Reserved-namespace guard.** Every attribute key is validated at
//! construction time against [`validate_attribute_key`]: only the model's own
//! namespace (`<model-ns>.*`) and the DomainForge vendor namespace
//! (`domainforge.*`) are allowed. Any OTel-reserved or well-known namespace
//! (`otel.*`, `service.*`, `telemetry.*`, …) is rejected with an `Err`, so the
//! IR can never hold an attribute that would collide with an upstream semantic
//! convention — validation by construction.
//!
//! Every element id is minted through [`crate::projection::ids::element_id`]
//! under the `otel` family. All collections are built and emitted in sorted
//! (`BTreeMap` / `BTreeSet` / `Vec::sort`) order, so output is byte-identical
//! run-to-run for a fixed input.

use crate::graph::Graph;
use crate::projection::ids::{content_hash, element_id};
use std::collections::{BTreeMap, BTreeSet};

/// Projection family tag for [`crate::projection::ids::element_id`].
pub const FAMILY: &str = "otel";

/// DomainForge's own vendor namespace. Correlation attributes live here and are
/// allowed alongside the model namespace; every other namespace is rejected.
pub const VENDOR_NS: &str = "domainforge";

/// OTel-reserved / well-known top-level namespaces a domain projection must
/// never mint into. The guard is an allowlist (model ns + [`VENDOR_NS`]), so
/// this list is documentation for the error message rather than the mechanism.
pub const RESERVED_NAMESPACES: &[&str] = &[
    "otel",
    "service",
    "telemetry",
    "process",
    "host",
    "os",
    "cloud",
    "k8s",
    "container",
    "http",
    "db",
    "rpc",
    "messaging",
    "network",
    "url",
    "server",
    "client",
];

/// A semantic-convention attribute value type. The [`AttrType::as_str`] value is
/// the exact `type:` string the YAML registry emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    String,
    Int,
    Double,
    Boolean,
}

impl AttrType {
    pub fn as_str(self) -> &'static str {
        match self {
            AttrType::String => "string",
            AttrType::Int => "int",
            AttrType::Double => "double",
            AttrType::Boolean => "boolean",
        }
    }
}

/// A single semantic-convention attribute definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub key: String,
    pub r#type: AttrType,
    pub brief: String,
}

/// The suggested span kind for a flow-derived operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Internal,
    Producer,
    Consumer,
    Client,
    Server,
}

impl SpanKind {
    /// The exact `span_kind:` string the YAML registry emits.
    pub fn as_str(self) -> &'static str {
        match self {
            SpanKind::Internal => "internal",
            SpanKind::Producer => "producer",
            SpanKind::Consumer => "consumer",
            SpanKind::Client => "client",
            SpanKind::Server => "server",
        }
    }
}

/// A registry attribute group: a named bundle of attribute definitions.
#[derive(Debug, Clone)]
pub struct AttributeGroup {
    pub id: String,
    pub brief: String,
    pub attributes: Vec<Attribute>,
}

/// A span-kind suggestion for a flow step. References (does not redefine) the
/// attributes of the flow's attribute group plus the resource correlation id.
#[derive(Debug, Clone)]
pub struct SpanSuggestion {
    pub id: String,
    pub brief: String,
    pub span_kind: SpanKind,
    pub attribute_refs: Vec<String>,
}

/// The complete telemetry model, ready to render to a registry and to constant
/// files. Groups and spans are sorted by id for byte-determinism.
#[derive(Debug, Clone)]
pub struct TelemetryIR {
    pub model_ns: String,
    pub model_ref: String,
    pub model_hash: String,
    pub groups: Vec<AttributeGroup>,
    pub spans: Vec<SpanSuggestion>,
}

/// The reserved-namespace guard. Returns `Ok` only when `key` is prefixed with
/// the model namespace or the DomainForge vendor namespace; every other prefix
/// (notably the OTel-reserved namespaces in [`RESERVED_NAMESPACES`]) is `Err`.
pub fn validate_attribute_key(model_ns: &str, key: &str) -> Result<(), String> {
    let model_prefix = format!("{model_ns}.");
    let vendor_prefix = format!("{VENDOR_NS}.");
    if key.starts_with(&model_prefix) || key.starts_with(&vendor_prefix) {
        return Ok(());
    }
    let ns = key.split('.').next().unwrap_or(key);
    Err(format!(
        "otel projection refuses attribute `{key}`: only the model namespace \
         `{model_ns}.*` and the DomainForge vendor namespace `{VENDOR_NS}.*` may \
         be minted; `{ns}.*` is reserved by OpenTelemetry semantic conventions"
    ))
}

/// Lowercase an arbitrary name into a single dot-free attribute segment:
/// ASCII alphanumerics are kept (lowercased), every other character becomes `_`,
/// and a leading digit is prefixed with `_`. Empty input yields `_`.
fn segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "_".to_string();
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// Deterministically choose the single namespace every domain attribute is
/// prefixed with: the lexicographically first non-empty namespace declared by
/// any concept, sanitized to one segment. Falls back to [`VENDOR_NS`] for an
/// empty model.
fn model_namespace(graph: &Graph) -> String {
    let mut nss: BTreeSet<String> = BTreeSet::new();
    for e in graph.all_entities() {
        nss.insert(e.namespace().to_string());
    }
    for r in graph.all_roles() {
        nss.insert(r.namespace().to_string());
    }
    for r in graph.all_resources() {
        nss.insert(r.namespace().to_string());
    }
    for f in graph.all_flows() {
        nss.insert(f.namespace().to_string());
    }
    for p in graph.all_policies() {
        nss.insert(p.namespace.clone());
    }
    nss.into_iter()
        .find(|s| !s.is_empty())
        .map(|s| segment(&s))
        .unwrap_or_else(|| VENDOR_NS.to_string())
}

impl TelemetryIR {
    /// All attribute definitions across every group, deduplicated by key and
    /// sorted. The single producer both the YAML registry and the constant
    /// files render from, so the two can never disagree on the attribute set.
    pub fn all_attributes(&self) -> Vec<&Attribute> {
        let mut by_key: BTreeMap<&str, &Attribute> = BTreeMap::new();
        for g in &self.groups {
            for a in &g.attributes {
                by_key.entry(&a.key).or_insert(a);
            }
        }
        by_key.into_values().collect()
    }

    /// Build the telemetry IR from `graph`. `model_ref` is a provenance label.
    /// Returns `Err` if any minted attribute key violates the reserved-namespace
    /// guard — which, by construction, this builder never does.
    pub fn from_graph(graph: &Graph, model_ref: &str) -> Result<Self, String> {
        let ns = model_namespace(graph);

        // Deterministic view of the graph.
        let mut entities: Vec<_> = graph.all_entities();
        entities.sort_by_key(|e| (e.name().to_string(), e.id().to_string()));
        let mut resources: Vec<_> = graph.all_resources();
        resources.sort_by_key(|r| (r.name().to_string(), r.id().to_string()));
        let mut policies: Vec<_> = graph.all_policies();
        policies.sort_by_key(|p| (p.name.clone(), p.id.to_string()));

        let entity_name: BTreeMap<String, String> = entities
            .iter()
            .map(|e| (e.id().to_string(), e.name().to_string()))
            .collect();
        let resource_name: BTreeMap<String, String> = resources
            .iter()
            .map(|r| (r.id().to_string(), r.name().to_string()))
            .collect();

        // Model hash: a content hash over the model's stable identity surface,
        // routed through the shared id module. Correlates runtime telemetry to
        // the exact projected model.
        let model_hash = compute_model_hash(&entities, &resources, &policies, graph);

        let mut groups: Vec<AttributeGroup> = Vec::new();
        let mut spans: Vec<SpanSuggestion> = Vec::new();

        // Helper that enforces the reserved-namespace guard on every key.
        let mk = |key: String, ty: AttrType, brief: String| -> Result<Attribute, String> {
            validate_attribute_key(&ns, &key)?;
            Ok(Attribute {
                key,
                r#type: ty,
                brief,
            })
        };

        // --- Resource correlation group ------------------------------------
        groups.push(AttributeGroup {
            id: format!("registry.{VENDOR_NS}.resource"),
            brief: "DomainForge correlation attributes carried on the OTel Resource. \
                    They bind every emitted span and metric back to the projected model \
                    and the specific domain element it describes."
                .to_string(),
            attributes: vec![
                mk(
                    format!("{VENDOR_NS}.model.ref"),
                    AttrType::String,
                    "Provenance label of the source model this telemetry was projected from."
                        .to_string(),
                )?,
                mk(
                    format!("{VENDOR_NS}.model.hash"),
                    AttrType::String,
                    format!(
                        "Deterministic content hash of the projected model ({model_hash}). \
                         Runtime telemetry carrying this value is provably about this exact model."
                    ),
                )?,
                mk(
                    format!("{VENDOR_NS}.element.id"),
                    AttrType::String,
                    "Deterministic element id (domainforge element_id) of the domain element a \
                     span or metric describes; the join key back to the model."
                        .to_string(),
                )?,
            ],
        });

        // --- Entity groups -------------------------------------------------
        for e in &entities {
            let name = e.name();
            let seg = segment(name);
            let eid = element_id(FAMILY, &["entity", e.namespace(), name]);
            groups.push(AttributeGroup {
                id: format!("registry.{ns}.entity.{seg}"),
                brief: format!("Telemetry attributes for entity `{name}` (element {eid})."),
                attributes: vec![
                    mk(
                        format!("{ns}.entity.{seg}.id"),
                        AttrType::String,
                        format!("Deterministic element id of entity `{name}`: {eid}."),
                    )?,
                    mk(
                        format!("{ns}.entity.{seg}.name"),
                        AttrType::String,
                        format!("Human-authored name of entity `{name}`."),
                    )?,
                ],
            });
        }

        // --- Flow groups + span suggestions --------------------------------
        // Deterministic key per flow, independent of graph insertion order.
        let mut flow_keys: Vec<(String, String, String)> = Vec::new();
        for f in graph.all_flows() {
            let from = f.from_id().to_string();
            let to = f.to_id().to_string();
            let res = f.resource_id().to_string();
            if !entity_name.contains_key(&from)
                || !entity_name.contains_key(&to)
                || !resource_name.contains_key(&res)
            {
                continue;
            }
            flow_keys.push((from, to, res));
        }
        flow_keys.sort();
        flow_keys.dedup();
        for (from, to, res) in &flow_keys {
            let from_name = &entity_name[from];
            let to_name = &entity_name[to];
            let res_name = &resource_name[res];
            let key = element_id(FAMILY, &["flow", res_name, from_name, to_name]);
            let prefix = format!("{ns}.flow.{key}");
            let attrs = vec![
                mk(
                    format!("{prefix}.resource"),
                    AttrType::String,
                    format!("Resource moved by this flow: `{res_name}`."),
                )?,
                mk(
                    format!("{prefix}.source"),
                    AttrType::String,
                    format!("Source entity of this flow: `{from_name}`."),
                )?,
                mk(
                    format!("{prefix}.target"),
                    AttrType::String,
                    format!("Target entity of this flow: `{to_name}`."),
                )?,
                mk(
                    format!("{prefix}.quantity"),
                    AttrType::Double,
                    format!("Quantity of `{res_name}` moved from `{from_name}` to `{to_name}`."),
                )?,
            ];
            let refs: Vec<String> = attrs
                .iter()
                .map(|a| a.key.clone())
                .chain(std::iter::once(format!("{VENDOR_NS}.element.id")))
                .collect();
            groups.push(AttributeGroup {
                id: format!("registry.{ns}.flow.{key}"),
                brief: format!(
                    "Telemetry attributes for the flow moving `{res_name}` from `{from_name}` \
                     to `{to_name}`."
                ),
                attributes: attrs,
            });
            spans.push(SpanSuggestion {
                id: format!("span.{ns}.flow.{key}"),
                brief: format!(
                    "Suggested INTERNAL span for the `{from_name}` → `{to_name}` transfer of \
                     `{res_name}`."
                ),
                span_kind: SpanKind::Internal,
                attribute_refs: refs,
            });
        }

        // --- Policy (decision-point) groups --------------------------------
        for p in &policies {
            let name = &p.name;
            let seg = segment(name);
            groups.push(AttributeGroup {
                id: format!("registry.{ns}.policy.{seg}"),
                brief: format!(
                    "Telemetry attributes for the decision point of policy `{name}` \
                     ({:?} / {:?}, priority {}).",
                    p.kind, p.modality, p.priority
                ),
                attributes: vec![
                    mk(
                        format!("{ns}.policy.{seg}.decision"),
                        AttrType::String,
                        format!(
                            "Decision emitted when evaluating policy `{name}` \
                             (e.g. `allow` / `deny`)."
                        ),
                    )?,
                    mk(
                        format!("{ns}.policy.{seg}.modality"),
                        AttrType::String,
                        format!("Deontic modality of policy `{name}`: {:?}.", p.modality),
                    )?,
                ],
            });
        }

        groups.sort_by(|a, b| a.id.cmp(&b.id));
        spans.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(TelemetryIR {
            model_ns: ns,
            model_ref: model_ref.to_string(),
            model_hash,
            groups,
            spans,
        })
    }
}

/// Content hash over the model's stable identity surface (entity / resource /
/// policy names + namespaces and flow endpoints), routed through the shared id
/// module so it matches the deterministic-identity convention.
fn compute_model_hash(
    entities: &[&crate::primitives::Entity],
    resources: &[&crate::primitives::Resource],
    policies: &[&crate::policy::Policy],
    graph: &Graph,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    for e in entities {
        parts.push(format!("entity:{}:{}", e.namespace(), e.name()));
    }
    for r in resources {
        parts.push(format!("resource:{}:{}", r.namespace(), r.name()));
    }
    let mut flow_parts: Vec<String> = graph
        .all_flows()
        .iter()
        .map(|f| format!("flow:{}:{}:{}", f.resource_id(), f.from_id(), f.to_id()))
        .collect();
    flow_parts.sort();
    parts.extend(flow_parts);
    for p in policies {
        parts.push(format!("policy:{}:{}", p.namespace, p.name));
    }
    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    content_hash(&refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_to_graph;

    const SOURCE: &str = r#"
@namespace "demo"

Entity "Warehouse" in demo
Entity "Factory" in demo
Resource "CameraUnits" units in demo
Flow "CameraUnits" from "Warehouse" to "Factory" quantity 100
Policy positive_flow as: Flow.quantity > 0
"#;

    fn ir() -> TelemetryIR {
        let graph = parse_to_graph(SOURCE).expect("parses");
        TelemetryIR::from_graph(&graph, "test.sea").expect("builds")
    }

    #[test]
    fn every_attribute_is_namespaced() {
        let ir = ir();
        for a in ir.all_attributes() {
            assert!(
                a.key.starts_with("demo.") || a.key.starts_with("domainforge."),
                "attribute `{}` escaped the model/vendor namespace",
                a.key
            );
        }
    }

    #[test]
    fn carries_correlation_attributes() {
        let ir = ir();
        let keys: BTreeSet<&str> = ir.all_attributes().iter().map(|a| a.key.as_str()).collect();
        assert!(keys.contains("domainforge.model.hash"));
        assert!(keys.contains("domainforge.element.id"));
        assert!(keys.contains("domainforge.model.ref"));
    }

    #[test]
    fn reserved_namespace_attribute_is_rejected() {
        // The teeth-check: an attempt to mint an OTel-reserved attribute must
        // fail at construction time.
        assert!(validate_attribute_key("demo", "service.name").is_err());
        assert!(validate_attribute_key("demo", "otel.scope.name").is_err());
        assert!(validate_attribute_key("demo", "telemetry.sdk.version").is_err());
        // The model namespace and the vendor namespace are the only ones allowed.
        assert!(validate_attribute_key("demo", "demo.entity.warehouse.id").is_ok());
        assert!(validate_attribute_key("demo", "domainforge.model.hash").is_ok());
    }

    #[test]
    fn flow_yields_internal_span_suggestion() {
        let ir = ir();
        assert_eq!(ir.spans.len(), 1);
        assert_eq!(ir.spans[0].span_kind, SpanKind::Internal);
        assert!(ir.spans[0]
            .attribute_refs
            .contains(&"domainforge.element.id".to_string()));
    }

    #[test]
    fn deterministic_across_builds() {
        let a = ir();
        let b = ir();
        assert_eq!(a.model_hash, b.model_hash);
        assert_eq!(
            a.all_attributes()
                .iter()
                .map(|x| x.key.clone())
                .collect::<Vec<_>>(),
            b.all_attributes()
                .iter()
                .map(|x| x.key.clone())
                .collect::<Vec<_>>()
        );
    }
}
