//! RDF / OWL projection: promotes the existing `kg.rs` triple model into a
//! first-class, deterministic projection family (`--format rdf`).
//!
//! Emits three artifacts over one triple set:
//! - `model.ttl` — Turtle instances, via the proven [`KnowledgeGraph::to_turtle`].
//! - `model.jsonld` — JSON-LD serialized from the same triples (this module).
//! - `ontology.owl.ttl` — OWL class / property axioms derived from the declared
//!   entity / role / resource / flow / relation domains (see [`ontology`]).
//!
//! Determinism: `kg.rs` builds triples in the graph's insertion order (an
//! `IndexMap`) and every `ConceptId` is a content-derived UUIDv5, so the triple
//! set is stable for a fixed model. This module additionally sorts JSON-LD
//! subjects (a `BTreeMap`) and routes every minted IRI fragment through
//! [`crate::projection::ids`], so output is byte-identical run-to-run.

pub mod ontology;

use crate::graph::Graph;
use crate::projection::ids::content_hash;
use crate::projection::sink::ArtifactSink;
use crate::KnowledgeGraph;
use ontology::OntologyIR;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// `kg.rs` mints flow and pattern subjects as `sea:flow_<ConceptId>` /
/// `sea:pattern_<ConceptId>`, and flow/pattern `ConceptId`s are allocated
/// per-parse (not content-derived like entities), so the raw IRIs vary run to
/// run. Rewrite each such IRI to `sea:<kind>_<hash>`, where the hash is derived
/// (via [`content_hash`]) from that node's *other* triples — its stable content.
/// This is the RDF family's deterministic-identity guarantee for anonymous nodes.
fn canonicalize_node_ids(kg: &mut KnowledgeGraph) {
    // Gather the stable content key for every flow_/pattern_ subject.
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for t in &kg.triples {
        if is_minted_node(&t.subject) {
            // Exclude rdf:type (constant per kind) so the key is pure content.
            if t.predicate != "rdf:type" {
                keys.entry(t.subject.clone())
                    .or_default()
                    .push(format!("{} {}", t.predicate, t.object));
            } else {
                keys.entry(t.subject.clone()).or_default();
            }
        }
    }

    let mut remap: BTreeMap<String, String> = BTreeMap::new();
    for (subject, mut parts) in keys {
        parts.sort();
        let kind = subject
            .strip_prefix("sea:")
            .and_then(|s| s.split('_').next())
            .unwrap_or("node")
            .to_string();
        let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
        let hash = content_hash(&refs);
        remap.insert(subject, format!("sea:{kind}_{hash}"));
    }

    for t in &mut kg.triples {
        if let Some(new) = remap.get(&t.subject) {
            t.subject = new.clone();
        }
        if let Some(new) = remap.get(&t.object) {
            t.object = new.clone();
        }
    }
}

fn is_minted_node(iri: &str) -> bool {
    iri.starts_with("sea:flow_") || iri.starts_with("sea:pattern_")
}

/// The IRI the `sea:` prefix expands to in `kg.rs`'s Turtle output. Used as the
/// default `--base-iri` so `model.ttl`, `model.jsonld`, and `ontology.owl.ttl`
/// share one vocabulary namespace unless the caller overrides it.
pub const SEA_NS: &str = "http://domainforge.ai/sea#";

/// Projection options. `base_iri` overrides the `sea:` prefix expansion in the
/// JSON-LD `@context` and the OWL ontology; `None` uses [`SEA_NS`].
#[derive(Debug, Clone, Default)]
pub struct RdfOptions {
    pub base_iri: Option<String>,
}

impl RdfOptions {
    fn base(&self) -> &str {
        self.base_iri.as_deref().unwrap_or(SEA_NS)
    }
}

/// Emit the RDF dataset into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    opts: &RdfOptions,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let base = opts.base();

    let mut kg =
        KnowledgeGraph::from_graph(graph).map_err(|e| format!("RDF projection failed: {e}"))?;
    canonicalize_node_ids(&mut kg);
    let turtle = kg.to_turtle();
    // Provenance hash of the canonical RDF serialization (routes through the
    // shared id module, per the projection kernel convention).
    let model_hash = content_hash(&[&turtle]);

    let jsonld = to_jsonld(&kg, base);
    let ontology_iri = base.trim_end_matches(['#', '/']);
    let header = header(model_ref, &created_at);
    let owl = OntologyIR::from_graph(graph).to_turtle(base, ontology_iri, &header, &model_hash);

    let files: Vec<(&str, String)> = vec![
        ("model.ttl", turtle),
        ("model.jsonld", jsonld),
        ("ontology.owl.ttl", owl),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (rel, content) in files {
        sink.write(rel, &content)?;
        written.push(rel.to_string());
    }
    Ok(written)
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_rdf_in_memory(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    base_iri: Option<String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(
        graph,
        model_ref,
        created_at,
        &RdfOptions { base_iri },
        &mut sink,
    )?;
    Ok(map)
}

fn header(model_ref: &str, created_at: &str) -> String {
    format!(
        "# Generated by DomainForge (--format rdf) from {}.\n# Created: {}. Do not edit by hand.\n\n",
        model_ref.replace(['\n', '\r'], " "),
        created_at.replace(['\n', '\r'], " ")
    )
}

/// Serialize the knowledge graph's triples as a JSON-LD document. Subjects are
/// grouped into nodes (`@id`), `rdf:type` becomes `@type`, IRI objects become
/// `{"@id": ...}`, and literals become plain or typed values. `base` is the
/// `sea:` prefix expansion. Output is deterministic: subjects and predicates
/// are `BTreeMap`-ordered.
fn to_jsonld(kg: &KnowledgeGraph, base: &str) -> String {
    // subject -> predicate-key -> ordered values
    let mut nodes: BTreeMap<&str, BTreeMap<String, Vec<Value>>> = BTreeMap::new();
    for t in &kg.triples {
        let key = if t.predicate == "rdf:type" {
            "@type".to_string()
        } else {
            t.predicate.clone()
        };
        nodes
            .entry(&t.subject)
            .or_default()
            .entry(key)
            .or_default()
            .push(object_value(&t.predicate, &t.object));
    }

    let graph_nodes: Vec<Value> = nodes
        .into_iter()
        .map(|(subject, preds)| {
            let mut obj: BTreeMap<String, Value> = BTreeMap::new();
            obj.insert("@id".to_string(), Value::String(subject.to_string()));
            for (key, mut values) in preds {
                let v = if values.len() == 1 {
                    values.pop().unwrap()
                } else {
                    Value::Array(values)
                };
                obj.insert(key, v);
            }
            Value::Object(obj.into_iter().collect::<Map<String, Value>>())
        })
        .collect();

    let mut context: BTreeMap<String, Value> = BTreeMap::new();
    context.insert("sea".to_string(), Value::String(base.to_string()));
    context.insert(
        "owl".to_string(),
        Value::String("http://www.w3.org/2002/07/owl#".to_string()),
    );
    context.insert(
        "rdf".to_string(),
        Value::String("http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string()),
    );
    context.insert(
        "rdfs".to_string(),
        Value::String("http://www.w3.org/2000/01/rdf-schema#".to_string()),
    );
    context.insert(
        "xsd".to_string(),
        Value::String("http://www.w3.org/2001/XMLSchema#".to_string()),
    );

    let mut doc: BTreeMap<String, Value> = BTreeMap::new();
    doc.insert(
        "@context".to_string(),
        Value::Object(context.into_iter().collect::<Map<String, Value>>()),
    );
    doc.insert("@graph".to_string(), Value::Array(graph_nodes));

    let doc = Value::Object(doc.into_iter().collect::<Map<String, Value>>());
    let mut out = serde_json::to_string_pretty(&doc).unwrap_or_default();
    out.push('\n');
    out
}

/// Convert one `kg.rs` triple object into a JSON-LD value. `rdf:type` objects
/// are compact IRI strings; other IRI objects become `{"@id": ...}`; literals
/// become a plain string or a `{"@value","@type"}` pair for typed literals.
fn object_value(predicate: &str, object: &str) -> Value {
    if predicate == "rdf:type" {
        return Value::String(object.to_string());
    }
    if let Some(stripped) = object.strip_prefix('"') {
        // Literal, optionally `"..."^^xsd:type`.
        if let Some(idx) = object.rfind("^^") {
            let datatype = &object[idx + 2..];
            let lit = &object[1..idx]; // between opening quote and `"^^`
            let lit = lit.strip_suffix('"').unwrap_or(lit);
            let mut m: BTreeMap<String, Value> = BTreeMap::new();
            m.insert("@type".to_string(), Value::String(datatype.to_string()));
            m.insert("@value".to_string(), Value::String(unescape_turtle(lit)));
            return Value::Object(m.into_iter().collect::<Map<String, Value>>());
        }
        let inner = stripped.strip_suffix('"').unwrap_or(stripped);
        return Value::String(unescape_turtle(inner));
    }
    // Prefixed IRI reference (e.g. `sea:Warehouse`).
    let mut m: BTreeMap<String, Value> = BTreeMap::new();
    m.insert("@id".to_string(), Value::String(object.to_string()));
    Value::Object(m.into_iter().collect::<Map<String, Value>>())
}

/// Reverse `kg.rs::escape_turtle_literal` for JSON-LD `@value` extraction.
fn unescape_turtle(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
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
"#;

    fn project(source: &str) -> BTreeMap<String, String> {
        let graph = parse_to_graph(source).expect("fixture parses");
        project_rdf_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()), None)
            .expect("projection succeeds")
    }

    #[test]
    fn emits_three_artifacts() {
        let files = project(SOURCE);
        assert_eq!(
            files.keys().collect::<Vec<_>>(),
            vec!["model.jsonld", "model.ttl", "ontology.owl.ttl"]
        );
    }

    #[test]
    fn turtle_has_instances() {
        let files = project(SOURCE);
        let ttl = &files["model.ttl"];
        assert!(ttl.contains("sea:Warehouse rdf:type sea:Entity"));
        assert!(ttl.contains("sea:CameraUnits"));
    }

    #[test]
    fn jsonld_is_valid_json_with_graph_and_context() {
        let files = project(SOURCE);
        let doc: Value = serde_json::from_str(&files["model.jsonld"]).expect("valid JSON");
        assert!(doc.get("@context").is_some());
        assert_eq!(doc["@context"]["sea"], Value::String(SEA_NS.to_string()));
        let graph = doc["@graph"].as_array().expect("@graph array");
        let warehouse = graph
            .iter()
            .find(|n| n["@id"] == Value::String("sea:Warehouse".to_string()))
            .expect("warehouse node");
        assert_eq!(warehouse["@type"], Value::String("sea:Entity".to_string()));
    }

    #[test]
    fn jsonld_typed_literal_carries_datatype() {
        let files = project(SOURCE);
        let doc: Value = serde_json::from_str(&files["model.jsonld"]).unwrap();
        let graph = doc["@graph"].as_array().unwrap();
        let flow = graph
            .iter()
            .find(|n| n.get("sea:quantity").is_some())
            .expect("flow node with quantity");
        assert_eq!(flow["sea:quantity"]["@type"], "xsd:decimal");
        assert_eq!(flow["sea:quantity"]["@value"], "100");
    }

    #[test]
    fn ontology_has_axioms_and_individuals() {
        let files = project(SOURCE);
        let owl = &files["ontology.owl.ttl"];
        assert!(owl.contains("sea:Entity a owl:Class"));
        assert!(owl.contains("sea:from a owl:ObjectProperty"));
        assert!(owl.contains("rdfs:range sea:Entity"));
        assert!(owl.contains("sea:quantity a owl:DatatypeProperty"));
        assert!(owl.contains("rdfs:range xsd:decimal"));
        assert!(owl.contains("sea:Warehouse a owl:NamedIndividual, sea:Entity"));
        assert!(owl.contains("owl:versionInfo"));
    }

    #[test]
    fn base_iri_override_changes_context_only() {
        let graph = parse_to_graph(SOURCE).expect("parses");
        let files = project_rdf_in_memory(
            &graph,
            "test.sea",
            Some(FIXED_TS.to_string()),
            Some("https://example.org/demo#".to_string()),
        )
        .unwrap();
        let doc: Value = serde_json::from_str(&files["model.jsonld"]).unwrap();
        assert_eq!(doc["@context"]["sea"], "https://example.org/demo#");
        assert!(files["ontology.owl.ttl"].contains("@prefix sea: <https://example.org/demo#>"));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_dataset() {
        let files = project("@namespace \"empty\"\n");
        assert_eq!(files.len(), 3);
        let doc: Value = serde_json::from_str(&files["model.jsonld"]).unwrap();
        assert!(doc["@graph"].is_array());
    }
}
