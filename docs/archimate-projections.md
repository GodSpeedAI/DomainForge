# ArchiMate Projection (`--format archimate`)

DomainForge projects a `.sea` model into an **ArchiMate 3.0 Model Exchange
File** — the enterprise-architecture operator of the projection family. The
output is a single, XSD-valid `model.xml` that validates against the vendored
Open Group Model Exchange File schema and imports into Archi and other
ArchiMate tools.

```bash
domainforge project --format archimate domain/model.sea out/
xmllint --schema schemas/archimate/archimate3_Diagram.xsd out/model.xml --noout   # XSD validation
```

The same artifact is available from the language bindings as a path → content
map (no filesystem access needed):

```python
artifacts = json.loads(graph.export_archimate(created_at="2026-07-02T00:00:00+00:00"))
```

```ts
const artifacts = JSON.parse(graph.exportArchimate(undefined, '2026-07-02T00:00:00+00:00'));
```

## What gets generated

A single `model.xml` containing one business-layer architecture:

| ArchiMate element | Derived from |
| --- | --- |
| `BusinessRole` | every declared role |
| `BusinessObject` | every declared entity **and** every declared resource (both are passive business-layer objects) |
| `BusinessProcess` | every declared flow — the activity that moves a resource from one entity to another |
| `Requirement` (motivation layer) | every authority policy — a driver the architecture must satisfy |
| `Assignment` relation | a role bound to a flow's source entity performs that flow's process |
| `Access` relation | a flow's process accesses the business objects for its source and target entities |
| `Triggering` relation | a flow whose target entity is another flow's source entity triggers that next flow's process |
| `Association` relation | a policy's condition expression references an object's name (token match) |
| one `Diagram` view | every business-layer element, laid out on a deterministic index-derived grid |

## Validation by construction (the core mechanism)

Where BPMN and CMMN validate their output *after* rendering (XSD + a
reference-integrity pass), ArchiMate's interesting risk is different: XSD
alone cannot reject an *illegal relation pairing* (e.g. an `Assignment` between
two passive objects) — the schema only enforces element/attribute shape, not
the ArchiMate relationship matrix. So this projection enforces the matrix
**at IR-construction time**, not after the fact:

- `domainforge-core/src/projection/archimate/ir.rs` declares a static
  `ALLOWED_RELATIONS: &[(ElemKind, RelKind, ElemKind)]` table — a curated,
  spec-legal subset of the full ArchiMate 3.1 relationship matrix covering
  only the element/relation kinds this projection mints.
- `Relation::build` is the *only* way to construct a `Relation`. It consults
  the table and returns `Err` naming the offending triple for anything absent
  from it — so the IR **cannot hold** an illegal relation. This is the
  teeth-check: a unit test (`forbidden_relation_tuple_is_rejected`) feeds a
  known-illegal triple into `Relation::build` and asserts `Err`.
- `ArchitectureIR::validate_references` is a second, defence-in-depth pass run
  inside `emit` (so broken output is never written): every relation endpoint
  must resolve to a real element of the declared kind, and every relation held
  by the IR is re-checked against the matrix.

## Non-goals (v1)

- **Per-stakeholder views.** Only one auto-generated business-layer view is
  emitted (listing every business-layer element). Splitting views by
  stakeholder/viewpoint is an additive v2 concern.
- **Application and technology layers.** The model's canonical vocabulary is
  business-layer (entities, roles, resources, flows, policies); inventing
  application/technology elements would fabricate semantics the model does
  not contain.
- **ArchiMate DI (visual notation) beyond node placement.** Views carry
  deterministic grid-positioned nodes so tools can open the file without a
  layout pass, but no styling, color, or connection routing is emitted.

## Determinism

Identical model + fixed `--created-at` produce byte-identical output:

- Roles, entities, resources, flows, and policies are all iterated in sorted
  (`BTreeMap`/`Vec::sort`) order; elements are sorted by `(kind, name, id)` and
  relations are collected into a `BTreeMap` keyed by relation id.
- Every element and relation id is minted through
  `domainforge-core/src/projection/ids.rs` (`element_id`, `archimate` family)
  and prefixed with an alpha tag so it is a legal XML `NCName`.
- View node layout is index-derived (a fixed grid), never authored or
  timestamp-dependent.
- The only timestamp in the file is the caller-supplied `--created-at`, in a
  provenance comment.

## Validation and the teeth-check

The primary gate is **XSD validation** against the vendored Model Exchange
File schema (`schemas/archimate/archimate3_Diagram.xsd`): run locally with
`xmllint --schema ... --noout`, in CI by the `verify-archimate` job, and in
`tests/test_archimate.py` via `lxml` (skipped if `lxml` is absent).
`archimate3_Diagram.xsd` is the entry point because it is the top of the
`xs:redefine` chain (it includes `archimate3_View.xsd`, which redefines
`archimate3_Model.xsd` to add `views`); the base `archimate3_Model.xsd` alone
has no `views` in its `ModelType`.

**The relation-matrix teeth-check is the point of this task**:
`forbidden_relation_tuple_is_rejected` (in
`domainforge-core/src/projection/archimate/mod.rs`) constructs a relation
triple that is not in `ALLOWED_RELATIONS` and asserts `Relation::build`
returns `Err` — proving illegal relations cannot reach the renderer at all,
not merely that a post-hoc check would catch them.

## Implementation

One IR module + one renderer + one `emit` + one binding surface, per the
shared projection pattern:

- `domainforge-core/src/projection/archimate/ir.rs` — `ArchitectureIR` (graph
  → elements, relation-matrix-checked relations, one business-layer view) and
  `validate_references`.
- `domainforge-core/src/projection/archimate/xml.rs` — the Model Exchange File
  renderer, reusing the generic XML writer built for the BPMN projection
  (`crate::projection::bpmn::xml::Xml`).
- `domainforge-core/src/projection/archimate/mod.rs` — `emit` and
  `project_archimate_in_memory`.

Schemas are vendored under `schemas/archimate/` (see
`schemas/archimate/VENDORED.md`).
