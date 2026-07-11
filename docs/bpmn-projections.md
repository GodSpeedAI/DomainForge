# BPMN Projection (`--format bpmn`)

DomainForge projects a `.sea` model into a **BPMN 2.0 XML process** — the
ordered-process operator of the projection family. The output is a single,
XSD-valid `model.bpmn` that imports cleanly into
[bpmn.io](https://bpmn.io), Camunda Modeler, and any BPMN 2.0 tool.

```bash
domainforge project --format bpmn domain/model.sea out/
xmllint --schema schemas/bpmn/BPMN20.xsd out/model.bpmn --noout   # XSD validation
```

The same artifact is available from the language bindings as a path → content
map (no filesystem access needed):

```python
artifacts = json.loads(graph.export_bpmn(created_at="2026-07-02T00:00:00+00:00"))
```

```ts
const artifacts = JSON.parse(graph.exportBpmn(undefined, '2026-07-02T00:00:00+00:00'));
```

## Executable-subset declaration

This is a **declared, non-executable v1 subset**. The generated process has
`isExecutable="false"`: it carries the *structure and semantics* of the model's
ordered process — who does what, in what order, where work splits and joins —
but not the executable bindings (service tasks, forms, expressions, listeners)
an engine needs to *run* it. The philosophy: **semantics live in the model, and
layout is the tool's job.** DomainForge emits the meaning; the modeling tool
lays it out and (if you choose) you attach execution bindings downstream.

## What gets generated

A single `model.bpmn` containing one `<process>`:

| BPMN element | Derived from |
| --- | --- |
| `process` (`isExecutable="false"`) | the model |
| `task` | every entity that participates in a flow (as source or target) — the actor performing that step |
| `sequenceFlow` | every declared flow edge, plus start/end wiring |
| `parallelGateway` (`Diverging`) | an entity that fans **out** to more than one flow |
| `parallelGateway` (`Converging`) | an entity that fans **in** from more than one flow |
| `startEvent` | a single entry that fans out to every source entity (no incoming flow) |
| `endEvent` | a single exit fed by every sink entity (no outgoing flow) |
| `lane` (in a `laneSet`) | every declared role — a swimlane |
| `dataObject` | every declared resource |

### Gateways: why parallel, not exclusive

Flows in the model are quantity transfers between entities; they carry **no
branch predicate**. A fan-out therefore means "all of these happen" — an AND
(parallel) split — not "one of these happens" (an exclusive/XOR choice).
Discriminating exclusive gateways would require condition semantics on flows
that the model does not express today.

> **redesign:** exclusive/conditional gateways are deferred pending a
> spec-level decision to add branch-condition semantics to flows (a DSL change,
> out of scope for the projection). Until then every split/join is a
> `parallelGateway`. This is a deliberate thinner-projection choice, not a bug.

### Lanes and role binding

Every declared `Role` becomes a swimlane. A lane's `flowNodeRef`s (the tasks it
owns) are populated from entity→role bindings the graph carries: each
participating entity's first (sorted) role owns its task. The current `.sea`
text surface declares roles but does not expose syntax to *bind* a role to an
entity, so lanes are typically emitted empty (the graph model supports binding,
so the wiring activates automatically once a binding surface exists — see the
`bound_roles_populate_lane_flow_node_refs` test). Emitting empty swimlanes is
intentional: the organizational roles are still surfaced for the modeler.

## Non-goals (v1)

- **BPMNDI (visual layout).** No `<bpmndi:BPMNDiagram>` is emitted. Import tools
  auto-layout the process on open; hand-authoring coordinates in a deterministic
  exporter buys nothing. The BPMNDI XSDs are vendored (`schemas/bpmn/BPMNDI.xsd`)
  for completeness but unused.
- **Execution bindings.** No service-task implementations, forms, expressions,
  message/timer definitions, or listeners (`isExecutable="false"`).
- **Collaborations / pools / message flows.** One process per model; no
  cross-pool messaging.

## Determinism

Identical model + fixed `--created-at` produce byte-identical output:

- Entities, roles, resources, and flow edges are all iterated in sorted
  (`BTreeMap`/`BTreeSet`) order; nodes and sequence flows are emitted sorted by
  id. No `HashMap` iteration order reaches the renderer.
- Every element id is minted through
  `domainforge-core/src/projection/ids.rs` (`element_id`, `bpmn` family) and
  prefixed with an alpha tag so it is a legal XML `NCName`. The same model
  yields the same ids run-to-run.
- The only timestamp in the file is the caller-supplied `--created-at`, in a
  provenance comment.

## Validation and the teeth-check

The primary gate is **XSD validation** against the vendored BPMN 2.0 schema
(`schemas/bpmn/BPMN20.xsd`, entry point that includes `Semantic.xsd`): run
locally with `xmllint --schema ... --noout`, in CI by the `verify-bpmn` job, and
in `tests/test_bpmn.py` via `lxml` (skipped if `lxml` is absent).

**Structural reference integrity is checked separately, in Rust.** BPMN's
`sequenceFlow`/`incoming`/`outgoing` references are `xsd:QName` (not `xsd:IDREF`),
so XSD validation does **not** resolve them against document ids — a BPMN file
with a dangling flow reference is still "XSD-valid". `ProcessIR::validate_references`
(called inside `emit`, so broken output is never written) fills that gap: it
verifies every node reference resolves to a declared sequence flow, every
sequence-flow endpoint to a declared node, and every lane `flowNodeRef` to a
declared node. The teeth-check test
(`deleting_a_sequence_flow_fails_structural_validation`) removes one sequence
flow from the IR and asserts this validator fails — the XSD alone cannot catch
that, which is exactly why the structural check exists.

## Implementation

One IR module + one renderer + one `emit` + one binding surface, per the shared
projection pattern:

- `domainforge-core/src/projection/bpmn/ir.rs` — `ProcessIR` (graph → nodes,
  gateways, sequence flows, lanes, data objects) and `validate_references`.
- `domainforge-core/src/projection/bpmn/xml.rs` — the BPMN 2.0 renderer over a
  small, generic XML writer (kept separable for the CMMN/ArchiMate targets that
  reuse XML infrastructure).
- `domainforge-core/src/projection/bpmn/mod.rs` — `emit` and
  `project_bpmn_in_memory`.

Schemas are vendored under `schemas/bpmn/` (see `schemas/bpmn/VENDORED.md`).
