//! OWL axiom layer for the RDF projection.
//!
//! `kg.rs` already expresses instance triples (Turtle / RDF-XML / JSON-LD via
//! [`super`]). What it does not surface as a first-class, model-derived artifact
//! is an **ontology**: the class axioms and the property domain/range axioms
//! implied by the entity / role / resource / flow / relation domains actually
//! declared in the model. This module is that missing pass — an `OntologyIR`
//! built over the same conceptual vocabulary as `kg.rs`, rendered to a
//! standalone `ontology.owl.ttl`.
//!
//! Only kinds present in the model produce axioms, so renaming or removing a
//! concept changes exactly the axioms derived from it. Every minted local name
//! is routed through [`crate::projection::ids::sanitize_qname`]; output order is
//! fully sorted, so a fixed input yields byte-identical output.

use crate::graph::Graph;
use crate::projection::ids::sanitize_qname;
use std::collections::BTreeSet;

/// A `sea:`-prefixed class axiom (`<local> a owl:Class`).
struct ClassAxiom {
    local: &'static str,
    comment: &'static str,
}

/// A property axiom with an optional `rdfs:domain` / `rdfs:range`. `object` is
/// `true` for `owl:ObjectProperty`, `false` for `owl:DatatypeProperty`.
struct PropAxiom {
    local: &'static str,
    object: bool,
    domain: &'static str,
    range: &'static str,
}

/// A `owl:NamedIndividual` minted from a declared concept.
struct Individual {
    /// `sanitize_qname`-encoded local name (the IRI fragment after `sea:`).
    local: String,
    class: &'static str,
    label: String,
}

/// The model-derived ontology: classes, properties, and individuals. Built only
/// from kinds the model actually declares.
pub struct OntologyIR {
    classes: Vec<ClassAxiom>,
    object_props: Vec<PropAxiom>,
    data_props: Vec<PropAxiom>,
    individuals: Vec<Individual>,
}

impl OntologyIR {
    /// Derive the ontology from the graph. Deterministic: individuals are sorted
    /// by `(label, local)` and de-duplicated by minted local name.
    pub fn from_graph(graph: &Graph) -> Self {
        let has_entities = !graph.all_entities().is_empty();
        let has_roles = !graph.all_roles().is_empty();
        let has_resources = !graph.all_resources().is_empty();
        let has_flows = !graph.all_flows().is_empty();
        let has_relations = !graph.all_relations().is_empty();

        let mut classes = Vec::new();
        if has_entities {
            classes.push(ClassAxiom {
                local: "Entity",
                comment: "Business actor, location, or organizational unit",
            });
        }
        if has_roles {
            classes.push(ClassAxiom {
                local: "Role",
                comment: "Named participant in a relation",
            });
        }
        if has_resources {
            classes.push(ClassAxiom {
                local: "Resource",
                comment: "Quantifiable subject of value",
            });
        }
        if has_flows {
            classes.push(ClassAxiom {
                local: "Flow",
                comment: "Transfer of a resource between entities",
            });
        }
        if has_relations {
            classes.push(ClassAxiom {
                local: "Relation",
                comment: "Typed association between two roles",
            });
        }

        let mut object_props = Vec::new();
        let mut data_props = Vec::new();
        if has_flows {
            object_props.push(PropAxiom {
                local: "from",
                object: true,
                domain: "Flow",
                range: "Entity",
            });
            object_props.push(PropAxiom {
                local: "to",
                object: true,
                domain: "Flow",
                range: "Entity",
            });
            object_props.push(PropAxiom {
                local: "hasResource",
                object: true,
                domain: "Flow",
                range: "Resource",
            });
            data_props.push(PropAxiom {
                local: "quantity",
                object: false,
                domain: "Flow",
                range: "xsd:decimal",
            });
        }
        if has_relations {
            object_props.push(PropAxiom {
                local: "subjectRole",
                object: true,
                domain: "Relation",
                range: "Role",
            });
            object_props.push(PropAxiom {
                local: "objectRole",
                object: true,
                domain: "Relation",
                range: "Role",
            });
        }

        // Individuals from the closed entity / role / resource domains.
        let mut seen = BTreeSet::new();
        let mut individuals = Vec::new();
        let mut push_individual = |local: String, class: &'static str, label: String| {
            if seen.insert(local.clone()) {
                individuals.push(Individual {
                    local,
                    class,
                    label,
                });
            }
        };
        for e in graph.all_entities() {
            push_individual(sanitize_qname(e.name()), "Entity", e.name().to_string());
        }
        for r in graph.all_roles() {
            push_individual(sanitize_qname(r.name()), "Role", r.name().to_string());
        }
        for r in graph.all_resources() {
            push_individual(sanitize_qname(r.name()), "Resource", r.name().to_string());
        }
        individuals.sort_by(|a, b| (&a.label, &a.local).cmp(&(&b.label, &b.local)));

        Self {
            classes,
            object_props,
            data_props,
            individuals,
        }
    }

    /// Render the ontology to Turtle. `base` is the IRI the `sea:` prefix
    /// expands to; `ontology_iri` is the ontology's own IRI (`base` minus a
    /// trailing `#`). `header` is a provenance comment block emitted verbatim.
    pub fn to_turtle(
        &self,
        base: &str,
        ontology_iri: &str,
        header: &str,
        model_hash: &str,
    ) -> String {
        let mut s = String::new();
        s.push_str(header);
        s.push_str(&format!("@prefix sea: <{base}> .\n"));
        s.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        s.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        s.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        s.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");

        s.push_str(&format!("<{ontology_iri}> a owl:Ontology ;\n"));
        s.push_str(&format!("    owl:versionInfo \"{model_hash}\" .\n\n"));

        if !self.classes.is_empty() {
            s.push_str("# Classes\n");
            for c in &self.classes {
                s.push_str(&format!(
                    "sea:{} a owl:Class ;\n    rdfs:label \"{}\" ;\n    rdfs:comment \"{}\" .\n\n",
                    c.local, c.local, c.comment
                ));
            }
        }

        let all_props: Vec<&PropAxiom> = self.object_props.iter().chain(&self.data_props).collect();
        if !all_props.is_empty() {
            s.push_str("# Properties\n");
            for p in &all_props {
                let kind = if p.object {
                    "owl:ObjectProperty"
                } else {
                    "owl:DatatypeProperty"
                };
                let range = if p.range.contains(':') {
                    p.range.to_string()
                } else {
                    format!("sea:{}", p.range)
                };
                s.push_str(&format!(
                    "sea:{} a {kind} ;\n    rdfs:domain sea:{} ;\n    rdfs:range {range} .\n\n",
                    p.local, p.domain
                ));
            }
        }

        if !self.individuals.is_empty() {
            s.push_str("# Individuals\n");
            for i in &self.individuals {
                s.push_str(&format!(
                    "sea:{} a owl:NamedIndividual, sea:{} ;\n    rdfs:label \"{}\" .\n\n",
                    i.local,
                    i.class,
                    escape_literal(&i.label)
                ));
            }
        }
        s
    }
}

/// Minimal Turtle string-literal escaper for individual labels (backslash and
/// double-quote only; concept names never carry control characters).
fn escape_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
