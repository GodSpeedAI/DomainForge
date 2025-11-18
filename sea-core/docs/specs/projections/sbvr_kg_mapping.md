# SBVR/KG Projection Mapping (Normative)

This document records the minimal SBVR→SEA and KG (RDF/SHACL)→SEA mapping implemented in Phase 18.

## High-level mappings

- SBVR:
  - `sbvr:GeneralConcept` → SEA `Entity`
  - `sbvr:IndividualConcept` → SEA `Resource`
  - `sbvr:VerbConcept` → not represented (verbs are currently not mapped to primitives)
  - `sbvr:FactType` (verb: `transfers`) → SEA `Flow` (quantity is defaulted to 1 in this import)
  - `sbvr` rules (Obligation/Prohibition/Permission/Derivation) → imported to `SbvrModel::rules` but not converted into `Policy` primitives in Phase 18

- KG / RDF / SHACL:
  - `sea:Entity` (rdf:type) + `rdfs:label` → SEA `Entity`
  - `sea:Resource` (rdf:type) + `sea:unit` → SEA `Resource` with unit mapping via `unit_from_string`
  - `sea:Flow` + `sea:from`, `sea:to`, `sea:hasResource`, `sea:quantity` → SEA `Flow` with quantity parsed as decimal

## Limitations

- SBVR `FactType` currently encodes only a subject/verb/object/destination with default quantity `1`. Complex SBVR expressions and modalities are stored in `SbvrModel` but not converted to SEA `Policy` primitives in Phase 18.
- The KG import is intentionally a best-effort parser tailored to the Turtle exports produced by this crate (`KnowledgeGraph::to_turtle`). It is not a general-purpose Turtle/RDF parser.
- SHACL validation is not enforced in the import functions; the CLI will validate KG exports in the planned steps.

## Example: SBVR Fact → Flow

Given SBVR XMI with a FactType referencing a `Subject` (GeneralConcept), `Verb` (transfers), `Object` (resource) and `Destination`, the import will map it to a `Flow`:

- Subject `Warehouse` → SEA `Entity` named "Warehouse"
- Verb `transfers` → recognized but not materialized
- Object `Cameras` → SEA `Resource` named "Cameras"; the importer calls `sea_core::units::unit_from_string("units")` (see `sea-core/src/units/mod.rs`), so the default unit literal `"units"` maps to a `Unit` whose `symbol` and `base_unit` are `"units"`, `dimension` is `Dimension::Count`, and `base_factor` is `Decimal::ONE`.
- Destination `Factory` → SEA `Entity` named "Factory"
- The resulting Flow will be added with quantity `1` unless a quantity fact is present (Phase 19+)

## Notes for Integrators

- If you have XMI produced by other SBVR tools, import using `SbvrModel::from_xmi` and then convert to `Graph` with `to_graph`.
- For RDF/Turtle produced externally, it's recommended to verify the triple patterns match the `sea:` predicates used here (`sea:Flow`, `sea:hasResource`, `sea:quantity`).
- Future phases will expand the mapping to support richer SBVR constructs, modalities, and using SHACL validation via `oxigraph`.

---

For more details and examples, review unit tests in `sea-core/tests/projection_contracts_tests.rs`.
