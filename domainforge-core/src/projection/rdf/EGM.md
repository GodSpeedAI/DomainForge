# RDF / OWL Projection

## Overview
Promotes the triple model into a first-class deterministic projection family, emitting three artifacts over one triple set: Turtle instances, JSON-LD instances, and an OWL ontology derived from the model's declared domains.

## Interface
- `[mod.rs:SEA_NS]` — the IRI the `sea:` prefix expands to, used as the default base IRI so all three artifacts share one vocabulary namespace.
- `[mod.rs:RdfOptions]` — projection options; `base_iri` overrides `SEA_NS` in the JSON-LD `@context` and the OWL ontology.
- `[mod.rs:emit]` — builds and canonicalizes the knowledge graph, then writes `model.ttl`, `model.jsonld`, and `ontology.owl.ttl` into an `ArtifactSink`.
- `[mod.rs:project_rdf_in_memory]` — binding surface: graph in, `BTreeMap<path, content>` out, no filesystem.
- `[ontology.rs:OntologyIR]` — the model-derived OWL axiom set (classes, object/datatype properties, named individuals).
- `[ontology.rs:OntologyIR::from_graph]` — derives the ontology from declared concepts only.
- `[ontology.rs:OntologyIR::to_turtle]` — renders the OWL axioms to Turtle with a verbatim provenance header and `owl:versionInfo` set to the model hash.

## Invariants
- `[mod.rs:canonicalize_node_ids]` — flow/pattern subjects (`sea:flow_<ConceptId>`, `sea:pattern_<ConceptId>`) carry per-parse ids, so they are rewritten to `sea:<kind>_<hash>` where the hash is derived from that node's non-`rdf:type` triples. This is the family's deterministic-identity guarantee for anonymous nodes.
- `[mod.rs:emit]` — `model_hash` is `content_hash` of the canonical Turtle serialization, so provenance identity tracks the actual serialization.
- `[ontology.rs:OntologyIR::from_graph]` — only kinds the model actually declares produce axioms, so renaming/removing a concept changes exactly the axioms derived from it.
- `[ontology.rs:OntologyIR::from_graph]` — individuals are sorted by `(label, local)` and de-duplicated by minted local name; every local name goes through `sanitize_qname`.
- `[mod.rs:emit]` — exactly three artifacts, emitted in the fixed order Turtle → JSON-LD → OWL; JSON-LD subjects are sorted (`BTreeMap`) so output is byte-identical run-to-run.

## Side Effects
- `[mod.rs:emit]` — writes through `ArtifactSink::write` (filesystem write under the output root, or in-memory map insertion).
- `[mod.rs:emit]` — substitutes `chrono::Utc::now().to_rfc3339()` when `created_at` is `None`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
