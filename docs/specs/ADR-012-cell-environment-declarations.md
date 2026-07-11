# ADR-012: Cell Environment Declarations and Projection

**Status:** Proposed (with a disclosed deviation — see "Deviation from
ADR-011" below). Pending human ratification; change to "Accepted" only after
recording the ratifier and approval date below.
**Date:** 2026-07-11
**Deciders:** DomainForge Architecture Team (proposed by implementation
agent; needs human ratification before this status is final)

> **Ratification record** _(fill in when accepted)_:
> - Ratifier:
> - Approval date:

## Proposed distinction

`.sea` currently has no way to declare an agent's *execution environment*:
the OS packages, language runtimes, application dependency closure,
sandbox/network isolation, and authority/evidence obligations a `Cell`
(an isolated execution unit) needs to run hermetically. Ten new
declarations close that gap: `Cell`, `SystemDependency`, `Runtime`, `Tool`,
`DependencySet`, `Service`, `Mount`, `Endpoint`, `NetworkFlow`,
`Credential`.

## Current representational limitation

No existing SEA category expresses "isolated execution unit with a pinned
system/runtime/dependency closure and network policy." `Entity` and
`Resource` model business/domain concepts, not execution environments;
overloading them with environment-shaped annotations would make ordinary
domain models unreadable and would not give the projection a stable place
to hang IDs, validation, or a normative mapping.

## Existing mechanisms considered and why they fail

- **Annotations on `Entity`**: no natural entity to annotate (an execution
  environment is not a domain entity), and annotations are per-declaration
  free-form data, not validated, typed structure.
- **`Mapping`/`Projection` overrides**: these retarget *existing* primitive
  categories into external formats; they cannot introduce new required
  fields like `@ecosystem`/`@manifest`/`@lockfile` with their own
  validation.
- **A single generic `Config` declaration carrying JSON**: would satisfy
  the parser trivially but throws away the language's whole value
  proposition — typed, checked declarations with stable semantic ids — and
  fails "no renderer convenience" (see
  `docs/reference/sea-language-evolution-policy.md`) for the same reason a
  bag of annotations does.

## Cross-target semantics

All ten declarations feed exactly one new projection target (`--format
cell`), but the categories themselves are meaningful independent of that
renderer: `SystemDependency`/`Runtime`/`Tool` describe facts about what a
cell needs regardless of whether the output is Devbox+Mise or some future
target; `Endpoint`/`NetworkFlow`/`Credential` describe network/authority
facts reusable by any future authority tooling.

## Normative mappings

See `docs/cell-environment-projections.md` §"Normative SEA → Cell IR
mapping" for the full table. Summary: `SystemDependency` → Devbox package;
`Runtime`/`Tool` → Mise `[tools]`; `DependencySet` → native lockfile
reference + install task; `Service` → Devbox package + service manifest
entry; `Mount` → sandbox mount policy; `Endpoint`/`NetworkFlow` → network
policy (deny-by-default allow-list); `Credential` → authority credential
contract (never a literal value); `Policy`/`Metric` (existing categories,
reused unmodified) → authority decisions / evidence probes.

## Grammar / AST impact

- `domainforge-core/grammar/sea.pest`: ten new `*_decl` rules plus a shared
  `generic_annotation = { "@" ~ identifier ~ annotation_value }` rule
  (matching the existing `Flow`'s `identifier ~ annotation_value` fallback
  annotation form — see `flow_annotation`), added to `declaration_inner`
  and `declaration_keyword`.
- `domainforge-core/src/parser/ast.rs`: ten new `AstNode` variants (name +
  optional version + `annotations: HashMap<String, JsonValue>`, matching
  the `Entity`/`Resource` shape), ten `parse_*` functions, and a shared
  `parse_generic_annotations` helper.
- `domainforge-core/src/parser/ast_schema.rs` + `ast_convert.rs`: mirrored
  schema variants and `From` conversions (additive to `ast-v3.schema.json`
  — no new schema version).
- `domainforge-core/src/formatter/printer.rs` and
  `domainforge-core/src/parser/printer.rs`: a shared
  `format_generic_decl` helper covering all ten declarations (keyword,
  quoted name, optional `version`/`from`/`to`, sorted `@key value`
  annotation lines).
- `domainforge-core/src/module/resolver.rs`: `declaration_name` extended
  for module-export resolution.

## Deviation from ADR-011

ADR-011 §1 states every projection family must build "**one IR module** —
a target-specific intermediate representation built from **the graph**
(never rendered straight from the AST)." The cell-environment IR
(`domainforge-core/src/projection/cell/ir.rs`) violates this: `CellIr::build`
is built from the parsed `Ast` directly (plus `Policy`/`Metric`, which are
also read from the `Ast`, not `Graph`), not from `Graph`.

**Why:** none of the ten new declaration kinds are consumed by any other
projection family — unlike `Entity`/`Resource`/`Flow`, which feed fifteen-
plus renderers and therefore earn their place in the shared `Graph`
(`IndexMap` storage, `add_*`/`all_*` accessors, `to_ast.rs` round-trip,
`extend()` merge semantics). Routing ten single-consumer declaration kinds
through that machinery would roughly double the size of this change for
zero reuse benefit today.

**This is a disclosed, not silent, deviation, and it is not a closed
question.** Before this ADR can be considered fully compliant with
ADR-011, a follow-up must do one of:

1. add `Graph` storage for the ten declarations (bringing this projection
   into full ADR-011 conformance), or
2. amend ADR-011 to carve out an explicit exception for single-consumer
   projection families, with this projection as the worked example.

Until one of those lands, treat `projection::cell` as an accepted,
documented exception rather than a precedent other projection families
should copy without the same single-consumer justification.

## Compatibility impact

Purely additive: ten new top-level keywords, all previously reserved
grammar keywords extended with the same names in `declaration_keyword` so
they cannot be misparsed as identifiers elsewhere. No existing declaration,
grammar rule, or serialized AST shape changes. All 366 pre-existing
`domainforge-core` tests pass unmodified after this change.

## Migration path

None required — additive only.

## Fixtures

`fixtures/cell_env/{repair-agent,basic-typescript,basic-rust,polyglot,
format_stable.sea,invalid/*}` (see
`docs/cell-environment-projections.md` §"Generated tree" and
`domainforge-core/tests/cell_projection_tests.rs`).

## Tests

Grammar/AST/formatter round-trip: `domainforge-core/src/parser/mod.rs`
existing suite plus ad hoc round-trip verification during implementation
(parse → format → parse → format idempotency on all ten declarations).
IR/renderer/override/golden coverage:
`domainforge-core/src/projection/cell/**/*` `#[cfg(test)]` modules (21
tests) and `domainforge-core/tests/cell_projection_tests.rs` (the
15-condition flagship proof plus the eight `CELL0xx` invalid-fixture
cases).

## Failure modes

Enumerated as `CELL001`–`CELL021` in
`docs/cell-environment-projections.md` §"Error codes"; every one fails
closed (unknown profile, missing Cell, duplicate declaration, dangling
`NetworkFlow` reference, wildcard endpoint, version-weakening override,
resource-increasing override, dangling override reference, incomplete or
expired unsafe-override metadata, malformed/unknown-schema override
document).

## Disconfirmation criterion

If a second projection target ever needs to consume
`Cell`/`SystemDependency`/`Runtime`/`Tool`/`DependencySet`/`Service`/
`Mount`/`Endpoint`/`NetworkFlow`/`Credential` data, the "single consumer"
justification for bypassing `Graph` no longer holds and the declarations
must be migrated into `Graph` per ADR-011 before that second target is
built.
