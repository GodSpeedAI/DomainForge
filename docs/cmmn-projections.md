# CMMN Projection (`--format cmmn`)

DomainForge projects a `.sea` model into a **CMMN 1.1 XML case** — the
adaptive-work / case operator of the projection family. The output is a single,
XSD-valid `model.cmmn` that validates against the vendored OMG CMMN 1.1 schema.

```bash
domainforge project --format cmmn domain/model.sea out/
xmllint --schema schemas/cmmn/CMMN11.xsd out/model.cmmn --noout   # XSD validation
```

The same artifact is available from the language bindings as a path → content
map (no filesystem access needed):

```python
artifacts = json.loads(graph.export_cmmn(created_at="2026-07-02T00:00:00+00:00"))
```

```ts
const artifacts = JSON.parse(graph.exportCmmn(undefined, '2026-07-02T00:00:00+00:00'));
```

## Why case management, not process

Where BPMN models an *ordered* process (fixed sequence of steps), CMMN models
**adaptive work**: a case file, the work items available in it, and the
*conditions* under which milestones are reached. DomainForge's authority
policies are exactly such conditions, which is why the interesting part of this
projection is the **policy → sentry** lowering.

## What gets generated

A single `model.cmmn` containing one `<case>`:

| CMMN element | Derived from |
| --- | --- |
| `case` | the model |
| `caseFileItem` (+ `caseFileItemDefinition`) | every declared resource |
| `role` (in `caseRoles`) | every declared role |
| `humanTask` | every entity — a work item in the case; `performerRef` names the entity's bound role when the graph carries a binding |
| `milestone` | every authority policy — the achievement "policy satisfied" |
| `sentry` (with `ifPart`/`condition`) | every policy **condition** — the policy expression, rendered as the sentry's `ifPart` condition text |
| `entryCriterion` (on the milestone's `planItem`) | the link gating a milestone on its policy's sentry |

### Policy → sentry (the core mapping)

Each authority `Policy name as: <expression>` lowers to:

1. a **milestone** `"<name> satisfied"`,
2. a **sentry** whose `ifPart` `<condition>` holds the policy's `<expression>`
   rendered as text (with `language="http://domainforge.ai/expression"` so a
   consumer does not mistake it for XPath), and
3. an **entryCriterion** on the milestone's plan item, referencing that sentry.

Semantically: the milestone is reached when the sentry fires, i.e. when the
policy condition holds. This is a faithful, non-trivial lowering — the policy
expression carries into the case as the achievement condition — so the projection
emits full policy-gated milestones rather than the narrower milestone-only
fallback the plan allowed for.

### Roles and performer binding

Every declared `Role` becomes a CMMN case role. A human task's `performerRef` is
populated from entity→role bindings the graph carries: each entity's first
(sorted) bound role performs its task. The current `.sea` text surface declares
roles but exposes no syntax to *bind* a role to an entity, so `performerRef` is
typically absent (the graph model supports binding, so the wiring activates
automatically once a binding surface exists — see the
`bound_role_populates_performer_ref` test).

## Non-goals (v1)

- **Event listeners.** No `userEventListener` or `timerEventListener` is emitted.
  These model *runtime* events (a user click, a timer firing), and the canonical
  DomainForge model carries no runtime-event source — inventing one would
  fabricate semantics the model does not contain. This is a deliberate omission,
  not an oversight.
- **CMMN DI (visual layout).** No `<cmmndi:CMMNDI>` is emitted; CMMN tools
  auto-layout on open. The DI XSDs are vendored (`schemas/cmmn/CMMNDI11.xsd`) for
  completeness but unused.
- **Stages, discretionary tasks, planning tables, process/decision/case tasks,
  repetition/required/manual-activation rules.** The v1 subset is a flat case
  plan (tasks + milestones + sentries); nested stages and discretionary planning
  are additive follow-ups.
- **Exit criteria.** Only entry criteria (milestone gating) are emitted;
  policies express reachability conditions, not termination conditions.

## Determinism

Identical model + fixed `--created-at` produce byte-identical output:

- Roles, resources, entities, and policies are all iterated in sorted
  (`BTreeMap`/`Vec::sort`) order; plan items, sentries, and definitions are
  emitted sorted. No `HashMap` iteration order reaches the renderer.
- Every element id is minted through
  `domainforge-core/src/projection/ids.rs` (`element_id`, `cmmn` family) and
  prefixed with an alpha tag so it is a legal XML `NCName`. The same model
  yields the same ids run-to-run.
- The only timestamp in the file is the caller-supplied `--created-at`, in a
  provenance comment.

## Validation and the teeth-check

The primary gate is **XSD validation** against the vendored CMMN 1.1 schema
(`schemas/cmmn/CMMN11.xsd`, entry point that includes `CMMN11CaseModel.xsd`): run
locally with `xmllint --schema ... --noout`, in CI by the `verify-cmmn` job, and
in `tests/test_cmmn.py` via `lxml` (skipped if `lxml` is absent).

**Structural reference integrity is checked separately, in Rust.** A plan item's
`definitionRef`, an entry criterion's `sentryRef`, and a human task's
`performerRef` are `xsd:IDREF`, whose referential integrity XSD validation does
not resolve against document ids — a CMMN file with a dangling reference is still
"XSD-valid". `CaseIR::validate_references` (called inside `emit`, so broken
output is never written) fills that gap. The teeth-check test
(`deleting_a_sentry_fails_structural_validation`) removes one sentry from the IR
and asserts this validator fails.

The **policy→sentry teeth-check** lives in the integration test
`removing_a_policy_removes_its_sentry`: a fixture *with* a policy condition emits
a sentry (and its milestone); the same fixture *without* the policy emits none.

## Implementation

One IR module + one renderer + one `emit` + one binding surface, per the shared
projection pattern:

- `domainforge-core/src/projection/cmmn/ir.rs` — `CaseIR` (graph → case file
  items, roles, human tasks, milestones, sentries, plan items) and
  `validate_references`.
- `domainforge-core/src/projection/cmmn/xml.rs` — the CMMN 1.1 renderer, reusing
  the generic XML writer built for the BPMN projection
  (`crate::projection::bpmn::xml::Xml`).
- `domainforge-core/src/projection/cmmn/mod.rs` — `emit` and
  `project_cmmn_in_memory`.

Schemas are vendored under `schemas/cmmn/` (see `schemas/cmmn/VENDORED.md`).
