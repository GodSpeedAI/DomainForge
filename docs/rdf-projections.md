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
- **No typed entity attributes.** Attributes are untyped in the IR today, so the
  ontology derives classes, relations, and named individuals only.
- **`model.ttl` uses the canonical SEA vocabulary IRI regardless of
  `--base-iri`** (it is produced by the legacy `kg.rs` serializer, which is not
  forked). `--base-iri` reparameterizes the JSON-LD and OWL files only; keep the
  default for full cross-file IRI identity. Namespace-derived defaults are a
  documented follow-up.
- **No RDF-star, named-graph datasets, or SHACL-shape emission beyond what
  `kg.rs` already produces.**
- No `projection … target rdf` contract surface — the target is
  CLI-format-driven (per the Lean precedent).
