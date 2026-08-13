# RDF / OWL Projection (`--format rdf`)

DomainForge can project a `.sea` model into a deterministic **RDF dataset**: the
semantic-graph operator of the projection family. It promotes the existing
knowledge-graph machinery (`kg.rs`) into a first-class projection target with a
stable IRI policy, JSON-LD, and an OWL ontology derived from the model.

```bash
domainforge project --format rdf domain/model.sea out/
domainforge validate-kg out/model.ttl        # SHACL validation (needs the shacl feature)
```

The same artifacts are available from the language bindings as a
path → content map (no filesystem access needed):

```python
artifacts = json.loads(graph.export_rdf_projection(created_at="2026-07-02T00:00:00+00:00"))
```

```ts
const artifacts = JSON.parse(graph.exportRdfProjection(undefined, '2026-07-02T00:00:00+00:00'));
```

## What gets generated

| File | Content |
| --- | --- |
| `model.ttl` | Turtle instance triples (via the proven `KnowledgeGraph::to_turtle`) |
| `model.jsonld` | JSON-LD serialization of the **same** triple set — one `@context`, subjects grouped into `@graph` nodes |
| `ontology.owl.ttl` | OWL class axioms and property domains/ranges derived from the declared entity/role/resource/flow/relation domains, plus `owl:NamedIndividual`s and a provenance `owl:versionInfo` |

All three files share one vocabulary: entities/roles/resources appear as
`sea:<Name>`, and the `sea:` prefix expands to the same base IRI in every file.

Declared `Instance` and `Policy` declarations are model content, not derived
structure, so they project too: an instance becomes `sea:instance_<name>` typed
`sea:EntityInstance`, linked to its entity with `sea:instanceOf` and carrying one
`sea:<fieldName>` triple per field; a policy becomes `sea:policy_<name>` typed
`sea:Policy`. Instance names are unique graph-wide, so the name alone identifies
the individual and retyping an instance does not change its IRI. Field values
carry the `xsd:` datatype declared by the entity's contract when the entity is
typed, and fall back to the JSON value's own shape otherwise. See
`docs/specs/SDS-005-knowledge-graph-module.md` §4.2–§4.5 for the full mapping.

## IRI policy

- Every minted local name is routed through
  `domainforge-core/src/projection/ids.rs` (`sanitize_qname`): ASCII
  alphanumerics and `_ - .` are kept, everything else becomes `_`, and a leading
  digit is prefixed with `_`. This is the shared projection-kernel identity rule,
  so RDF IRIs are stable and uniform with the other families.
- The `sea:` prefix expands to `http://domainforge.ai/sea#` by default — the
  canonical SEA vocabulary namespace, matching `model.ttl`. Override it with
  `--base-iri <IRI>` to relocate the JSON-LD `@context` and OWL prefix (for
  example to your organization's namespace).
- Provenance: `ontology.owl.ttl` stamps `owl:versionInfo` with a content hash of
  the canonical RDF serialization (via `projection::ids::content_hash`), so the
  ontology is traceable to the exact triples it was generated from.

## Determinism

Identical model + fixed `--created-at` produce byte-identical output:

- `kg.rs` builds triples in the graph's insertion order (an `IndexMap`), and
  every `ConceptId` is a content-derived UUIDv5 — so the triple set is stable.
- The JSON-LD serializer additionally sorts subjects and predicates
  (`BTreeMap`), and the OWL pass sorts individuals by `(label, local)`.
- Renaming exactly one entity changes exactly the IRIs derived from that entity
  and nothing else (the deterministic-identity teeth-check in
  `domainforge-core/tests/rdf_projection_tests.rs`).

CI gate (see the `verify-rdf` job in `.github/workflows/ci.yml`): project the
fixture, run `domainforge validate-kg out/model.ttl` (SHACL), and parse
`out/model.jsonld` with a stock JSON loader.

## Validation

`domainforge validate-kg out/model.ttl` re-parses the Turtle through oxigraph
and runs the SHACL shapes emitted by `kg.rs`. This requires the `shacl` feature:

```bash
cargo run --features cli,shacl -- validate-kg out/model.ttl
```

## Non-goals (v1)

- **OWL reasoning / consistency checking is not performed** — the ontology
  states class and property axioms; DomainForge does not run a reasoner.
- **Policies are stated, not enforced.** A declared `Policy` projects as a
  `sea:Policy` individual carrying its normalized expression, modality, kind,
  priority, rationale, and tags. Policies are not lowered into SHACL
  constraints; the emitted shapes remain the fixed structural invariants over
  `sea:Flow` and `sea:Entity`. A partial translation would silently change what
  a policy means.
- **No typed instance-to-instance edges.** A `FieldType::EntityRef` field emits
  the referenced key value as a literal, not as an IRI, so a model that has not
  passed validation cannot produce dangling references.
- **Resource instances are not projected.** `ResourceInstance` identity is a
  random UUIDv4 rather than a content-derived `ConceptId`, so emitting it would
  break the determinism guarantee. A content-derived identity is the
  prerequisite, not a projection change.
- **`model.ttl` uses the canonical SEA vocabulary IRI regardless of
  `--base-iri`** (it is produced by the legacy `kg.rs` serializer, which is not
  forked). `--base-iri` reparameterizes the JSON-LD and OWL files only; keep the
  default for full cross-file IRI identity. Namespace-derived defaults are a
  documented follow-up.
- **No RDF-star, named-graph datasets, or SHACL-shape emission beyond what
  `kg.rs` already produces.**
- No `projection … target rdf` contract surface — the target is
  CLI-format-driven (per the Lean precedent).
