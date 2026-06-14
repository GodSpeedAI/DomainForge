# KG (RDF/Turtle) Projection Loss Manifest

**Phase 4 of the Semantic Infrastructure Audit** (`.agents/reports/semantic_infrastructure_audit_DomainForge_2026-06-12.md`, G3/G4).

KG (RDF/Turtle) export is **rich** but import is **lossy**. Consumers must not treat
re-imported graphs as a complete representation of the original model. The import
path (`KnowledgeGraph::to_graph()`) reconstructs only entities, resources, and flows;
every other domain construct either triggers a loud error (when its `rdf:type` is
recognized as a SEA domain class) or is absent from the Turtle vocabulary.

## Round-trip behavior

| Construct | Exported to Turtle? | Reconstructable on import? | Loss reason / notes |
|---|---|---|---|
| Entity | Yes (`sea:Entity`) | Yes | Full reconstruction including namespace. |
| Resource | Yes (`sea:Resource`) | Yes | Reconstructed with unit (via `sea:unit`). |
| Flow | Yes (`sea:Flow`) | Yes | Reconstructed via `sea:from`, `sea:to`, `sea:hasResource`, `sea:quantity`. |
| Role | Yes (`sea:Role`) | **No — loud error** | `to_graph()` detects `sea:Role` and returns `Err`. |
| Relation | Yes (`sea:Relation`) | **No — loud error** | `to_graph()` detects `sea:Relation` and returns `Err`. |
| Pattern | Yes (`sea:Pattern`) | **No — loud error** | `to_graph()` detects `sea:Pattern` and returns `Err`. |
| Instance | No | No | Not exported to Turtle; absent from round-trip. |
| Policy | No | **No — loud error** | Policies are not serialized as `sea:Policy` triples; if such triples appear (e.g. hand-authored Turtle), import errors. |
| Unit (as concept) | Partial (`sea:unit` literal on resources) | **No — loud error** | Unit *symbols* are preserved on resources; standalone `sea:Unit` concept definitions are not reconstructable and error on import. |
| Dimension | No | No | Not exported. |
| Quantifier (`forall`/`exists`) | No | **No — loud error** | Quantifier triples (`sea:forall`, `sea:exists`) are not reconstructable. |
| Temporal | No | No | Temporal semantics are not represented in Turtle. |
| Metric | No | No | Not exported. |
| Mapping | No | No | Not exported. |
| Projection | No | No | Not exported. |
| ConceptChange | No | No | Not exported. |
| Association | No | No | Not exported. |

## Import error behavior

When `import_kg_turtle` (or `to_graph()`) encounters triples whose `rdf:type` is a
SEA domain class outside the reconstructable set (`sea:Entity`, `sea:Resource`,
`sea:Flow`), the import returns:

```
KG import cannot reconstruct the following domain construct(s): sea:Role, sea:Relation.
Only entities, resources, and flows survive round-trip.
See docs/specs/kg_projection_loss.md for the full loss manifest.
```

This is a **deliberate behavior change** from silent dropping (audit G3/G4).
Consumers who previously relied on silent partial import must now handle the error
or restrict their Turtle to the supported subset.

## Pinned by

`conformance/10_kg_roundtrip/` — a model exercising both supported constructs
(entities/resources/flows) and unsupported constructs (policy, unit), with the
expected surviving subset and loud-error assertions pinned.
